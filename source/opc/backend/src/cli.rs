mod args;
/// OPC CLI — HTTP client for all backend features
/// Architecture: CLI → reqwest → running opc-server HTTP server
mod client;
mod helpers;
mod output;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use serde_json::{Value, json};

pub use self::args::Cli;
use self::args::{
    AiAction, ApiKeyAction, ArAction, AuthAction, BackupAction, BankAction, Commands,
    CompanyAction, CrudAction, CurrencyAction, FileAction, MemberAction, OpcAction, PdfAction,
    PolicyAction, ReportAction, TaxToolAction, WebhookAction,
};
use self::client::CliClient;
use self::helpers::{build_body, entity_api_path, parse_json_arg, read_contract_text};
use self::output::{
    print_action_result, print_delete_result, print_download_result, print_list_response,
    print_mutation_result, print_status_result,
};

// ═══════════════════════════════════════════════════════════════
// Command Runner
// ═══════════════════════════════════════════════════════════════

pub async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let c = CliClient::new(&cli)?;

    match cli.command {
        // ── System ─────────────────
        Commands::Status => {
            let v = c.get("/api/v1/system/status").await?;
            c.print_value(&v);
        }
        Commands::Dashboard => {
            let v = c.get("/api/v1/stats/dashboard").await?;
            c.print_value(&v);
        }
        Commands::Logs { limit } => {
            let v = c
                .get(&format!("/api/v1/operation_logs?limit={}", limit))
                .await?;
            print_list_response(&c, &v);
        }

        // ── Backup ─────────────────
        Commands::Backup { action } => match action {
            BackupAction::Create => {
                let v = c.post("/api/v1/settings/backups", &json!({})).await?;
                print_mutation_result(&c, &v, "✅ Backup created");
            }
            BackupAction::List => {
                let v = c.get("/api/v1/settings/backups").await?;
                print_list_response(&c, &v);
            }
        },

        // ── Auth ───────────────────
        Commands::Auth { action } => match action {
            AuthAction::Login => {
                println!("🔑 Open this URL in your browser to log in:");
                println!("   {}/auth/login", c.base_url);
                println!("\nAfter login, set the session cookie:");
                println!("   export OPC_SESSION=<session_id>");
            }
            AuthAction::Status => match c.get("/auth/me").await {
                Ok(v) => {
                    println!("✅ Logged in");
                    c.print_value(&v);
                }
                Err(e) => println!("❌ Not authenticated: {}", e),
            },
        },

        // ── Company ────────────────
        Commands::Company { action } => match action {
            CompanyAction::List => {
                let v = c.get("/api/companies").await?;
                print_list_response(&c, &v);
            }
            CompanyAction::Create { json, fields } => {
                let body = build_body(&json, &fields)?;
                let v = c.post("/api/companies", &body).await?;
                print_mutation_result(&c, &v, "✅ Company created");
            }
            CompanyAction::Get { id } => {
                let v = c.get(&format!("/api/companies/{}", id)).await?;
                c.print_value(&v);
            }
            CompanyAction::Update { id, json, fields } => {
                let body = build_body(&json, &fields)?;
                let v = c.put(&format!("/api/companies/{}", id), &body).await?;
                print_mutation_result(&c, &v, "✅ Company updated");
            }
            CompanyAction::Delete { id } => {
                c.delete_req(&format!("/api/companies/{}", id)).await?;
                print_delete_result(&c, &id, &format!("✅ Company {} deleted", id));
            }
            CompanyAction::Switch { id } => {
                let v = c
                    .post(&format!("/api/companies/{}/switch", id), &json!({}))
                    .await?;
                print_status_result(
                    &c,
                    &v,
                    "current_company_id",
                    &format!("✅ Switched to company {}", id),
                );
            }
            CompanyAction::Members { action } => match action {
                MemberAction::List { company_id } => {
                    let v = c
                        .get(&format!("/api/companies/{}/members", company_id))
                        .await?;
                    print_list_response(&c, &v);
                }
                MemberAction::Add {
                    company_id,
                    email,
                    role,
                } => {
                    let v = c
                        .post(
                            &format!("/api/companies/{}/members", company_id),
                            &json!({ "email": email, "role": role }),
                        )
                        .await?;
                    print_mutation_result(&c, &v, "✅ Member added");
                }
                MemberAction::Remove {
                    company_id,
                    member_id,
                } => {
                    c.delete_req(&format!(
                        "/api/companies/{}/members/{}",
                        company_id, member_id
                    ))
                    .await?;
                    print_delete_result(&c, &member_id, "✅ Member removed");
                }
                MemberAction::UpdateRole {
                    company_id,
                    member_id,
                    role,
                } => {
                    let v = c
                        .put(
                            &format!("/api/companies/{}/members/{}", company_id, member_id),
                            &json!({ "role": role }),
                        )
                        .await?;
                    print_mutation_result(&c, &v, "✅ Role updated");
                }
            },
        },

        // ── Bank ───────────────────
        Commands::Bank { action } => match action {
            BankAction::Accounts => {
                let v = c.get("/api/bank/accounts").await?;
                print_list_response(&c, &v);
            }
            BankAction::CreateAccount { json, fields } => {
                let body = build_body(&json, &fields)?;
                let v = c.post("/api/bank/accounts", &body).await?;
                print_mutation_result(&c, &v, "✅ Bank account created");
            }
            BankAction::Transact {
                account,
                txn_type,
                amount,
                date,
                description,
            } => {
                let mut body = json!({
                    "account_id": account,
                    "transaction_type": txn_type,
                    "amount": amount,
                    "transaction_date": date,
                });
                if let Some(desc) = description {
                    body["description"] = json!(desc);
                }
                let v = c.post("/api/bank/transactions", &body).await?;
                print_mutation_result(&c, &v, "✅ Transaction created");
            }
        },

        // ── AI ─────────────────────
        Commands::Ai { action } => match action {
            AiAction::Chat { message } => {
                let v = c
                    .post("/api/ai/chat", &json!({ "message": message }))
                    .await?;
                if let Some(reply) = v["reply"].as_str().or(v["response"].as_str()) {
                    println!("{}", reply);
                } else {
                    c.print_value(&v);
                }
            }
            AiAction::Analyze {
                data_type,
                data,
                question,
            } => {
                let parsed_data = parse_json_arg("--data", &data)?;
                let mut body = json!({
                    "data_type": data_type,
                    "data": parsed_data,
                });
                if let Some(question) = question {
                    body["question"] = json!(question);
                }
                let v = c.post("/api/ai/analyze", &body).await?;
                c.print_value(&v);
            }
            AiAction::Audit {
                audit_type,
                application_id,
                material_id,
                contract_id,
                employee_id,
                budget_id,
                project_id,
            } => {
                let mut body = json!({ "audit_type": audit_type });
                if let Some(id) = application_id {
                    body["application_id"] = json!(id);
                }
                if let Some(id) = material_id {
                    body["material_id"] = json!(id);
                }
                if let Some(id) = contract_id {
                    body["contract_id"] = json!(id);
                }
                if let Some(id) = employee_id {
                    body["employee_id"] = json!(id);
                }
                if let Some(id) = budget_id {
                    body["budget_id"] = json!(id);
                }
                if let Some(id) = project_id {
                    body["project_id"] = json!(id);
                }
                let v = c.post("/api/ai/audit", &body).await?;
                c.print_value(&v);
            }
            AiAction::ParseDoc {
                file,
                document_type,
                model,
            } => {
                let content = std::fs::read(&file)?;
                let filename = std::path::Path::new(&file)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&file);
                let mut body = json!({
                    "file_base64": BASE64_STANDARD.encode(content),
                    "document_type": document_type,
                    "filename": filename,
                });
                if let Some(model) = model {
                    body["model"] = json!(model);
                }
                let v = c.post("/api/ai/parse-document", &body).await?;
                c.print_value(&v);
            }
            AiAction::ContractReview {
                text,
                file,
                contract_type,
                counterparty_type,
            } => {
                let contract_text = read_contract_text(text, file)?;
                let mut body = json!({ "contract_text": contract_text });
                if let Some(contract_type) = contract_type {
                    body["contract_type"] = json!(contract_type);
                }
                if let Some(counterparty_type) = counterparty_type {
                    body["counterparty_type"] = json!(counterparty_type);
                }
                let v = c.post("/api/ai/contract-review", &body).await?;
                c.print_value(&v);
            }
        },

        // ── Reports ────────────────
        Commands::Report { action } => {
            let path = match action {
                ReportAction::FinancialHealth => "/api/v1/reports/financial-health",
                ReportAction::Cashflow => "/api/v1/reports/cashflow-forecast",
                ReportAction::Profit => "/api/v1/reports/profit-analysis",
                ReportAction::Deals => "/api/v1/reports/deals-conversion",
            };
            let v = c.get(path).await?;
            c.print_value(&v);
        }

        // ── PDF ────────────────────
        Commands::Pdf { action } => {
            let (id, path, output) = match action {
                PdfAction::Invoice { id, output } => {
                    let output = output.unwrap_or(format!("invoice-{}.pdf", id));
                    let path = format!("/api/v1/invoices/{}/pdf", id);
                    (id, path, output)
                }
                PdfAction::Quote { id, output } => {
                    let output = output.unwrap_or(format!("quote-{}.pdf", id));
                    let path = format!("/api/v1/quotes/{}/pdf", id);
                    (id, path, output)
                }
            };
            let (bytes, _ct) = c.get_bytes(&path).await?;
            std::fs::write(&output, &bytes)?;
            print_download_result(&c, &id, &output);
        }

        // ── Policies ───────────────
        Commands::Policies { action } => match action {
            PolicyAction::List => {
                let v = c.get("/api/policies").await?;
                print_list_response(&c, &v);
            }
            PolicyAction::Match => {
                let v = c.get("/api/policies/match").await?;
                print_list_response(&c, &v);
            }
            PolicyAction::Get { id } => {
                let v = c.get(&format!("/api/policies/{}", id)).await?;
                c.print_value(&v);
            }
            PolicyAction::Applications { action } => {
                handle_crud(&c, "/api/policies/applications", action).await?;
            }
        },

        Commands::Locations { action } => match action {
            None => {
                let v = c.get("/api/global_opc_hubs").await?;
                print_list_response(&c, &v);
            }
            Some(OpcAction::Calculate { revenue, col_index }) => {
                let v = c
                    .post(
                        "/api/global_opc_hubs/calculator",
                        &json!({
                            "est_annual_revenue": revenue,
                            "current_base_col_index": col_index,
                        }),
                    )
                    .await?;
                c.print_value(&v);
            }
        },

        // ── Files ──────────────────
        Commands::Files { action } => match action {
            FileAction::Upload {
                file,
                entity_type,
                entity_id,
            } => {
                let path = std::path::Path::new(&file);
                if !path.exists() {
                    eprintln!("❌ File not found: {}", file);
                    std::process::exit(1);
                }
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let file_bytes = std::fs::read(path)?;
                let mime = mime_guess::from_path(path)
                    .first_or_octet_stream()
                    .to_string();

                let part = reqwest::multipart::Part::bytes(file_bytes)
                    .file_name(file_name.clone())
                    .mime_str(&mime)
                    .unwrap();
                let form = reqwest::multipart::Form::new().part("file", part);

                let upload_url = match (entity_type.as_deref(), entity_id.as_deref()) {
                    (Some(et), Some(eid)) => {
                        format!("{}/api/files/upload/{}/{}", c.base_url, et, eid)
                    }
                    (None, None) => format!("{}/api/files/upload", c.base_url),
                    _ => return Err("provide both --entity-type and --entity-id together".into()),
                };

                let resp = c.client.post(upload_url).multipart(form).send().await?;
                if !resp.status().is_success() {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    eprintln!("❌ Upload failed ({}): {}", status, text);
                    std::process::exit(1);
                }
                let v: Value = resp.json().await?;
                print_mutation_result(
                    &c,
                    &v,
                    &format!(
                        "✅ Uploaded: {} → {}",
                        file_name,
                        v["id"].as_str().unwrap_or("?")
                    ),
                );
            }
            FileAction::List {
                entity_type,
                entity_id,
            } => {
                let v = c
                    .get(&format!("/api/files/list/{}/{}", entity_type, entity_id))
                    .await?;
                print_list_response(&c, &v);
            }
            FileAction::Download { id, output } => {
                let (bytes, _ct) = c.get_bytes(&format!("/api/files/{}", id)).await?;
                let out = output.unwrap_or_else(|| format!("file-{}", id));
                std::fs::write(&out, &bytes)?;
                print_download_result(&c, &id, &out);
            }
            FileAction::Delete { id } => {
                c.delete_req(&format!("/api/files/{}", id)).await?;
                print_delete_result(&c, &id, &format!("✅ File {} deleted", id));
            }
        },

        // ── API Keys ──────────────
        Commands::ApiKeys { action } => match action {
            ApiKeyAction::List => {
                let v = c.get("/api/api-keys").await?;
                print_list_response(&c, &v);
            }
            ApiKeyAction::Create {
                name,
                expires_in_days,
            } => {
                let mut body = json!({});
                if let Some(n) = name {
                    body["name"] = json!(n);
                }
                if let Some(d) = expires_in_days {
                    body["expires_in_days"] = json!(d);
                }
                let v = c.post("/api/api-keys", &body).await?;
                print_mutation_result(
                    &c,
                    &v,
                    "✅ API Key created (save the key — it won't be shown again!)",
                );
            }
            ApiKeyAction::Revoke { id } => {
                let v = c
                    .post(&format!("/api/api-keys/{}/revoke", id), &json!({}))
                    .await?;
                print_action_result(&c, &v, &id, &format!("✅ API Key {} revoked", id));
            }
            ApiKeyAction::Delete { id } => {
                c.delete_req(&format!("/api/api-keys/{}", id)).await?;
                print_delete_result(&c, &id, &format!("✅ API Key {} deleted", id));
            }
        },

        // ── Tax Tools ─────────────
        Commands::Tax { action } => match action {
            TaxToolAction::Calculate { amount } => {
                let mut qs = String::new();
                if let Some(a) = amount {
                    qs = format!("?amount={}", a);
                }
                let v = c.get(&format!("/api/v1/tax/calculate{}", qs)).await?;
                c.print_value(&v);
            }
            TaxToolAction::ComplianceCheck => {
                let v = c.get("/api/v1/tax/compliance-check").await?;
                c.print_value(&v);
            }
            TaxToolAction::InvoiceRequirements => {
                let v = c.get("/api/v1/tax/invoice-requirements").await?;
                c.print_value(&v);
            }
            TaxToolAction::InputCredit => {
                let v = c.get("/api/v1/tax/input-credit").await?;
                c.print_value(&v);
            }
        },

        // ── Currency ──────────────
        Commands::Currency { action } => match action {
            CurrencyAction::Convert { from, to, amount } => {
                let v = c
                    .get(&format!(
                        "/api/v1/currency/convert?from={}&to={}&amount={}",
                        from, to, amount
                    ))
                    .await?;
                c.print_value(&v);
            }
            CurrencyAction::Balances => {
                let v = c.get("/api/v1/currency/balances").await?;
                c.print_value(&v);
            }
        },

        // ── AR ─────────────────────
        Commands::Ar { action } => match action {
            ArAction::Aging => {
                let v = c.get("/api/v1/ar/aging").await?;
                c.print_value(&v);
            }
            ArAction::Summary => {
                let v = c.get("/api/v1/ar/summary").await?;
                c.print_value(&v);
            }
        },

        // ── Webhooks ──────────────
        Commands::Webhooks { action } => match action {
            WebhookAction::List => {
                let v = c.get("/api/v1/webhooks").await?;
                print_list_response(&c, &v);
            }
            WebhookAction::Create { url, events } => {
                let event_list: Vec<&str> = events.split(',').collect();
                let v = c
                    .post(
                        "/api/v1/webhooks",
                        &json!({
                            "url": url,
                            "events": event_list,
                        }),
                    )
                    .await?;
                print_mutation_result(&c, &v, "✅ Webhook created");
            }
            WebhookAction::Delete { id } => {
                c.delete_req(&format!("/api/v1/webhooks/{}", id)).await?;
                print_delete_result(&c, &id, &format!("✅ Webhook {} deleted", id));
            }
            WebhookAction::Test { id } => {
                let v = c
                    .post(&format!("/api/v1/webhooks/{}/test", id), &json!({}))
                    .await?;
                print_action_result(&c, &v, &id, "✅ Webhook test sent");
            }
        },

        // ── Conversions ───────────
        Commands::TimeToInvoice { entries, rate } => {
            let entry_ids: Vec<&str> = entries.split(',').collect();
            let mut body = json!({ "time_entry_ids": entry_ids });
            if let Some(r) = rate {
                body["hourly_rate"] = json!(r);
            }
            let v = c.post("/api/v1/time_entries/invoice", &body).await?;
            print_mutation_result(&c, &v, "✅ Invoice created from time entries");
        }
        Commands::QuoteToInvoice { quote_id } => {
            let v = c
                .post(
                    &format!("/api/v1/quotes/{}/convert-to-invoice", quote_id),
                    &json!({}),
                )
                .await?;
            print_mutation_result(&c, &v, "✅ Quote converted to invoice");
        }

        // ── Generic CRUD ───────────
        Commands::List {
            entity,
            limit,
            page,
        } => {
            let base = entity_api_path(&entity);
            let mut qs = format!("limit={}", limit);
            if let Some(p) = page {
                qs.push_str(&format!("&page={}", p));
            }
            let v = c.get(&format!("{}?{}", base, qs)).await?;
            print_list_response(&c, &v);
        }
        Commands::Get { entity, id } => {
            let base = entity_api_path(&entity);
            let v = c.get(&format!("{}/{}", base, id)).await?;
            c.print_value(&v);
        }
        Commands::Create {
            entity,
            json,
            fields,
        } => {
            let base = entity_api_path(&entity);
            let body = build_body(&json, &fields)?;
            let v = c.post(&base, &body).await?;
            print_mutation_result(&c, &v, "✅ Created");
        }
        Commands::Update {
            entity,
            id,
            json,
            fields,
        } => {
            let base = entity_api_path(&entity);
            let body = build_body(&json, &fields)?;
            let v = c.put(&format!("{}/{}", base, id), &body).await?;
            print_mutation_result(&c, &v, "✅ Updated");
        }
        Commands::Delete { entity, id } => {
            let base = entity_api_path(&entity);
            c.delete_req(&format!("{}/{}", base, id)).await?;
            print_delete_result(&c, &id, &format!("✅ Deleted {}/{}", entity, id));
        }
        Commands::Export { entity } => {
            let v = c.get(&format!("/api/v1/export/{}", entity)).await?;
            // Export returns CSV text or JSON
            if let Some(csv) = v.as_str() {
                println!("{}", csv);
            } else {
                c.print_value(&v);
            }
        }
    }

    Ok(())
}

/// Handle CrudAction for sub-resources (e.g., policy applications)
async fn handle_crud(
    c: &CliClient,
    base: &str,
    action: CrudAction,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        CrudAction::List { limit } => {
            let v = c.get(&format!("{}?limit={}", base, limit)).await?;
            print_list_response(c, &v);
        }
        CrudAction::Get { id } => {
            let v = c.get(&format!("{}/{}", base, id)).await?;
            c.print_value(&v);
        }
        CrudAction::Create { json, fields } => {
            let body = build_body(&json, &fields)?;
            let v = c.post(base, &body).await?;
            print_mutation_result(c, &v, "✅ Created");
        }
        CrudAction::Update { id, json, fields } => {
            let body = build_body(&json, &fields)?;
            let v = c.put(&format!("{}/{}", base, id), &body).await?;
            print_mutation_result(c, &v, "✅ Updated");
        }
        CrudAction::Delete { id } => {
            c.delete_req(&format!("{}/{}", base, id)).await?;
            print_delete_result(c, &id, "✅ Deleted");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::helpers::{build_body, entity_api_path, parse_kv};

    #[test]
    fn parse_kv_rejects_empty_key() {
        assert!(parse_kv("=value").is_err());
    }

    #[test]
    fn build_body_prefers_json_payload() {
        let body = build_body(&Some("{\"name\":\"Acme\"}".into()), &[]).expect("json body");
        assert_eq!(body["name"], "Acme");
    }

    #[test]
    fn entity_api_path_prefers_known_api_roots() {
        assert_eq!(entity_api_path("invoices"), "/api/invoices");
        assert_eq!(entity_api_path("policies"), "/api/policies");
        assert_eq!(entity_api_path("applications"), "/api/v1/applications");
    }
}
