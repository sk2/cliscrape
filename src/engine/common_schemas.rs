//! Universal Ledger schema registry.
//!
//! Loads `common_schemas/*.yaml` at compile time via `include_str!`, parses each
//! into a typed [`CommonSchema`], and exposes a [`SchemaRegistry`] used by the
//! template loader to validate `common_schema:` field references.
//!
//! See `common_schemas/README.md` for the spec format and
//! `docs/guides/COMMON_SCHEMAS.md` for the user-facing guide.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

const BUILTIN_SCHEMA_FILES: &[(&str, &str)] = &[
    (
        "version",
        include_str!("../../common_schemas/version.yaml"),
    ),
    (
        "interface",
        include_str!("../../common_schemas/interface.yaml"),
    ),
    (
        "bgp_neighbor",
        include_str!("../../common_schemas/bgp_neighbor.yaml"),
    ),
    (
        "lldp_neighbor",
        include_str!("../../common_schemas/lldp_neighbor.yaml"),
    ),
    ("route", include_str!("../../common_schemas/route.yaml")),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    String,
    Int,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyDef {
    pub ty: KeyType,
    pub required: bool,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct CommonSchema {
    pub name: String,
    pub version: u32,
    pub description: String,
    pub applies_to: Vec<String>,
    pub keys: BTreeMap<String, KeyDef>,
}

impl CommonSchema {
    pub fn required_keys(&self) -> impl Iterator<Item = &str> {
        self.keys
            .iter()
            .filter(|(_, def)| def.required)
            .map(|(k, _)| k.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaValidationError {
    UnknownKey {
        template_field: String,
        key: String,
        suggestions: Vec<String>,
    },
    AmbiguousKey {
        template_field: String,
        key: String,
        schemas: Vec<String>,
    },
    TypeMismatch {
        template_field: String,
        key: String,
        schema: String,
        expected: KeyType,
        found: KeyType,
    },
}

impl std::fmt::Display for SchemaValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaValidationError::UnknownKey {
                template_field,
                key,
                suggestions,
            } => {
                write!(
                    f,
                    "field '{}' references common_schema key '{}' which is not declared by any built-in schema",
                    template_field, key
                )?;
                if !suggestions.is_empty() {
                    write!(f, ". Did you mean: {}?", suggestions.join(", "))?;
                }
                Ok(())
            }
            SchemaValidationError::AmbiguousKey {
                template_field,
                key,
                schemas,
            } => write!(
                f,
                "field '{}' references common_schema key '{}' which is declared by multiple schemas ({}); qualify with 'schema.key' or add a top-level 'claims_schema:'",
                template_field,
                key,
                schemas.join(", ")
            ),
            SchemaValidationError::TypeMismatch {
                template_field,
                key,
                schema,
                expected,
                found,
            } => write!(
                f,
                "field '{}' has type {:?} but '{}.{}' is declared as {:?} in the {} schema",
                template_field, found, schema, key, expected, schema
            ),
        }
    }
}

#[derive(Debug)]
pub struct SchemaRegistry {
    schemas: BTreeMap<String, CommonSchema>,
    bare_key_index: BTreeMap<String, Vec<String>>,
}

impl SchemaRegistry {
    pub fn get(&self, name: &str) -> Option<&CommonSchema> {
        self.schemas.get(name)
    }

    pub fn schemas(&self) -> impl Iterator<Item = &CommonSchema> {
        self.schemas.values()
    }

    pub fn schema_names(&self) -> impl Iterator<Item = &str> {
        self.schemas.keys().map(|s| s.as_str())
    }

    pub fn schemas_declaring_bare_key(&self, key: &str) -> &[String] {
        self.bare_key_index
            .get(key)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Resolve a `common_schema:` reference against the registry.
    ///
    /// Accepts both bare (`hostname`) and qualified (`version.hostname`) forms.
    /// When `claimed_schemas` is non-empty, bare keys are restricted to those
    /// schemas — this is how `claims_schema:` (cliscrape-1s8.6) disambiguates.
    pub fn resolve(
        &self,
        reference: &str,
        claimed_schemas: &[String],
    ) -> Result<(&CommonSchema, &str, &KeyDef), ResolveError> {
        if let Some((schema_name, key_name)) = reference.split_once('.') {
            let schema = self
                .schemas
                .get(schema_name)
                .ok_or_else(|| ResolveError::UnknownSchema {
                    schema: schema_name.to_string(),
                    known: self.schemas.keys().cloned().collect(),
                })?;
            let (stored_key, key) =
                schema
                    .keys
                    .get_key_value(key_name)
                    .ok_or_else(|| ResolveError::UnknownKey {
                        schema: Some(schema_name.to_string()),
                        key: key_name.to_string(),
                        suggestions: closest_keys(key_name, schema.keys.keys().map(|s| s.as_str())),
                    })?;
            return Ok((schema, stored_key.as_str(), key));
        }

        let candidates: Vec<&str> = if claimed_schemas.is_empty() {
            self.schemas_declaring_bare_key(reference)
                .iter()
                .map(|s| s.as_str())
                .collect()
        } else {
            self.schemas_declaring_bare_key(reference)
                .iter()
                .filter(|s| claimed_schemas.iter().any(|c| c == *s))
                .map(|s| s.as_str())
                .collect()
        };

        match candidates.as_slice() {
            [] => Err(ResolveError::UnknownKey {
                schema: None,
                key: reference.to_string(),
                suggestions: closest_keys(reference, self.bare_key_index.keys().map(|s| s.as_str())),
            }),
            [only] => {
                let schema = &self.schemas[*only];
                let (stored_key, key) = schema
                    .keys
                    .get_key_value(reference)
                    .expect("bare_key_index references a key absent from schema.keys");
                Ok((schema, stored_key.as_str(), key))
            }
            many => Err(ResolveError::AmbiguousKey {
                key: reference.to_string(),
                schemas: many.iter().map(|s| s.to_string()).collect(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolveError {
    UnknownSchema {
        schema: String,
        known: Vec<String>,
    },
    UnknownKey {
        schema: Option<String>,
        key: String,
        suggestions: Vec<String>,
    },
    AmbiguousKey {
        key: String,
        schemas: Vec<String>,
    },
}

pub fn builtin_registry() -> &'static SchemaRegistry {
    static REGISTRY: OnceLock<SchemaRegistry> = OnceLock::new();
    REGISTRY.get_or_init(load_builtin)
}

fn load_builtin() -> SchemaRegistry {
    let mut schemas = BTreeMap::new();
    let mut bare_key_index: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (expected_name, yaml) in BUILTIN_SCHEMA_FILES {
        let parsed: SchemaYaml = serde_yaml_ng::from_str(yaml).unwrap_or_else(|e| {
            panic!(
                "built-in common schema '{}' failed to parse: {}",
                expected_name, e
            )
        });
        assert_eq!(
            &parsed.schema, expected_name,
            "built-in schema name mismatch for {}",
            expected_name
        );
        let schema = CommonSchema::try_from(parsed).unwrap_or_else(|e| {
            panic!("built-in common schema '{}' invalid: {}", expected_name, e)
        });
        for key_name in schema.keys.keys() {
            bare_key_index
                .entry(key_name.clone())
                .or_default()
                .push(schema.name.clone());
        }
        schemas.insert(schema.name.clone(), schema);
    }

    SchemaRegistry {
        schemas,
        bare_key_index,
    }
}

#[derive(Deserialize)]
struct SchemaYaml {
    schema: String,
    version: u32,
    description: String,
    #[serde(default)]
    applies_to: Vec<String>,
    keys: BTreeMap<String, KeyDefYaml>,
}

#[derive(Deserialize)]
struct KeyDefYaml {
    #[serde(rename = "type")]
    ty: String,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    description: String,
}

impl TryFrom<SchemaYaml> for CommonSchema {
    type Error = String;

    fn try_from(yaml: SchemaYaml) -> Result<Self, Self::Error> {
        let mut keys = BTreeMap::new();
        for (name, def) in yaml.keys {
            let ty = match def.ty.as_str() {
                "string" => KeyType::String,
                "int" => KeyType::Int,
                other => {
                    return Err(format!(
                        "key '{}': unsupported type '{}' (expected 'string' or 'int')",
                        name, other
                    ));
                }
            };
            keys.insert(
                name,
                KeyDef {
                    ty,
                    required: def.required,
                    description: def.description,
                },
            );
        }
        Ok(CommonSchema {
            name: yaml.schema,
            version: yaml.version,
            description: yaml.description,
            applies_to: yaml.applies_to,
            keys,
        })
    }
}

fn closest_keys<'a>(query: &str, candidates: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut scored: Vec<(usize, &str)> = candidates
        .map(|c| (levenshtein(query, c), c))
        .filter(|(d, _)| *d <= 2)
        .collect();
    scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));
    scored
        .into_iter()
        .take(3)
        .map(|(_, s)| s.to_string())
        .collect()
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (prev[j + 1] + 1)
                .min(curr[j] + 1)
                .min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_schemas_load() {
        let reg = builtin_registry();
        let names: Vec<&str> = reg.schema_names().collect();
        assert!(names.contains(&"version"));
        assert!(names.contains(&"interface"));
        assert!(names.contains(&"bgp_neighbor"));
        assert!(names.contains(&"lldp_neighbor"));
        assert!(names.contains(&"route"));
    }

    #[test]
    fn version_schema_required_keys() {
        let reg = builtin_registry();
        let version = reg.get("version").unwrap();
        let required: Vec<&str> = version.required_keys().collect();
        assert!(required.contains(&"hostname"));
        assert!(required.contains(&"model"));
        assert!(required.contains(&"version"));
    }

    #[test]
    fn route_schema_protocol_is_required() {
        let reg = builtin_registry();
        let route = reg.get("route").unwrap();
        assert!(route.keys["protocol"].required);
        assert_eq!(route.keys["prefix"].ty, KeyType::String);
        assert_eq!(route.keys["metric"].ty, KeyType::Int);
    }

    #[test]
    fn resolve_bare_key_unique() {
        let reg = builtin_registry();
        let (schema, key_name, def) = reg.resolve("hostname", &[]).unwrap();
        assert_eq!(schema.name, "version");
        assert_eq!(key_name, "hostname");
        assert_eq!(def.ty, KeyType::String);
    }

    #[test]
    fn resolve_qualified_key() {
        let reg = builtin_registry();
        let (schema, key_name, _) = reg.resolve("interface.name", &[]).unwrap();
        assert_eq!(schema.name, "interface");
        assert_eq!(key_name, "name");
    }

    #[test]
    fn resolve_unknown_key_offers_suggestion() {
        let reg = builtin_registry();
        let err = reg.resolve("hostnme", &[]).unwrap_err();
        match err {
            ResolveError::UnknownKey { suggestions, .. } => {
                assert!(
                    suggestions.contains(&"hostname".to_string()),
                    "expected 'hostname' suggestion, got {:?}",
                    suggestions
                );
            }
            other => panic!("expected UnknownKey, got {:?}", other),
        }
    }

    #[test]
    fn resolve_unknown_schema_in_qualified_form() {
        let reg = builtin_registry();
        let err = reg.resolve("nopesuch.foo", &[]).unwrap_err();
        match err {
            ResolveError::UnknownSchema { schema, .. } => {
                assert_eq!(schema, "nopesuch");
            }
            other => panic!("expected UnknownSchema, got {:?}", other),
        }
    }

    #[test]
    fn levenshtein_basics() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("hostname", "hostnme"), 1);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("abc", "abc"), 0);
    }

    #[test]
    fn closest_keys_returns_close_matches() {
        let candidates = vec!["hostname", "model", "version", "serial"];
        let suggestions = closest_keys("hostnme", candidates.iter().copied());
        assert_eq!(suggestions, vec!["hostname"]);
    }

    #[test]
    fn closest_keys_ignores_distant() {
        let candidates = vec!["hostname", "model"];
        let suggestions = closest_keys("xyz", candidates.iter().copied());
        assert!(suggestions.is_empty());
    }

    #[test]
    fn known_bare_key_collisions_resolve_as_ambiguous() {
        // Schemas legitimately share keys (uptime: version + bgp_neighbor;
        // vrf: bgp_neighbor + route). Without claims_schema (1s8.6), bare
        // references must surface as AmbiguousKey errors so templates fail
        // to load until they qualify with `schema.key`.
        let reg = builtin_registry();
        let err = reg.resolve("uptime", &[]).unwrap_err();
        match err {
            ResolveError::AmbiguousKey { schemas, .. } => {
                assert!(schemas.contains(&"version".to_string()));
                assert!(schemas.contains(&"bgp_neighbor".to_string()));
            }
            other => panic!("expected AmbiguousKey for 'uptime', got {:?}", other),
        }

        let err = reg.resolve("vrf", &[]).unwrap_err();
        assert!(matches!(err, ResolveError::AmbiguousKey { .. }));
    }

    #[test]
    fn ambiguous_bare_key_resolves_when_disambiguated_by_claim() {
        // claims_schema scope (1s8.6 will produce this list) restricts bare
        // resolution to the claimed schema's key set.
        let reg = builtin_registry();
        let claimed = vec!["version".to_string()];
        let (schema, _, _) = reg.resolve("uptime", &claimed).unwrap();
        assert_eq!(schema.name, "version");

        let claimed = vec!["bgp_neighbor".to_string()];
        let (schema, _, _) = reg.resolve("uptime", &claimed).unwrap();
        assert_eq!(schema.name, "bgp_neighbor");
    }
}
