use std::collections::HashMap;
use crate::engine::types::Value;

#[derive(Debug, Default)]
pub struct RecordBuffer {
    buffer: HashMap<String, String>,
}

impl RecordBuffer {
    pub fn new() -> Self {
        Self {
            buffer: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, value: String) {
        self.buffer.insert(name, value);
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Validates and extracts the record.
    /// If valid, returns the record and updates the buffer based on filldown rules.
    pub fn emit(&mut self, values: &HashMap<String, Value>) -> Option<HashMap<String, String>> {
        // Check required fields
        for (name, val) in values {
            if val.required {
                if !self.buffer.contains_key(name) || self.buffer[name].is_empty() {
                    // Required field missing, drop record
                    // Note: In TextFSM, if a required value is missing, the record is not added.
                    // We also clear non-filldown values even if record is dropped?
                    // TextFSM docs say: "If any of the Values with 'Required' or 'Required, Filldown' 
                    // are not assigned a value during the parsing of the current record, then the 
                    // record is not added to the results table."
                    self.reset_after_emit(values);
                    return None;
                }
            }
        }

        let record = self.buffer.clone();
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
    }

    pub fn get_buffer(&self) -> &HashMap<String, String> {
        &self.buffer
    }
}
