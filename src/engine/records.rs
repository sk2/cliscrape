use std::collections::HashMap;
use crate::engine::types::Value;

#[derive(Debug, Default)]
pub struct RecordBuffer {
    buffer: HashMap<String, String>,
    dirty: bool,
}

impl RecordBuffer {
    pub fn new() -> Self {
        Self {
            buffer: HashMap::new(),
            dirty: false,
        }
    }

    pub fn insert(&mut self, name: String, value: String) {
        self.buffer.insert(name, value);
        self.dirty = true;
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.dirty = false;
    }

    /// Validates and extracts the record.
    /// If valid, returns the record and updates the buffer based on filldown rules.
    pub fn emit(&mut self, values: &HashMap<String, Value>) -> Option<HashMap<String, String>> {
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
        self.dirty = false;
    }

    pub fn get_buffer(&self) -> &HashMap<String, String> {
        &self.buffer
    }
}
