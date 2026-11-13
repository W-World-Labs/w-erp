//! OCR quality pipeline helpers (issue #14).
//!
//! - **Format-route first**: structured direct-read for China e-invoice XML / OFD /
//!   PDF text-layer via `w-einvoice` (no vision call when fields are recoverable).
//! - **Vision path helpers**: tolerant model-JSON parsing + invoice field validation.

use axum::http::StatusCode;
use serde_json::{Map, Value, json};
use w_einvoice::{
    DetectedFormat, EInvoice, PartialEInvoice, SourceFormat, detect_format, extract_from_pdf_text,
    parse_ofd, parse_xml,
};

/// Decode a `data:` URL or raw base64 payload into (mime_hint, bytes).
pub fn decode_upload_payload(file_base64: &str) -> Result<(String, Vec<u8>), StatusCode> {
    use base64::Engine;
    let (mime, b64) = if let Some(rest) = file_base64.strip_prefix("data:") {
        let (meta, data) = rest.split_once(',').ok_or(StatusCode::BAD_REQUEST)?;
        let mime = meta.split(';').next().unwrap_or("application/octet-stream");
        (mime.to_string(), data)
    } else {
        // Legacy clients (CLI/tests) send bare base64 for images without a data: URL.
        ("image/png".to_string(), file_base64)
    };
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(b64.as_bytes())
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok((mime, decoded))
}

/// Result of a successful structured direct-read (no vision).
#[derive(Debug, Clone)]
pub struct StructuredParseOutcome {
    pub fields: Value,
    pub source_format: String,
    pub warnings: Vec<String>,
}

/// Attempt structured e-invoice parse for XML / OFD / PDF text.
///
/// Returns `None` when the bytes are not a supported structured format or
/// when extraction fails — callers fall back to vision OCR.
pub fn try_structured_invoice_parse(
    bytes: &[u8],
    filename: Option<&str>,
) -> Option<StructuredParseOutcome> {
    if bytes.is_empty() {
        return None;
    }
    let fmt = detect_format(bytes, filename);
    match fmt {
        DetectedFormat::Xml => {
            let inv = parse_xml(bytes).ok()?;
            Some(einvoice_to_outcome(inv))
        }
        DetectedFormat::Ofd => {
            let inv = parse_ofd(bytes).ok()?;
            Some(einvoice_to_outcome(inv))
        }
        DetectedFormat::Pdf => try_pdf_structured(bytes),
        DetectedFormat::Unknown => {
            // Filename / content sniff fallback for plain text dumps of PDF layout.
            let name = filename.unwrap_or("").to_ascii_lowercase();
            if name.ends_with(".xml") || looks_like_xml(bytes) {
                let inv = parse_xml(bytes).ok()?;
                return Some(einvoice_to_outcome(inv));
            }
            if name.ends_with(".ofd") {
                let inv = parse_ofd(bytes).ok()?;
                return Some(einvoice_to_outcome(inv));
            }
            if name.ends_with(".txt") || name.ends_with(".pdf.txt") {
                return partial_to_outcome(extract_from_pdf_text(&String::from_utf8_lossy(bytes)));
            }
            None
        }
    }
}

fn try_pdf_structured(bytes: &[u8]) -> Option<StructuredParseOutcome> {
    // Pure-Rust text layer extract; image-only scans return empty and fall through to OCR.
    let text = pdf_extract::extract_text_from_mem(bytes).ok()?;
    if text.trim().is_empty() {
        return None;
    }
    partial_to_outcome(extract_from_pdf_text(&text))
}

fn looks_like_xml(bytes: &[u8]) -> bool {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(256)]);
    let t = head.trim_start_matches('\u{feff}').trim_start();
    t.starts_with("<?xml") || t.starts_with('<')
}

fn einvoice_to_outcome(inv: EInvoice) -> StructuredParseOutcome {
    let mut warnings = inv.warnings.clone();
    let source_format = source_format_label(inv.source);
    let mut fields = einvoice_to_field_map(&inv);
    warnings.extend(validate_invoice_field_map(&mut fields));
    StructuredParseOutcome {
        fields: Value::Object(fields),
        source_format,
        warnings,
    }
}

fn partial_has_any_field(p: &PartialEInvoice) -> bool {
    p.invoice_number.is_some()
        || p.invoice_date.is_some()
        || p.buyer_name.is_some()
        || p.buyer_tax_id.is_some()
        || p.seller_name.is_some()
        || p.seller_tax_id.is_some()
        || p.amount.is_some()
        || p.tax_amount.is_some()
        || p.total_amount.is_some()
        || !p.items.is_empty()
}

fn partial_to_outcome(
    partial: Result<PartialEInvoice, w_einvoice::Error>,
) -> Option<StructuredParseOutcome> {
    let partial = partial.ok()?;
    let source = partial.source?;
    if !partial_has_any_field(&partial) {
        return None;
    }
    // Promote to full EInvoice when invoice number is present (mirrors crate private into_full).
    if let Some(number) = partial
        .invoice_number
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        let inv = EInvoice {
            invoice_number: number,
            invoice_date: partial.invoice_date,
            buyer_name: partial.buyer_name,
            buyer_tax_id: partial.buyer_tax_id,
            seller_name: partial.seller_name,
            seller_tax_id: partial.seller_tax_id,
            amount: partial.amount,
            tax_amount: partial.tax_amount,
            total_amount: partial.total_amount,
            items: partial.items,
            source,
            warnings: partial.warnings,
        };
        return Some(einvoice_to_outcome(inv));
    }
    let mut warnings = partial.warnings.clone();
    warnings.push("partial_extract: missing invoice_number".to_string());
    let mut fields = partial_to_field_map(&partial);
    warnings.extend(validate_invoice_field_map(&mut fields));
    if fields.is_empty() {
        return None;
    }
    Some(StructuredParseOutcome {
        fields: Value::Object(fields),
        source_format: source_format_label(source),
        warnings,
    })
}

fn source_format_label(s: SourceFormat) -> String {
    match s {
        SourceFormat::Xml => "xml".into(),
        SourceFormat::OfdAttachment => "ofd_attachment".into(),
        SourceFormat::OfdPageText => "ofd_page_text".into(),
        SourceFormat::PdfTextRules => "pdf_text_rules".into(),
    }
}

fn opt_string(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn amount_string(v: Option<f64>) -> Option<String> {
    v.map(|n| {
        // Prefer compact decimal without trailing zeros for money UI.
        let s = format!("{n:.2}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    })
}

fn date_string(d: Option<chrono::NaiveDate>) -> Option<String> {
    d.map(|d| d.format("%Y-%m-%d").to_string())
}

/// Map `w-einvoice` result onto ERP invoice dialog field names.
pub fn einvoice_to_field_map(inv: &EInvoice) -> Map<String, Value> {
    let mut m = Map::new();
    insert_opt(&mut m, "invoice_number", Some(inv.invoice_number.clone()));
    insert_opt(&mut m, "invoice_date", date_string(inv.invoice_date));
    insert_opt(&mut m, "amount", amount_string(inv.amount));
    insert_opt(&mut m, "tax_amount", amount_string(inv.tax_amount));
    insert_opt(&mut m, "total_amount", amount_string(inv.total_amount));
    insert_opt(&mut m, "supplier", opt_string(inv.seller_name.clone()));
    insert_opt(&mut m, "buyer", opt_string(inv.buyer_name.clone()));
    insert_opt(
        &mut m,
        "seller_tax_id",
        opt_string(inv.seller_tax_id.clone()),
    );
    insert_opt(&mut m, "buyer_tax_id", opt_string(inv.buyer_tax_id.clone()));
    let desc = inv
        .items
        .first()
        .and_then(|i| i.name.clone())
        .and_then(|s| {
            let t = s.trim().to_string();
            if t.is_empty() { None } else { Some(t) }
        });
    insert_opt(&mut m, "description", desc);
    // Default structured tax invoices as input (payable/进项); UI can flip for receivable.
    m.insert("invoice_type".into(), json!("input"));
    m
}

fn partial_to_field_map(p: &PartialEInvoice) -> Map<String, Value> {
    let mut m = Map::new();
    insert_opt(
        &mut m,
        "invoice_number",
        opt_string(p.invoice_number.clone()),
    );
    insert_opt(&mut m, "invoice_date", date_string(p.invoice_date));
    insert_opt(&mut m, "amount", amount_string(p.amount));
    insert_opt(&mut m, "tax_amount", amount_string(p.tax_amount));
    insert_opt(&mut m, "total_amount", amount_string(p.total_amount));
    insert_opt(&mut m, "supplier", opt_string(p.seller_name.clone()));
    insert_opt(&mut m, "buyer", opt_string(p.buyer_name.clone()));
    m
}

fn insert_opt(m: &mut Map<String, Value>, key: &str, value: Option<String>) {
    if let Some(v) = value {
        m.insert(key.to_string(), json!(v));
    }
}

/// Strip markdown fences, recover first JSON object, and tolerate trailing commas.
pub fn parse_model_json_tolerant(raw: &str) -> Result<Value, String> {
    let cleaned = extract_json_content(raw);
    if let Ok(v) = serde_json::from_str::<Value>(&cleaned) {
        return Ok(v);
    }
    let fixed = strip_trailing_commas(&cleaned);
    serde_json::from_str::<Value>(&fixed).map_err(|e| format!("json parse failed: {e}"))
}

/// Strip markdown fences / extract first JSON object (vision models).
pub fn extract_json_content(raw: &str) -> String {
    let trimmed = raw.trim();
    let without_fence = if trimmed.starts_with("```") {
        let rest = trimmed.trim_start_matches('`');
        let rest = rest
            .strip_prefix("json")
            .or_else(|| rest.strip_prefix("JSON"))
            .unwrap_or(rest);
        let rest = rest.trim_start_matches('\n');
        if let Some(end) = rest.rfind("```") {
            rest[..end].trim()
        } else {
            rest.trim()
        }
    } else {
        trimmed
    };
    if serde_json::from_str::<Value>(without_fence).is_ok() {
        return without_fence.to_string();
    }
    if let Some(start) = without_fence.find('{')
        && let Some(end) = without_fence.rfind('}')
        && end > start
    {
        let slice = &without_fence[start..=end];
        if serde_json::from_str::<Value>(slice).is_ok()
            || serde_json::from_str::<Value>(&strip_trailing_commas(slice)).is_ok()
        {
            return slice.to_string();
        }
        return slice.to_string();
    }
    without_fence.to_string()
}

/// Remove trailing commas before `}` / `]` (common LLM slip).
fn strip_trailing_commas(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == ',' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                // skip comma
                i += 1;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Cross-check numeric invoice fields; mutates nothing critical, returns warnings.
///
/// Checks: parseable amounts, plausible date already handled upstream,
/// `total ≈ amount + tax` within 0.05.
pub fn validate_invoice_field_map(fields: &mut Map<String, Value>) -> Vec<String> {
    let mut warnings = Vec::new();
    let amount = parse_field_f64(fields.get("amount"));
    let tax = parse_field_f64(fields.get("tax_amount"));
    let total = parse_field_f64(fields.get("total_amount"));

    if fields.get("amount").is_some() && amount.is_none() {
        warnings.push("amount is not a valid number".into());
    }
    if fields.get("tax_amount").is_some() && tax.is_none() {
        warnings.push("tax_amount is not a valid number".into());
    }
    if fields.get("total_amount").is_some() && total.is_none() {
        warnings.push("total_amount is not a valid number".into());
    }

    if let (Some(a), Some(t), Some(tot)) = (amount, tax, total) {
        let expected = a + t;
        if (expected - tot).abs() > 0.05 {
            warnings.push(format!(
                "total_amount ({tot}) != amount+tax ({expected:.2})"
            ));
        }
    } else if let (Some(a), Some(t)) = (amount, tax) {
        // Fill total when missing and both parts present.
        if total.is_none() {
            let filled = format!("{:.2}", a + t);
            let filled = filled
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            fields.insert("total_amount".into(), json!(filled));
        }
    } else if let (Some(tot), Some(t)) = (total, tax) {
        if amount.is_none() {
            let filled = format!("{:.2}", tot - t);
            let filled = filled
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            fields.insert("amount".into(), json!(filled));
            warnings.push("amount inferred from total_amount - tax_amount".into());
        }
    }

    if let Some(Value::String(d)) = fields.get("invoice_date") {
        if chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").is_err() {
            warnings.push(format!("invoice_date not ISO date: {d}"));
        }
    }

    // Surface confidence heuristic: structured-complete vs partial.
    let confidence = invoice_field_confidence(fields, &warnings);
    fields.insert("field_confidence".into(), json!(confidence));
    if !warnings.is_empty() {
        fields.insert("warnings".into(), json!(warnings.clone()));
    }
    warnings
}

fn invoice_field_confidence(fields: &Map<String, Value>, warnings: &[String]) -> f64 {
    let keys = [
        "invoice_number",
        "invoice_date",
        "amount",
        "tax_amount",
        "total_amount",
        "supplier",
        "buyer",
    ];
    let present = keys
        .iter()
        .filter(|k| {
            fields
                .get(**k)
                .and_then(|v| v.as_str())
                .is_some_and(|s| !s.is_empty())
        })
        .count();
    let mut score = present as f64 / keys.len() as f64;
    if warnings.iter().any(|w| w.contains("total_amount")) {
        score *= 0.85;
    }
    (score * 100.0).round() / 100.0
}

fn parse_field_f64(v: Option<&Value>) -> Option<f64> {
    match v {
        Some(Value::Number(n)) => n.as_f64(),
        Some(Value::String(s)) => parse_money_string(s),
        _ => None,
    }
}

/// Local money parse (w-einvoice's amount helper is crate-private in 0.1.0).
fn parse_money_string(raw: &str) -> Option<f64> {
    let s = raw
        .trim()
        .trim_start_matches(['¥', '￥', 'R', 'M', 'B'])
        .trim_start_matches("CNY")
        .trim_start_matches("RMB")
        .trim_end_matches(['元', '圆'])
        .trim()
        .replace(',', "")
        .replace('，', "");
    if s.is_empty() {
        return None;
    }
    s.parse::<f64>().ok()
}

/// Build the success JSON envelope for `/api/ai/parse-document`.
pub fn parse_success_response(
    document_type: &str,
    fields: Value,
    source: &str,
    warnings: &[String],
) -> Value {
    let mut body = json!({
        "fields": fields,
        "document_type": document_type,
        "success": true,
        "parse_source": source,
    });
    if !warnings.is_empty() {
        body["warnings"] = json!(warnings);
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> Vec<u8> {
        let path = format!(
            "{}/tests/fixtures/einvoice/{}",
            env!("CARGO_MANIFEST_DIR"),
            name
        );
        std::fs::read(&path).unwrap_or_else(|e| panic!("read {path}: {e}"))
    }

    #[test]
    fn structured_xml_fixture_parses() {
        let bytes = fixture("sample_utf8.xml");
        let out =
            try_structured_invoice_parse(&bytes, Some("sample_utf8.xml")).expect("xml direct-read");
        assert_eq!(out.source_format, "xml");
        assert_eq!(out.fields["invoice_number"], "25332000000012345678");
        assert_eq!(out.fields["amount"], "100");
        assert_eq!(out.fields["tax_amount"], "13");
        assert_eq!(out.fields["total_amount"], "113");
        assert_eq!(out.fields["supplier"], "测试销方科技有限公司");
        assert_eq!(out.fields["buyer"], "测试购方有限公司");
        assert!(out.fields.get("field_confidence").is_some());
    }

    #[test]
    fn structured_ofd_with_attachment_parses() {
        let bytes = fixture("sample_ofd_with_attach.ofd");
        let out =
            try_structured_invoice_parse(&bytes, Some("sample.ofd")).expect("ofd direct-read");
        assert!(
            out.source_format == "ofd_attachment" || out.source_format == "ofd_page_text",
            "source={}",
            out.source_format
        );
        assert!(
            out.fields
                .get("invoice_number")
                .and_then(|v| v.as_str())
                .is_some_and(|s| !s.is_empty()),
            "fields={:?}",
            out.fields
        );
    }

    #[test]
    fn structured_pdf_text_rules_parse() {
        let bytes = fixture("sample_pdf_text.txt");
        let out = try_structured_invoice_parse(&bytes, Some("sample_pdf_text.txt"))
            .expect("pdf text rules");
        assert_eq!(out.source_format, "pdf_text_rules");
        assert!(out.fields.get("invoice_number").is_some());
    }

    #[test]
    fn extract_json_strips_fences_and_trailing_commas() {
        let raw = "```json\n{\"a\":1, \"b\":2,}\n```";
        let v = parse_model_json_tolerant(raw).unwrap();
        assert_eq!(v["a"], 1);
        assert_eq!(v["b"], 2);
    }

    #[test]
    fn extract_json_from_prose() {
        let raw = "识别结果如下：\n{\"invoice_number\":\"X1\",\"amount\":\"10\",}\n谢谢";
        let v = parse_model_json_tolerant(raw).unwrap();
        assert_eq!(v["invoice_number"], "X1");
    }

    #[test]
    fn validate_flags_total_mismatch() {
        let mut m = Map::new();
        m.insert("amount".into(), json!("100"));
        m.insert("tax_amount".into(), json!("13"));
        m.insert("total_amount".into(), json!("200"));
        let warnings = validate_invoice_field_map(&mut m);
        assert!(
            warnings.iter().any(|w| w.contains("total_amount")),
            "{warnings:?}"
        );
    }

    #[test]
    fn validate_fills_missing_total() {
        let mut m = Map::new();
        m.insert("amount".into(), json!("100"));
        m.insert("tax_amount".into(), json!("13"));
        let _ = validate_invoice_field_map(&mut m);
        assert_eq!(m.get("total_amount").and_then(|v| v.as_str()), Some("113"));
    }

    #[test]
    fn decode_data_url() {
        // "hi" base64
        let (mime, bytes) = decode_upload_payload("data:text/plain;base64,aGk=").unwrap();
        assert_eq!(mime, "text/plain");
        assert_eq!(bytes, b"hi");
    }
}
