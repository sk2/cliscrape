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

#[test]
fn claims_schema_with_all_required_keys_loads() {
    let template = r#"
version: 1
claims_schema: version
fields:
  hostname:
    type: string
    common_schema: hostname
  model:
    type: string
    common_schema: model
  version:
    type: string
    common_schema: version
patterns:
  - regex: '^(?P<hostname>\S+) (?P<model>\S+) (?P<version>\S+)$'
    record: true
"#;
    load_yaml_str(template).expect("claims_schema with all required keys should load");
}

#[test]
fn claims_schema_missing_required_key_fails() {
    // version schema requires hostname, model, version. This template only
    // maps hostname.
    let template = r#"
version: 1
claims_schema: version
fields:
  hostname:
    type: string
    common_schema: hostname
patterns:
  - regex: '^Host=(?P<hostname>\S+)$'
    record: true
"#;
    let err = load_yaml_str(template).unwrap_err().to_string();
    assert!(
        err.contains("claims_schema 'version'"),
        "error should name the schema: {err}"
    );
    assert!(
        err.contains("model") && err.contains("version"),
        "error should list missing keys: {err}"
    );
}

#[test]
fn claims_schema_disambiguates_otherwise_ambiguous_bare_key() {
    // 'uptime' is normally ambiguous (version + bgp_neighbor). With
    // claims_schema: version, the bare reference resolves to version.uptime.
    let template = r#"
version: 1
claims_schema: version
fields:
  hostname:
    type: string
    common_schema: hostname
  model:
    type: string
    common_schema: model
  version:
    type: string
    common_schema: version
  uptime:
    type: string
    common_schema: uptime
patterns:
  - regex: '^(?P<hostname>\S+) (?P<model>\S+) (?P<version>\S+) (?P<uptime>.+)$'
    record: true
"#;
    load_yaml_str(template).expect("claims_schema scope should disambiguate uptime");
}

#[test]
fn claims_schema_unknown_name_fails() {
    let template = r#"
version: 1
claims_schema: not_a_real_schema
fields:
  whatever:
    type: string
patterns:
  - regex: '^x=(?P<whatever>\S+)$'
    record: true
"#;
    let err = load_yaml_str(template).unwrap_err().to_string();
    assert!(
        err.contains("not_a_real_schema"),
        "error should name the bad schema: {err}"
    );
    assert!(
        err.contains("known"),
        "error should list known schemas: {err}"
    );
}

#[test]
fn claims_schema_as_list_loads() {
    // Multi-schema claim is uncommon but supported. version + interface
    // share no required keys, so this template needs to satisfy both.
    let template = r#"
version: 1
claims_schema: [version, interface]
fields:
  hostname:
    type: string
    common_schema: hostname
  model:
    type: string
    common_schema: model
  version:
    type: string
    common_schema: version
  iface_name:
    type: string
    common_schema: name
patterns:
  - regex: '^(?P<hostname>\S+) (?P<model>\S+) (?P<version>\S+) (?P<iface_name>\S+)$'
    record: true
"#;
    load_yaml_str(template).expect("multi-schema claim should load when all required keys are mapped");
}
