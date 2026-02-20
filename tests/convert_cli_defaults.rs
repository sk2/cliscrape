use assert_cmd::Command;

#[test]
fn convert_defaults_non_interactive_writes_output() {
    // Ensure output directory exists (cargo test doesn't create it).
    std::fs::create_dir_all("target").expect("create target dir");

    let mut cmd = Command::cargo_bin("cliscrape").expect("cliscrape binary builds");
    cmd.args([
        "convert",
        "-i",
        "test_required.textfsm",
        "--defaults",
        "--output",
        "target/tmp_converted.yaml",
    ]);
    cmd.assert().success();

    // Basic sanity: the file exists and is non-empty.
    let out = std::fs::read_to_string("target/tmp_converted.yaml")
        .expect("converted output should be written");
    assert!(!out.trim().is_empty());
}
