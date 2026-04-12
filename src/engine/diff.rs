use crate::engine::types::Value as TemplateValue;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum DiffOp {
    Added {
        after: BTreeMap<String, Value>,
    },
    Removed {
        before: BTreeMap<String, Value>,
    },
    Modified {
        identity: BTreeMap<String, Value>,
        changes: BTreeMap<String, ValueChange>,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ValueChange {
    pub before: Value,
    pub after: Value,
}

pub struct SemanticDiffEngine {
    identity_fields: Vec<String>,
    ignore_fields: Vec<String>,
}

impl SemanticDiffEngine {
    pub fn new(template_values: &HashMap<String, TemplateValue>) -> Self {
        let mut identity_fields = Vec::new();
        let mut ignore_fields = Vec::new();

        for (name, val) in template_values {
            if val.identity {
                identity_fields.push(name.clone());
            }
            if val.ignore {
                ignore_fields.push(name.clone());
            }
        }

        Self {
            identity_fields,
            ignore_fields,
        }
    }

    pub fn diff(
        &self,
        before: Vec<BTreeMap<String, Value>>,
        after: Vec<BTreeMap<String, Value>>,
    ) -> Vec<DiffOp> {
        let mut ops = Vec::new();

        let before_map = self.index_records(before);
        let after_map = self.index_records(after);

        // Check for removed and modified
        for (id, b_rec) in &before_map {
            match after_map.get(id) {
                Some(a_rec) => {
                    let changes = self.diff_record(b_rec, a_rec);
                    if !changes.is_empty() {
                        ops.push(DiffOp::Modified {
                            identity: id.clone(),
                            changes,
                        });
                    }
                }
                None => {
                    ops.push(DiffOp::Removed {
                        before: b_rec.clone(),
                    });
                }
            }
        }

        // Check for added
        for (id, a_rec) in &after_map {
            if !before_map.contains_key(id) {
                ops.push(DiffOp::Added {
                    after: a_rec.clone(),
                });
            }
        }

        ops
    }

    fn index_records(
        &self,
        records: Vec<BTreeMap<String, Value>>,
    ) -> HashMap<BTreeMap<String, Value>, BTreeMap<String, Value>> {
        let mut indexed = HashMap::new();
        for rec in records {
            let mut id = BTreeMap::new();
            for field in &self.identity_fields {
                if let Some(val) = rec.get(field) {
                    id.insert(field.clone(), val.clone());
                }
            }
            // If no identity fields, use the whole record (minus ignored) as identity
            if id.is_empty() {
                for (k, v) in &rec {
                    if !self.ignore_fields.contains(k) {
                        id.insert(k.clone(), v.clone());
                    }
                }
            }
            indexed.insert(id, rec);
        }
        indexed
    }

    fn diff_record(
        &self,
        before: &BTreeMap<String, Value>,
        after: &BTreeMap<String, Value>,
    ) -> BTreeMap<String, ValueChange> {
        let mut changes = BTreeMap::new();

        // Check all fields in both
        let mut all_keys = std::collections::HashSet::new();
        all_keys.extend(before.keys());
        all_keys.extend(after.keys());

        for key in all_keys {
            if self.ignore_fields.contains(key) {
                continue;
            }

            let b_val = before.get(key);
            let a_val = after.get(key);

            match (b_val, a_val) {
                (Some(b), Some(a)) if b != a => {
                    changes.insert(
                        key.clone(),
                        ValueChange {
                            before: b.clone(),
                            after: a.clone(),
                        },
                    );
                }
                (Some(b), None) => {
                    changes.insert(
                        key.clone(),
                        ValueChange {
                            before: b.clone(),
                            after: Value::Null,
                        },
                    );
                }
                (None, Some(a)) => {
                    changes.insert(
                        key.clone(),
                        ValueChange {
                            before: Value::Null,
                            after: a.clone(),
                        },
                    );
                }
                _ => {} // Equal or both Null/Missing
            }
        }

        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mock_template_values(
        identities: &[&str],
        ignores: &[&str],
    ) -> HashMap<String, TemplateValue> {
        let mut map = HashMap::new();
        for &id in identities {
            map.insert(
                id.to_string(),
                TemplateValue {
                    name: id.to_string(),
                    regex: "".to_string(),
                    filldown: false,
                    required: false,
                    list: false,
                    identity: true,
                    ignore: false,
                    common_schema: None,
                    constraints: None,
                    type_hint: None,
                },
            );
        }
        for &ign in ignores {
            map.insert(
                ign.to_string(),
                TemplateValue {
                    name: ign.to_string(),
                    regex: "".to_string(),
                    filldown: false,
                    required: false,
                    list: false,
                    identity: false,
                    ignore: true,
                    common_schema: None,
                    constraints: None,
                    type_hint: None,
                },
            );
        }
        map
    }

    #[test]
    fn test_basic_diff() {
        let values = mock_template_values(&["name"], &["uptime"]);
        let engine = SemanticDiffEngine::new(&values);

        let before = vec![
            json!({"name": "Gi0/1", "status": "up", "uptime": "10s"})
                .as_object()
                .unwrap()
                .clone()
                .into_iter()
                .collect(),
        ];
        let after = vec![
            json!({"name": "Gi0/1", "status": "down", "uptime": "20s"})
                .as_object()
                .unwrap()
                .clone()
                .into_iter()
                .collect(),
        ];

        let diffs = engine.diff(before, after);
        assert_eq!(diffs.len(), 1);
        if let DiffOp::Modified { identity, changes } = &diffs[0] {
            assert_eq!(identity.get("name").unwrap(), "Gi0/1");
            assert_eq!(changes.get("status").unwrap().before, "up");
            assert_eq!(changes.get("status").unwrap().after, "down");
            assert!(!changes.contains_key("uptime"));
        } else {
            panic!("Expected Modified op");
        }
    }
}
