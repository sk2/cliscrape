use crate::engine::types::FieldType;

pub fn convert_scalar(raw: &str, hint: Option<FieldType>) -> serde_json::Value {
    match hint {
        Some(FieldType::String) => serde_json::Value::String(raw.to_string()),
        Some(FieldType::Int) => {
            convert_int(raw).unwrap_or_else(|| serde_json::Value::String(raw.to_string()))
        }
        None => convert_int(raw).unwrap_or_else(|| serde_json::Value::String(raw.to_string())),
    }
}

fn convert_int(raw: &str) -> Option<serde_json::Value> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut s = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        match ch {
            ',' | '_' => {}
            _ => s.push(ch),
        }
    }

    let bytes = s.as_bytes();
    let mut idx = 0;
    if matches!(bytes.first(), Some(b'+') | Some(b'-')) {
        idx = 1;
    }
    if idx >= bytes.len() {
        return None;
    }
    if !bytes[idx..].iter().all(|b| b.is_ascii_digit()) {
        return None;
    }

    let parsed: i64 = s.parse().ok()?;
    Some(serde_json::Value::Number(serde_json::Number::from(parsed)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_lenient_parsing_strips_separators() {
        assert_eq!(
            convert_scalar("1,234_567", Some(FieldType::Int)),
            serde_json::Value::Number(serde_json::Number::from(1234567_i64))
        );
    }

    #[test]
    fn test_string_hint_disables_numeric_heuristic() {
        assert_eq!(
            convert_scalar("1,234", Some(FieldType::String)),
            serde_json::Value::String("1,234".to_string())
        );
    }
}
