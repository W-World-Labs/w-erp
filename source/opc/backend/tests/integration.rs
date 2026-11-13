mod common;

mod s01_company_setup {
    include!("s01_company_setup.rs");
}

mod s02_monthly_billing {
    include!("s02_monthly_billing.rs");
}

mod s03_employee_payroll {
    include!("s03_employee_payroll.rs");
}

mod s04_bank_cashflow {
    include!("s04_bank_cashflow.rs");
}

mod s05_budget_cycle {
    include!("s05_budget_cycle.rs");
}

mod s06_asset_lifecycle {
    include!("s06_asset_lifecycle.rs");
}

mod s07_project_applications {
    include!("s07_project_applications.rs");
}

mod s08_security_isolation {
    include!("s08_security_isolation.rs");
}

mod s09_policy_intelligence {
    include!("s09_policy_intelligence.rs");
}

mod s100_backend_owned_asset_filters {
    include!("s100_backend_owned_asset_filters.rs");
}

mod s101_backend_owned_tax_management_rows {
    include!("s101_backend_owned_tax_management_rows.rs");
}

mod s102_backend_owned_budget_display {
    include!("s102_backend_owned_budget_display.rs");
}

mod s103_backend_owned_rent_display {
    include!("s103_backend_owned_rent_display.rs");
}

mod s104_backend_owned_time_tracking_display {
    include!("s104_backend_owned_time_tracking_display.rs");
}

mod s105_backend_owned_reconciliation_overview {
    include!("s105_backend_owned_reconciliation_overview.rs");
}

mod s106_backend_owned_company_info_overview {
    include!("s106_backend_owned_company_info_overview.rs");
}

mod s107_backend_owned_salary_display {
    include!("s107_backend_owned_salary_display.rs");
}

mod s108_backend_owned_projects_display {
    include!("s108_backend_owned_projects_display.rs");
}

mod s109_backend_owned_invoices_display {
    include!("s109_backend_owned_invoices_display.rs");
}

mod s10_tax_compliance {
    include!("s10_tax_compliance.rs");
}

mod s110_backend_owned_financial_health_display {
    include!("s110_backend_owned_financial_health_display.rs");
}

mod s111_backend_owned_cloud_cost_display {
    include!("s111_backend_owned_cloud_cost_display.rs");
}

mod s112_backend_owned_recurring_display {
    include!("s112_backend_owned_recurring_display.rs");
}

mod s113_backend_owned_crm_display {
    include!("s113_backend_owned_crm_display.rs");
}

mod s114_backend_owned_payment_invoice_reminders {
    include!("s114_backend_owned_payment_invoice_reminders.rs");
}

mod s115_backend_owned_kpi_display {
    include!("s115_backend_owned_kpi_display.rs");
}

mod s116_backend_owned_workbench_display {
    include!("s116_backend_owned_workbench_display.rs");
}

mod s117_backend_owned_licenses_display {
    include!("s117_backend_owned_licenses_display.rs");
}

mod s118_backend_owned_policies_display {
    include!("s118_backend_owned_policies_display.rs");
}

mod s119_backend_owned_audit_center {
    include!("s119_backend_owned_audit_center.rs");
}

mod s11_cloud_cost_lifecycle {
    include!("s11_cloud_cost_lifecycle.rs");
}

mod s120_backend_owned_settings {
    include!("s120_backend_owned_settings.rs");
}

mod s121_backend_owned_system_status {
    include!("s121_backend_owned_system_status.rs");
}

mod s122_backend_owned_onboarding {
    include!("s122_backend_owned_onboarding.rs");
}

mod s123_backend_owned_completion_overview {
    include!("s123_backend_owned_completion_overview.rs");
}

mod s124_backend_owned_invoice_import {
    include!("s124_backend_owned_invoice_import.rs");
}

mod s125_backend_owned_form_inputs {
    include!("s125_backend_owned_form_inputs.rs");
}

mod s126_backend_owned_invoice_form {
    include!("s126_backend_owned_invoice_form.rs");
}

mod s127_backend_owned_dedicated_form_inputs {
    include!("s127_backend_owned_dedicated_form_inputs.rs");
}

mod s128_backend_owned_profile {
    include!("s128_backend_owned_profile.rs");
}

mod s129_backend_owned_entity_documents {
    include!("s129_backend_owned_entity_documents.rs");
}

mod s12_procurement_cycle {
    include!("s12_procurement_cycle.rs");
}

mod s130_backend_owned_attachments {
    include!("s130_backend_owned_attachments.rs");
}

mod s131_spa_static_fallback {
    include!("s131_spa_static_fallback.rs");
}

mod s132_agent_gateway {
    include!("s132_agent_gateway.rs");
}

mod s133_collection_automation {
    include!("s133_collection_automation.rs");
}

mod s133_openapi_contract {
    include!("s133_openapi_contract.rs");
}

mod s134_demo_company_seed {
    include!("s134_demo_company_seed.rs");
}

mod s135_demo_backfill {
    include!("s135_demo_backfill.rs");
}

mod s136_time_entry_tags {
    include!("s136_time_entry_tags.rs");
}

mod s137_file_upload_size_cap {
    include!("s137_file_upload_size_cap.rs");
}

mod s13_audit_and_reconciliation {
    include!("s13_audit_and_reconciliation.rs");
}

mod s14_dashboard_analytics {
    include!("s14_dashboard_analytics.rs");
}

mod s15_global_opc_tax {
    include!("s15_global_opc_tax.rs");
}

mod s16_mandatory_contract_review {
    include!("s16_mandatory_contract_review.rs");
}

mod s17_api_key_management {
    include!("s17_api_key_management.rs");
}

mod s18_company_member_management {
    include!("s18_company_member_management.rs");
}

mod s19_vendor_rent_license {
    include!("s19_vendor_rent_license.rs");
}

mod s20_company_settings {
    include!("s20_company_settings.rs");
}

mod s21_material_management {
    include!("s21_material_management.rs");
}

mod s22_data_assets_rd {
    include!("s22_data_assets_rd.rs");
}

mod s23_system_config {
    include!("s23_system_config.rs");
}

mod s24_crm_client_management {
    include!("s24_crm_client_management.rs");
}

mod s25_time_tracking {
    include!("s25_time_tracking.rs");
}

mod s26_recurring_invoices {
    include!("s26_recurring_invoices.rs");
}

mod s27_compliance {
    include!("s27_compliance.rs");
}

mod s28_tax_calendar {
    include!("s28_tax_calendar.rs");
}

mod s29_data_integrity {
    include!("s29_data_integrity.rs");
}

mod s30_email {
    include!("s30_email.rs");
}

mod s31_opc_seed {
    include!("s31_opc_seed.rs");
}

mod s32_boundary_values {
    include!("s32_boundary_values.rs");
}

mod s33_fk_and_cascade {
    include!("s33_fk_and_cascade.rs");
}

mod s34_multi_tenant {
    include!("s34_multi_tenant.rs");
}

mod s35_pagination_export {
    include!("s35_pagination_export.rs");
}

mod s36_amount_and_export {
    include!("s36_amount_and_export.rs");
}

mod s37_cross_module {
    include!("s37_cross_module.rs");
}

mod s38_operational_modules {
    include!("s38_operational_modules.rs");
}

mod s39_cloud_infra {
    include!("s39_cloud_infra.rs");
}

mod s40_admin_kpi_assets {
    include!("s40_admin_kpi_assets.rs");
}

mod s41_remaining_crud {
    include!("s41_remaining_crud.rs");
}

mod s42_special_endpoints {
    include!("s42_special_endpoints.rs");
}

mod s43_time_to_invoice {
    include!("s43_time_to_invoice.rs");
}

mod s44_ar_management {
    include!("s44_ar_management.rs");
}

mod s45_pdf_engine {
    include!("s45_pdf_engine.rs");
}

mod s46_reports_engine {
    include!("s46_reports_engine.rs");
}

mod s47_multi_currency {
    include!("s47_multi_currency.rs");
}

mod s48_asian_compliance {
    include!("s48_asian_compliance.rs");
}

mod s49_webhooks_security {
    include!("s49_webhooks_security.rs");
}

mod s50_system_endpoints {
    include!("s50_system_endpoints.rs");
}

mod s51_e2e_workflow {
    include!("s51_e2e_workflow.rs");
}

mod s52_error_logs {
    include!("s52_error_logs.rs");
}

mod s53_file_management {
    include!("s53_file_management.rs");
}

mod s54_edge_cases {
    include!("s54_edge_cases.rs");
}

mod s55_company_lifecycle {
    include!("s55_company_lifecycle.rs");
}

mod s56_ai_degradation {
    include!("s56_ai_degradation.rs");
}

mod s57_generic_crud_gaps {
    include!("s57_generic_crud_gaps.rs");
}

mod s58_dedicated_module_routes {
    include!("s58_dedicated_module_routes.rs");
}

mod s59_cli_crud_e2e {
    include!("s59_cli_crud_e2e.rs");
}

mod s60_cli_business_flow {
    include!("s60_cli_business_flow.rs");
}

mod s61_cli_tenant_isolation {
    include!("s61_cli_tenant_isolation.rs");
}

mod s62_file_upload_and_tools_e2e {
    include!("s62_file_upload_and_tools_e2e.rs");
}

mod s63_full_entity_coverage_e2e {
    include!("s63_full_entity_coverage_e2e.rs");
}

mod s64_cli_binary_e2e {
    include!("s64_cli_binary_e2e.rs");
}

mod s65_cli_api_golden_flow {
    include!("s65_cli_api_golden_flow.rs");
}

mod s66_workflow_usecases {
    include!("s66_workflow_usecases.rs");
}

mod s67_runtime_config {
    include!("s67_runtime_config.rs");
}

mod s68_backend_owned_workbench {
    include!("s68_backend_owned_workbench.rs");
}

mod s69_backend_owned_imports {
    include!("s69_backend_owned_imports.rs");
}

mod s70_backend_owned_kpi {
    include!("s70_backend_owned_kpi.rs");
}

mod s71_backend_owned_budgets {
    include!("s71_backend_owned_budgets.rs");
}

mod s72_backend_owned_tax_management {
    include!("s72_backend_owned_tax_management.rs");
}

mod s73_backend_owned_assets {
    include!("s73_backend_owned_assets.rs");
}

mod s74_backend_owned_rent {
    include!("s74_backend_owned_rent.rs");
}

mod s75_backend_owned_payments {
    include!("s75_backend_owned_payments.rs");
}

mod s76_backend_owned_time_tracking {
    include!("s76_backend_owned_time_tracking.rs");
}

mod s77_backend_owned_salary {
    include!("s77_backend_owned_salary.rs");
}

mod s78_backend_owned_vendors {
    include!("s78_backend_owned_vendors.rs");
}

mod s79_backend_owned_recurring_invoices {
    include!("s79_backend_owned_recurring_invoices.rs");
}

mod s80_backend_owned_invoices {
    include!("s80_backend_owned_invoices.rs");
}

mod s81_backend_owned_clients {
    include!("s81_backend_owned_clients.rs");
}

mod s82_backend_owned_cloud_cost {
    include!("s82_backend_owned_cloud_cost.rs");
}

mod s83_backend_owned_applications {
    include!("s83_backend_owned_applications.rs");
}

mod s84_backend_owned_tax_calendar {
    include!("s84_backend_owned_tax_calendar.rs");
}

mod s85_backend_owned_compliance {
    include!("s85_backend_owned_compliance.rs");
}

mod s86_backend_owned_salary_insurance {
    include!("s86_backend_owned_salary_insurance.rs");
}

mod s87_backend_owned_projects {
    include!("s87_backend_owned_projects.rs");
}

mod s88_backend_owned_deals {
    include!("s88_backend_owned_deals.rs");
}

mod s89_backend_owned_quotes {
    include!("s89_backend_owned_quotes.rs");
}

mod s90_backend_owned_contracts {
    include!("s90_backend_owned_contracts.rs");
}

mod s91_backend_owned_bank_accounts {
    include!("s91_backend_owned_bank_accounts.rs");
}

mod s92_backend_owned_asset_records {
    include!("s92_backend_owned_asset_records.rs");
}

mod s93_backend_owned_reminders {
    include!("s93_backend_owned_reminders.rs");
}

mod s94_backend_owned_operation_logs {
    include!("s94_backend_owned_operation_logs.rs");
}

mod s95_backend_owned_company_members {
    include!("s95_backend_owned_company_members.rs");
}

mod s96_backend_owned_vendor_filters {
    include!("s96_backend_owned_vendor_filters.rs");
}

mod s97_backend_owned_client_filters {
    include!("s97_backend_owned_client_filters.rs");
}

mod s98_backend_owned_employees {
    include!("s98_backend_owned_employees.rs");
}

mod s99_backend_owned_payment_filters {
    include!("s99_backend_owned_payment_filters.rs");
}

mod s138_health_error_logs_pdf {
    include!("s138_health_error_logs_pdf.rs");
}

mod s139_invoice_mail_import {
    include!("s139_invoice_mail_import.rs");
}

mod s130_document_parser_openai_vision {
    include!("s130_document_parser_openai_vision.rs");
}

mod s139_einvoice_structured_direct_read {
    include!("s139_einvoice_structured_direct_read.rs");
}

mod s140_monthly_close {
    include!("s140_monthly_close.rs");
}

mod s141_nav_preferences {
    include!("s141_nav_preferences.rs");
}

mod s131_notification_delivery {
    include!("s131_notification_delivery.rs");
}
