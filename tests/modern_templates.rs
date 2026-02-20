use assert_cmd::Command;

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
fn cli_can_override_template_format_for_ambiguous_extension() {
    let doc = r#"
version: 1
fields:
  speed:
    type: int
patterns:
  - regex: '^speed=(?P<speed>[0-9,]+)$'
    record: true
"#;

    let template_path = write_temp_file("unknown", doc);

    let output = Command::cargo_bin("cliscrape")
        .unwrap()
        .args([
            "parse",
            "--template-format",
            "yaml",
            "-t",
            template_path.to_str().unwrap(),
        ])
        .write_stdin("speed=1,234\n")
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["speed"], serde_json::json!(1234));

    let _ = std::fs::remove_file(template_path);
}
