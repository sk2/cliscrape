//! Integration tests for the common-schema validator wired into the modern
//! template loader. Exercises the public `load_yaml_str` path so that any
//! refactoring that bypasses the validator surfaces here.

use cliscrape::template::modern::load_yaml_str;

#[test]
fn unknown_common_schema_key_fails_with_suggestion() {
    let template = r#"
version: 1
fields:
  hostname:
    type: string
    common_schema: hostnme
patterns:
  - regex: '^Host=(?P<hostname>\S+)$'
    record: true
"#;
    let err = load_yaml_str(template).unwrap_err().to_string();
    assert!(
        err.contains("hostnme"),
        "error should mention the bad key: {err}"
    );
    assert!(
        err.contains("hostname"),
        "error should suggest 'hostname': {err}"
    );
}

#[test]
fn type_mismatch_between_field_and_schema_key_fails() {
    let template = r#"
version: 1
fields:
  hostname:
    type: int
    common_schema: hostname
patterns:
  - regex: '^Host=(?P<hostname>\d+)$'
    record: true
"#;
    let err = load_yaml_str(template).unwrap_err().to_string();
    assert!(
        err.contains("hostname"),
        "error should mention the field name: {err}"
    );
    assert!(
        err.to_lowercase().contains("string"),
        "error should mention expected type: {err}"
    );
    assert!(
        err.to_lowercase().contains("int"),
        "error should mention found type: {err}"
    );
}

#[test]
fn ambiguous_bare_key_fails_without_qualification() {
    // 'uptime' is in both version and bgp_neighbor schemas. Bare reference
    // must fail until `claims_schema:` (cliscrape-1s8.6) disambiguates.
    let template = r#"
version: 1
fields:
  uptime:
    type: string
    common_schema: uptime
patterns:
  - regex: '^uptime is (?P<uptime>.+)$'
    record: true
"#;
    let err = load_yaml_str(template).unwrap_err().to_string();
    assert!(
        err.to_lowercase().contains("multiple schemas"),
        "error should call out the ambiguity: {err}"
    );
    assert!(err.contains("version"), "error should list 'version': {err}");
    assert!(
        err.contains("bgp_neighbor"),
        "error should list 'bgp_neighbor': {err}"
    );
}

#[test]
fn qualified_reference_resolves_unambiguously() {
    let template = r#"
version: 1
fields:
  uptime:
    type: string
    common_schema: version.uptime
patterns:
  - regex: '^uptime is (?P<uptime>.+)$'
    record: true
"#;
    load_yaml_str(template).expect("qualified common_schema reference should load");
}

#[test]
fn qualified_reference_with_unknown_schema_fails() {
    let template = r#"
version: 1
fields:
  hostname:
    type: string
    common_schema: nopesuch.hostname
patterns:
  - regex: '^Host=(?P<hostname>\S+)$'
    record: true
"#;
    let err = load_yaml_str(template).unwrap_err().to_string();
    assert!(err.contains("nopesuch"));
    assert!(
        err.contains("no such schema") || err.contains("known"),
        "error should explain why: {err}"
    );
}

#[test]
fn template_without_common_schema_still_loads() {
    let template = r#"
version: 1
fields:
  whatever:
    type: string
patterns:
  - regex: '^x=(?P<whatever>\S+)$'
    record: true
"#;
    load_yaml_str(template).expect("template without common_schema should load unchanged");
}

#[test]
fn unique_bare_key_resolves() {
    let template = r#"
version: 1
fields:
  hostname:
    type: string
    common_schema: hostname
patterns:
  - regex: '^Host=(?P<hostname>\S+)$'
    record: true
"#;
    load_yaml_str(template).expect("unique bare key should resolve");
}
