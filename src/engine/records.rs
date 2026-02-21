use crate::engine::convert::convert_scalar;
use crate::engine::types::Value;
use serde_json;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct RecordBuffer {
    buffer: HashMap<String, Vec<String>>,
    dirty: bool,
}

impl RecordBuffer {
    pub fn new() -> Self {
        Self {
            buffer: HashMap::new(),
            dirty: false,
        }
    }

    pub fn insert(&mut self, name: String, value: String, is_list: bool) {
        if is_list {
            self.buffer.entry(name).or_default().push(value);
        } else {
            self.buffer.insert(name, vec![value]);
        }
        self.dirty = true;
    }

    pub fn clear(&mut self) {
        self.clear_all();
    }

    pub fn clear_all(&mut self) {
        self.buffer.clear();
        self.dirty = false;
    }

    /// TextFSM `Clear` semantics: clear only non-Filldown values.
    pub fn clear_non_filldown(&mut self, values: &HashMap<String, Value>) {
        self.reset_after_emit(values);
    }

    /// Validates and extracts the record.
    /// If valid, returns the record and updates the buffer based on filldown rules.
    pub fn emit(
        &mut self,
        values: &HashMap<String, Value>,
    ) -> Option<HashMap<String, serde_json::Value>> {
        if !self.dirty {
            return None;
        }

        // Check required fields
        for (name, val) in values {
            if val.required {
                if !self.buffer.contains_key(name) || self.buffer[name].is_empty() {
                    // Required field missing, drop record
                    self.reset_after_emit(values);
                    return None;
                }
            }
        }

        let mut record = HashMap::new();
        for (name, val_def) in values {
            if let Some(vals) = self.buffer.get(name) {
                if val_def.list {
                    record.insert(
                        name.clone(),
                        serde_json::Value::Array(
                            vals.iter()
                                .map(|s| convert_scalar(s, val_def.type_hint))
                                .collect(),
                        ),
                    );
                } else {
                    // Should only have one value if it's not a list, but we take the last one just in case
                    if let Some(v) = vals.last() {
                        record.insert(name.clone(), convert_scalar(v, val_def.type_hint));
                    }
                }
            } else {
                // If it's a list, we might want an empty array instead of missing key?
                // TextFSM usually returns empty string for missing non-list values.
                if val_def.list {
                    record.insert(name.clone(), serde_json::Value::Array(vec![]));
                } else {
                    record.insert(name.clone(), serde_json::Value::String("".to_string()));
                }
            }
        }

        self.reset_after_emit(values);
        Some(record)
    }

    fn reset_after_emit(&mut self, values: &HashMap<String, Value>) {
        let mut next_buffer = HashMap::new();
        for (name, val) in values {
            if val.filldown {
                if let Some(v) = self.buffer.get(name) {
                    next_buffer.insert(name.clone(), v.clone());
                }
            }
        }
        self.buffer = next_buffer;
        self.dirty = false;
    }

    pub fn get_buffer(&self) -> &HashMap<String, Vec<String>> {
        &self.buffer
    }

    /// Returns the current variable values as a snapshot for trace events.
    /// Converts raw string buffers to typed JSON values based on value definitions.
    pub fn current_values(
        &self,
        values: &HashMap<String, Value>,
    ) -> HashMap<String, serde_json::Value> {
        let mut snapshot = HashMap::new();
        for (name, val_def) in values {
            if let Some(vals) = self.buffer.get(name) {
                if val_def.list {
                    snapshot.insert(
                        name.clone(),
                        serde_json::Value::Array(
                            vals.iter()
                                .map(|s| convert_scalar(s, val_def.type_hint))
                                .collect(),
                        ),
                    );
                } else if let Some(v) = vals.last() {
                    snapshot.insert(name.clone(), convert_scalar(v, val_def.type_hint));
                }
            }
        }
        snapshot
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::types::FieldType;

    #[test]
    fn test_list_accumulation() {
        let mut rb = RecordBuffer::new();
        let mut values = HashMap::new();
        values.insert(
            "Interfaces".to_string(),
            Value {
                name: "Interfaces".to_string(),
                regex: r#"\S+"#.to_string(),
                filldown: false,
                required: false,
                list: true,
                type_hint: None,
            },
        );

        rb.insert("Interfaces".to_string(), "Eth1".to_string(), true);
        rb.insert("Interfaces".to_string(), "Eth2".to_string(), true);

        let record = rb.emit(&values).unwrap();
        let interfaces = record.get("Interfaces").unwrap();
        assert!(interfaces.is_array());
        let arr = interfaces.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], "Eth1");
        assert_eq!(arr[1], "Eth2");
    }

    #[test]
    fn test_typed_int_conversion_emits_number() {
        let mut rb = RecordBuffer::new();

        let mut values = HashMap::new();
        values.insert(
            "Count".to_string(),
            Value {
                name: "Count".to_string(),
                regex: r#"\S+"#.to_string(),
                filldown: false,
                required: false,
                list: false,
                type_hint: Some(FieldType::Int),
            },
        );

        rb.insert("Count".to_string(), "1,234".to_string(), false);
        let record = rb.emit(&values).unwrap();

        assert_eq!(
            record["Count"],
            serde_json::Value::Number(serde_json::Number::from(1234_i64))
        );
    }

    #[test]
    fn test_failed_typed_int_conversion_falls_back_to_string() {
        let mut rb = RecordBuffer::new();

        let mut values = HashMap::new();
        values.insert(
            "Count".to_string(),
            Value {
                name: "Count".to_string(),
                regex: r#"\S+"#.to_string(),
                filldown: false,
                required: false,
                list: false,
                type_hint: Some(FieldType::Int),
            },
        );

        rb.insert("Count".to_string(), "12x".to_string(), false);
        let record = rb.emit(&values).unwrap();

        assert_eq!(
            record["Count"],
            serde_json::Value::String("12x".to_string())
        );
    }

    #[test]
    fn test_string_hint_preserves_raw_value() {
        let mut rb = RecordBuffer::new();

        let mut values = HashMap::new();
        values.insert(
            "Raw".to_string(),
            Value {
                name: "Raw".to_string(),
                regex: r#"\S+"#.to_string(),
                filldown: false,
                required: false,
                list: false,
                type_hint: Some(FieldType::String),
            },
        );

        rb.insert("Raw".to_string(), "1,234".to_string(), false);
        let record = rb.emit(&values).unwrap();

        assert_eq!(
            record["Raw"],
            serde_json::Value::String("1,234".to_string())
        );
    }

    #[test]
    fn test_typed_int_list_conversion_emits_number_array() {
        let mut rb = RecordBuffer::new();

        let mut values = HashMap::new();
        values.insert(
            "Counts".to_string(),
            Value {
                name: "Counts".to_string(),
                regex: r#"\S+"#.to_string(),
                filldown: false,
                required: false,
                list: true,
                type_hint: Some(FieldType::Int),
            },
        );

        rb.insert("Counts".to_string(), "1".to_string(), true);
        rb.insert("Counts".to_string(), "2,000".to_string(), true);
        let record = rb.emit(&values).unwrap();

        let arr = record["Counts"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(
            arr[0],
            serde_json::Value::Number(serde_json::Number::from(1_i64))
        );
        assert_eq!(
            arr[1],
            serde_json::Value::Number(serde_json::Number::from(2000_i64))
        );
    }

    #[test]
    fn test_heuristic_numeric_conversion_for_untyped_field() {
        let mut rb = RecordBuffer::new();

        let mut values = HashMap::new();
        values.insert(
            "MaybeNum".to_string(),
            Value {
                name: "MaybeNum".to_string(),
                regex: r#"\S+"#.to_string(),
                filldown: false,
                required: false,
                list: false,
                type_hint: None,
            },
        );

        rb.insert("MaybeNum".to_string(), "+1_234".to_string(), false);
        let record = rb.emit(&values).unwrap();

        assert_eq!(
            record["MaybeNum"],
            serde_json::Value::Number(serde_json::Number::from(1234_i64))
        );
    }
}
