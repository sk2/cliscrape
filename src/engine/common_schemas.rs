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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyFormat {
    /// IPv4 dotted-quad address (e.g. "10.0.0.1").
    Ipv4,
    /// IPv6 address per RFC 4291 (e.g. "2001:db8::1").
    Ipv6,
    /// Either IPv4 or IPv6.
    Ip,
    /// Lowercase, colon-separated MAC (e.g. "00:1c:2d:3e:4f:50").
    Mac,
    /// CIDR-notation prefix (`<ip>/<len>`).
    Cidr,
    /// Autonomous System Number, 0..=4294967295 (4-byte ASN). Applied to
    /// `int`-typed keys; checks numeric range only.
    Asn,
}

impl KeyFormat {
    /// Returns `Ok(())` if `value` matches the declared format; `Err(reason)`
    /// otherwise. The error string is suitable for tracing/error output.
    pub fn validate(self, value: &serde_json::Value) -> Result<(), String> {
        use std::net::{Ipv4Addr, Ipv6Addr};
        use std::str::FromStr;

        match self {
            KeyFormat::Ipv4 => {
                let s = value
                    .as_str()
                    .ok_or_else(|| format!("expected string for ipv4, got {}", value))?;
                Ipv4Addr::from_str(s)
                    .map(|_| ())
                    .map_err(|_| format!("'{}' is not a valid IPv4 address", s))
            }
            KeyFormat::Ipv6 => {
                let s = value
                    .as_str()
                    .ok_or_else(|| format!("expected string for ipv6, got {}", value))?;
                Ipv6Addr::from_str(s)
                    .map(|_| ())
                    .map_err(|_| format!("'{}' is not a valid IPv6 address", s))
            }
            KeyFormat::Ip => {
                let s = value
                    .as_str()
                    .ok_or_else(|| format!("expected string for ip, got {}", value))?;
                if Ipv4Addr::from_str(s).is_ok() || Ipv6Addr::from_str(s).is_ok() {
                    Ok(())
                } else {
                    Err(format!("'{}' is not a valid IPv4 or IPv6 address", s))
                }
            }
            KeyFormat::Mac => {
                let s = value
                    .as_str()
                    .ok_or_else(|| format!("expected string for mac, got {}", value))?;
                static MAC_RE: OnceLock<regex::Regex> = OnceLock::new();
                let re = MAC_RE.get_or_init(|| {
                    regex::Regex::new(r"^[0-9a-f]{2}(?::[0-9a-f]{2}){5}$").unwrap()
                });
                if re.is_match(s) {
                    Ok(())
                } else {
                    Err(format!(
                        "'{}' is not a valid MAC address (expected lowercase colon-separated, e.g. 00:1c:2d:3e:4f:50)",
                        s
                    ))
                }
            }
            KeyFormat::Cidr => {
                let s = value
                    .as_str()
                    .ok_or_else(|| format!("expected string for cidr, got {}", value))?;
                let Some((addr, prefix)) = s.rsplit_once('/') else {
                    return Err(format!("'{}' is not in CIDR notation (missing '/')", s));
                };
                let prefix_len: u32 = prefix
                    .parse()
                    .map_err(|_| format!("'{}' has invalid prefix length", s))?;
                if Ipv4Addr::from_str(addr).is_ok() {
                    if prefix_len > 32 {
                        return Err(format!("'{}' has invalid IPv4 prefix length", s));
                    }
                } else if Ipv6Addr::from_str(addr).is_ok() {
                    if prefix_len > 128 {
                        return Err(format!("'{}' has invalid IPv6 prefix length", s));
                    }
                } else {
                    return Err(format!("'{}' has invalid address part", s));
                }
                Ok(())
            }
            KeyFormat::Asn => match value {
                serde_json::Value::Number(n) => {
                    let v = n.as_u64().ok_or_else(|| {
                        format!("ASN '{}' must be a non-negative integer", n)
                    })?;
                    if v <= u32::MAX as u64 {
                        Ok(())
                    } else {
                        Err(format!("ASN {} exceeds 4-byte range", v))
                    }
                }
                other => Err(format!("expected integer for asn, got {}", other)),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyDef {
    pub ty: KeyType,
    pub required: bool,
    pub description: String,
    pub format: Option<KeyFormat>,
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
    #[serde(default)]
    format: Option<String>,
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
            let format = match def.format.as_deref() {
                None => None,
                Some("ipv4") => Some(KeyFormat::Ipv4),
                Some("ipv6") => Some(KeyFormat::Ipv6),
                Some("ip") => Some(KeyFormat::Ip),
                Some("mac") => Some(KeyFormat::Mac),
                Some("cidr") => Some(KeyFormat::Cidr),
                Some("asn") => Some(KeyFormat::Asn),
                Some(other) => {
                    return Err(format!(
                        "key '{}': unsupported format '{}' (expected ipv4, ipv6, ip, mac, cidr, or asn)",
                        name, other
                    ));
                }
            };
            // ASN format requires int type, IP/MAC/CIDR require string.
            match (format, ty) {
                (Some(KeyFormat::Asn), KeyType::Int) | (Some(KeyFormat::Asn), _) => {
                    if !matches!(ty, KeyType::Int) {
                        return Err(format!(
                            "key '{}': format 'asn' requires type 'int'",
                            name
                        ));
                    }
                }
                (
                    Some(KeyFormat::Ipv4 | KeyFormat::Ipv6 | KeyFormat::Ip | KeyFormat::Mac | KeyFormat::Cidr),
                    KeyType::Int,
                ) => {
                    return Err(format!(
                        "key '{}': format '{:?}' requires type 'string'",
                        name,
                        format.unwrap()
                    ));
                }
                _ => {}
            }
            keys.insert(
                name,
                KeyDef {
                    ty,
                    required: def.required,
                    description: def.description,
                    format,
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
        // hostname was downgraded from required to optional during 1s8.2
        // wiring: Arista EOS' show version does not print hostname.
        let reg = builtin_registry();
        let version = reg.get("version").unwrap();
        let required: Vec<&str> = version.required_keys().collect();
        assert!(required.contains(&"model"));
        assert!(required.contains(&"version"));
        assert!(!required.contains(&"hostname"));
        assert!(version.keys.contains_key("hostname"));
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
    fn format_ipv4_accepts_valid_rejects_invalid() {
        assert!(
            KeyFormat::Ipv4
                .validate(&serde_json::json!("10.0.0.1"))
                .is_ok()
        );
        assert!(
            KeyFormat::Ipv4
                .validate(&serde_json::json!("192.0.2.1"))
                .is_ok()
        );
        assert!(
            KeyFormat::Ipv4
                .validate(&serde_json::json!("256.0.0.1"))
                .is_err()
        );
        assert!(
            KeyFormat::Ipv4
                .validate(&serde_json::json!("Router1"))
                .is_err()
        );
        assert!(
            KeyFormat::Ipv4
                .validate(&serde_json::json!("2001:db8::1"))
                .is_err()
        );
    }

    #[test]
    fn format_ipv6_accepts_valid_rejects_invalid() {
        assert!(
            KeyFormat::Ipv6
                .validate(&serde_json::json!("2001:db8::1"))
                .is_ok()
        );
        assert!(
            KeyFormat::Ipv6
                .validate(&serde_json::json!("::1"))
                .is_ok()
        );
        assert!(
            KeyFormat::Ipv6
                .validate(&serde_json::json!("10.0.0.1"))
                .is_err()
        );
        assert!(KeyFormat::Ipv6.validate(&serde_json::json!("xyz")).is_err());
    }

    #[test]
    fn format_ip_accepts_either_v4_or_v6() {
        assert!(
            KeyFormat::Ip
                .validate(&serde_json::json!("10.0.0.1"))
                .is_ok()
        );
        assert!(
            KeyFormat::Ip
                .validate(&serde_json::json!("2001:db8::1"))
                .is_ok()
        );
        assert!(
            KeyFormat::Ip
                .validate(&serde_json::json!("Router1"))
                .is_err()
        );
    }

    #[test]
    fn format_mac_requires_lowercase_colon_separated() {
        assert!(
            KeyFormat::Mac
                .validate(&serde_json::json!("00:1c:2d:3e:4f:50"))
                .is_ok()
        );
        // Uppercase rejected — schemas require lowercase.
        assert!(
            KeyFormat::Mac
                .validate(&serde_json::json!("00:1C:2D:3E:4F:50"))
                .is_err()
        );
        // Cisco-style dot-separated rejected.
        assert!(
            KeyFormat::Mac
                .validate(&serde_json::json!("001c.2d3e.4f50"))
                .is_err()
        );
        // Hyphen-separated rejected.
        assert!(
            KeyFormat::Mac
                .validate(&serde_json::json!("00-1c-2d-3e-4f-50"))
                .is_err()
        );
        // Too short.
        assert!(
            KeyFormat::Mac
                .validate(&serde_json::json!("00:1c:2d:3e:4f"))
                .is_err()
        );
    }

    #[test]
    fn format_cidr_accepts_v4_and_v6_with_valid_prefix_lengths() {
        assert!(
            KeyFormat::Cidr
                .validate(&serde_json::json!("10.0.0.0/24"))
                .is_ok()
        );
        assert!(
            KeyFormat::Cidr
                .validate(&serde_json::json!("2001:db8::/32"))
                .is_ok()
        );
        assert!(
            KeyFormat::Cidr
                .validate(&serde_json::json!("10.0.0.0/33"))
                .is_err()
        );
        assert!(
            KeyFormat::Cidr
                .validate(&serde_json::json!("10.0.0.0"))
                .is_err()
        );
        assert!(
            KeyFormat::Cidr
                .validate(&serde_json::json!("xyz/24"))
                .is_err()
        );
    }

    #[test]
    fn format_asn_accepts_valid_4byte_range() {
        assert!(KeyFormat::Asn.validate(&serde_json::json!(64512)).is_ok());
        assert!(
            KeyFormat::Asn
                .validate(&serde_json::json!(4_294_967_295u32))
                .is_ok()
        );
        // Exceeds 4-byte range.
        assert!(
            KeyFormat::Asn
                .validate(&serde_json::json!(5_000_000_000u64))
                .is_err()
        );
        // Wrong type.
        assert!(
            KeyFormat::Asn
                .validate(&serde_json::json!("65000"))
                .is_err()
        );
    }

    #[test]
    fn shipped_schemas_have_format_declarations_where_documented() {
        // Smoke test: the format: amendments to the shipped schemas should
        // round-trip through the loader.
        let reg = builtin_registry();
        assert_eq!(
            reg.get("interface").unwrap().keys["mac_address"].format,
            Some(KeyFormat::Mac)
        );
        assert_eq!(
            reg.get("interface").unwrap().keys["ipv4_address"].format,
            Some(KeyFormat::Ipv4)
        );
        assert_eq!(
            reg.get("bgp_neighbor").unwrap().keys["neighbor"].format,
            Some(KeyFormat::Ip)
        );
        assert_eq!(
            reg.get("bgp_neighbor").unwrap().keys["remote_as"].format,
            Some(KeyFormat::Asn)
        );
        assert_eq!(
            reg.get("route").unwrap().keys["prefix"].format,
            Some(KeyFormat::Cidr)
        );
        assert_eq!(
            reg.get("route").unwrap().keys["next_hop"].format,
            Some(KeyFormat::Ip)
        );
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
