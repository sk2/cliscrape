//! Schema-compliance gate. Walks every embedded template that declares
//! `claims_schema:`, parses the matching fixture(s) under
//! `tests/fixtures/<vendor>/<command>/`, projects to Universal Ledger keys,
//! and asserts:
//!
//! * required keys for each claimed schema are present in every emitted
//!   record;
//! * format-declared values pass their validators (no `format_violation`s);
//! * the registry resolves every `common_schema:` reference (validated at
//!   template-load time, but re-checked end-to-end here for belt-and-braces).
//!
//! A negative-path test asserts the format check actually catches bad data —
//! without it, a refactor that breaks discovery would pass silently.

use cliscrape::FsmParser;
use cliscrape::engine::common_schemas::builtin_registry;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[test]
fn every_claiming_template_satisfies_its_schema_against_committed_fixtures() {
    let registry = builtin_registry();
    let mut checked_pairs = 0u32;
    let mut errors: Vec<String> = Vec::new();

    for template_path in glob_templates() {
        let parser = FsmParser::from_file(&template_path).unwrap_or_else(|e| {
            panic!("failed to load template {}: {}", template_path.display(), e)
        });

        let claims = parser.claims_schema();
        if claims.is_empty() {
            continue;
        }

        let fixtures = discover_fixtures(&template_path);
        if fixtures.is_empty() {
            errors.push(format!(
                "template '{}' claims schema(s) {:?} but no fixture under tests/fixtures/<vendor>/<command>/ matches",
                template_path.display(),
                claims,
            ));
            continue;
        }

        for fixture_path in fixtures {
            let input = std::fs::read_to_string(&fixture_path).unwrap_or_else(|e| {
                panic!(
                    "failed to read fixture {}: {}",
                    fixture_path.display(),
                    e
                )
            });
            let records = parser.parse(&input).unwrap_or_else(|e| {
                panic!(
                    "template '{}' failed to parse fixture '{}': {}",
                    template_path.display(),
                    fixture_path.display(),
                    e
                )
            });

            if records.is_empty() {
                errors.push(format!(
                    "template '{}' produced 0 records for fixture '{}' — fixture likely doesn't match the template's patterns",
                    template_path.display(),
                    fixture_path.display(),
                ));
                continue;
            }

            let projection = parser.project_common_schema(records.clone());

            // Format violations must be zero.
            for v in &projection.format_violations {
                errors.push(format!(
                    "template '{}' fixture '{}': field '{}' (schema {}.{}) failed format check '{:?}': {}",
                    template_path.display(),
                    fixture_path.display(),
                    v.field,
                    v.schema,
                    v.key,
                    v.format,
                    v.reason,
                ));
            }

            // Required keys must be present in every projected record for
            // every claimed schema.
            for schema_name in claims {
                let schema = registry.get(schema_name).unwrap_or_else(|| {
                    panic!(
                        "template '{}' claims unknown schema '{}'",
                        template_path.display(),
                        schema_name
                    )
                });
                let required: Vec<&str> = schema.required_keys().collect();
                for (idx, rec) in projection.records.iter().enumerate() {
                    for req in &required {
                        if !rec.contains_key(*req) {
                            errors.push(format!(
                                "template '{}' fixture '{}' record #{}: missing required key '{}.{}'",
                                template_path.display(),
                                fixture_path.display(),
                                idx,
                                schema_name,
                                req,
                            ));
                        }
                    }
                }
            }

            checked_pairs += 1;
        }
    }

    assert!(
        checked_pairs > 0,
        "compliance test discovered no (template, fixture) pairs to validate"
    );
    assert!(
        errors.is_empty(),
        "schema compliance failures ({} pairs checked):\n{}",
        checked_pairs,
        errors.join("\n")
    );
}

#[test]
fn format_check_rejects_malformed_value_in_synthetic_record() {
    // Belt-and-braces: prove the compliance machinery actually catches bad
    // data. Without this, a refactor that breaks the projection's
    // format-check call would silently pass the positive-path test on
    // already-clean fixtures.
    //
    // Uses cisco_ios_show_interfaces (claims `interface`, maps `mtu` ->
    // bandwidth_kbps which is int but unconstrained). We synthesize a
    // record with a bad ipv4_address-shaped value mapped via a fresh
    // template loaded from an inline string.
    let template_yaml = r#"
version: 1
claims_schema: interface
fields:
  iface_name:
    type: string
    common_schema: name
  addr:
    type: string
    common_schema: ipv4_address
patterns:
  - regex: '^(?P<iface_name>\S+)\s+(?P<addr>.+)$'
    record: true
"#;
    let parser = FsmParser::from_yaml_str(template_yaml).expect("template loads");

    // Synthesize a record with a malformed IPv4 value the regex would have
    // happily captured.
    let mut rec = BTreeMap::new();
    rec.insert(
        "iface_name".to_string(),
        serde_json::Value::String("Ethernet1".to_string()),
    );
    rec.insert(
        "addr".to_string(),
        serde_json::Value::String("Router1".to_string()),
    );

    let projection = parser.project_common_schema(vec![rec]);
    assert!(
        !projection.format_violations.is_empty(),
        "expected format_violation for malformed ipv4_address; got none"
    );
    let v = &projection.format_violations[0];
    assert_eq!(v.schema, "interface");
    assert_eq!(v.key, "ipv4_address");
    assert!(
        v.reason.contains("Router1"),
        "violation message should name the bad value: {}",
        v.reason
    );
}

fn glob_templates() -> Vec<PathBuf> {
    let templates_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
    let mut out = Vec::new();
    visit(&templates_dir, &mut out);
    out
}

fn visit(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
            out.push(path);
        }
    }
}

fn discover_fixtures(template_path: &Path) -> Vec<PathBuf> {
    // Templates are named e.g. `cisco_ios_show_version.yaml` and live under
    // tests/fixtures/<vendor>/<rest>/<*.txt>. The first underscore-segment
    // is the vendor; the rest forms the command directory.
    let stem = template_path.file_stem().and_then(|s| s.to_str()).unwrap();
    let Some((vendor, command)) = stem.split_once('_') else {
        return Vec::new();
    };
    let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(vendor)
        .join(command);
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(&fixture_dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("txt") {
            out.push(p);
        }
    }
    out
}
