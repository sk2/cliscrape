use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct CoverageReport {
    pub percentage: f64,
    pub captured_fields: Vec<String>,
    pub missing_fields: Vec<String>,
    pub total_expected: usize,
}

pub fn calculate_coverage(
    parsed_record: &HashMap<String, Value>,
    template_fields: &[String],
) -> CoverageReport {
    let captured: Vec<String> = parsed_record.keys().cloned().collect();
    let missing: Vec<String> = template_fields
        .iter()
        .filter(|field| !parsed_record.contains_key(*field))
        .cloned()
        .collect();

    let percentage = if template_fields.is_empty() {
        100.0  // Edge case: no expected fields
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_coverage_full() {
        let mut record = HashMap::new();
        record.insert("version".to_string(), Value::String("15.0".to_string()));
        record.insert("hostname".to_string(), Value::String("router1".to_string()));

        let fields = vec!["version".to_string(), "hostname".to_string()];
        let report = calculate_coverage(&record, &fields);

        assert_eq!(report.percentage, 100.0);
        assert_eq!(report.missing_fields.len(), 0);
    }

    #[test]
    fn test_calculate_coverage_partial() {
        let mut record = HashMap::new();
        record.insert("version".to_string(), Value::String("15.0".to_string()));

        let fields = vec!["version".to_string(), "hostname".to_string(), "serial".to_string()];
        let report = calculate_coverage(&record, &fields);

        assert!((report.percentage - 33.33).abs() < 0.1);
        assert_eq!(report.missing_fields.len(), 2);
        assert!(report.missing_fields.contains(&"hostname".to_string()));
        assert!(report.missing_fields.contains(&"serial".to_string()));
    }

    #[test]
    fn test_calculate_coverage_empty_template() {
        let record = HashMap::new();
        let fields: Vec<String> = vec![];
        let report = calculate_coverage(&record, &fields);

        assert_eq!(report.percentage, 100.0);  // Edge case
    }
}
