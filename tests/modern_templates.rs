use assert_cmd::Command;
use cliscrape::FsmParser;

fn write_temp_file(ext: &str, content: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    let uniq = format!(
        "cliscrape-test-{}-{}.{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        ext
    );
    path.push(uniq);
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn from_file_loads_starter_yaml_template_and_emits_typed_int() {
    let parser = FsmParser::from_file("templates/modern/ios_show_interfaces.yaml").unwrap();
    let results = parser
        .parse("Interface GigabitEthernet0/0, MTU 1500")
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["iface"], serde_json::json!("GigabitEthernet0/0"));
    assert_eq!(results[0]["mtu"], serde_json::json!(1500));
}

#[test]
fn from_file_loads_starter_toml_template_in_pattern_mode() {
    let parser = FsmParser::from_file("templates/modern/simple_hostname.toml").unwrap();
    let results = parser.parse("Hostname: Router1").unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["hostname"], serde_json::json!("Router1"));
}

#[test]
fn cli_can_override_template_format_for_ambiguous_extension() {
    let doc = std::fs::read_to_string("templates/modern/ios_show_interfaces.yaml").unwrap();
    let template_path = write_temp_file("unknown", &doc);

    let output = Command::cargo_bin("cliscrape")
        .unwrap()
        .args([
            "parse",
            "--template-format",
            "yaml",
            "-t",
            template_path.to_str().unwrap(),
        ])
        .write_stdin("Interface GigabitEthernet0/0, MTU 1500\n")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["mtu"], serde_json::json!(1500));

    let _ = std::fs::remove_file(template_path);
}
