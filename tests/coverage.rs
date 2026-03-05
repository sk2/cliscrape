pub use cliscrape::engine::coverage::calculate_coverage;
use serde_json::Value;
use std::collections::BTreeMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_coverage_full() {
        let mut record = BTreeMap::new();
        record.insert("version".to_string(), Value::String("15.0".to_string()));
        record.insert("hostname".to_string(), Value::String("router1".to_string()));

        let fields = vec!["version".to_string(), "hostname".to_string()];
        let report = calculate_coverage(&record, &fields);

        assert_eq!(report.percentage, 100.0);
        assert_eq!(report.missing_fields.len(), 0);
    }

    #[test]
    fn test_calculate_coverage_partial() {
        let mut record = BTreeMap::new();
        record.insert("version".to_string(), Value::String("15.0".to_string()));
        record.insert("empty".to_string(), Value::String("".to_string()));

        let fields = vec![
            "version".to_string(),
            "hostname".to_string(),
            "serial".to_string(),
            "empty".to_string(),
        ];
        let report = calculate_coverage(&record, &fields);

        // only "version" is captured. "empty" is ignored because it's empty string.
        assert_eq!(report.percentage, 25.0);
        assert_eq!(report.missing_fields.len(), 3);
        assert!(report.missing_fields.contains(&"hostname".to_string()));
        assert!(report.missing_fields.contains(&"serial".to_string()));
        assert!(report.missing_fields.contains(&"empty".to_string()));
    }

    #[test]
    fn test_calculate_coverage_empty_template() {
        let record = BTreeMap::new();
        let fields: Vec<String> = vec![];
        let report = calculate_coverage(&record, &fields);

        assert_eq!(report.percentage, 100.0); // Edge case
    }
}
