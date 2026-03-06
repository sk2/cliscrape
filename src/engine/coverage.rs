use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub percentage: f64,
    pub captured_fields: Vec<String>,
    pub missing_fields: Vec<String>,
    pub total_expected: usize,
}

pub fn calculate_coverage(
    parsed_record: &BTreeMap<String, Value>,
    template_fields: &[String],
) -> CoverageReport {
    let captured: Vec<String> = parsed_record
        .iter()
        .filter(|(_, v)| match v {
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            _ => true, // numbers etc are considered captured
        })
        .map(|(k, _)| k.clone())
        .collect();

    let missing: Vec<String> = template_fields
        .iter()
        .filter(|field| !captured.contains(*field))
        .cloned()
        .collect();

    let percentage = if template_fields.is_empty() {
        100.0 // Edge case: no expected fields
    } else {
        (captured.len() as f64 / template_fields.len() as f64) * 100.0
    };

    CoverageReport {
        percentage,
        captured_fields: captured,
        missing_fields: missing,
        total_expected: template_fields.len(),
    }
}
