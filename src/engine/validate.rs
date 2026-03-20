use crate::engine::types::FieldConstraints;
use crate::TemplateWarning;
use serde_json::Value;

pub fn validate_value(
    field_name: &str,
    value: &Value,
    constraints: &FieldConstraints,
) -> Vec<TemplateWarning> {
    let mut warnings = Vec::new();

    // Min/Max validation
    if let Some(number) = value.as_f64() {
        if let Some(min) = constraints.min {
            if number < min {
                warnings.push(TemplateWarning {
                    kind: "ConstraintViolation".to_string(),
                    message: format!(
                        "Field '{}' value {} is less than minimum {}",
                        field_name, number, min
                    ),
                    line_idx: None,
                });
            }
        }
        if let Some(max) = constraints.max {
            if number > max {
                warnings.push(TemplateWarning {
                    kind: "ConstraintViolation".to_string(),
                    message: format!(
                        "Field '{}' value {} is greater than maximum {}",
                        field_name, number, max
                    ),
                    line_idx: None,
                });
            }
        }
    }

    // Choices validation
    if let Some(allowed) = &constraints.choices {
        if let Some(s) = value.as_str() {
            if !allowed.contains(&s.to_string()) {
                warnings.push(TemplateWarning {
                    kind: "ConstraintViolation".to_string(),
                    message: format!(
                        "Field '{}' value '{}' is not in allowed choices {:?}",
                        field_name, s, allowed
                    ),
                    line_idx: None,
                });
            }
        }
    }

    // Regex validation
    if let Some(re_str) = &constraints.regex {
        if let Some(s) = value.as_str() {
            match regex::Regex::new(re_str) {
                Ok(re) => {
                    if !re.is_match(s) {
                        warnings.push(TemplateWarning {
                            kind: "ConstraintViolation".to_string(),
                            message: format!(
                                "Field '{}' value '{}' does not match constraint regex '{}'",
                                field_name, s, re_str
                            ),
                            line_idx: None,
                        });
                    }
                }
                Err(e) => warnings.push(TemplateWarning {
                    kind: "InvalidConstraint".to_string(),
                    message: format!(
                        "Field '{}' has invalid constraint regex '{}': {}",
                        field_name, re_str, e
                    ),
                    line_idx: None,
                }),
            }
        }
    }

    warnings
}
