use std::collections::HashMap;
use crate::engine::types::Value;
use serde_json;

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
        self.buffer.clear();
        self.dirty = false;
    }

    /// Validates and extracts the record.
    /// If valid, returns the record and updates the buffer based on filldown rules.
    pub fn emit(&mut self, values: &HashMap<String, Value>) -> Option<HashMap<String, serde_json::Value>> {
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
                    record.insert(name.clone(), serde_json::Value::Array(
                        vals.iter().map(|s| serde_json::Value::String(s.clone())).collect()
                    ));
                } else {
                    // Should only have one value if it's not a list, but we take the last one just in case
                    if let Some(v) = vals.last() {
                        record.insert(name.clone(), serde_json::Value::String(v.clone()));
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_accumulation() {
        let mut rb = RecordBuffer::new();
        let mut values = HashMap::new();
        values.insert("Interfaces".to_string(), Value {
            name: "Interfaces".to_string(),
            regex: r#"\S+"#.to_string(),
            filldown: false,
            required: false,
            list: true,
        });

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
}

