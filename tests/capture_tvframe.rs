//! End-to-end test for `cliscrape parse --capture` (telemetry-vis ADR-0002).
//!
//! Runs a real Universal Ledger projection on a Cisco IOS show-interfaces
//! fixture, writes the capture, and asserts the resulting `.tvframe.ndjson`
//! has the envelope shape any consumer (telemetry-vis `tv`, future workbench
//! panels) can rely on.

use assert_cmd::Command;
use std::fs;

const TVFRAME_SCHEMA: &str = "netauto/universal-ledger/v1.0";

#[test]
fn parse_capture_emits_valid_tvframe_ndjson() {
    let capture = tempfile::NamedTempFile::with_suffix(".tvframe.ndjson")
        .expect("temp file creates");

    let output = Command::new(assert_cmd::cargo::cargo_bin!("cliscrape"))
        .args([
            "parse",
            "--template",
            "cisco_ios_show_interfaces.yaml",
            "tests/fixtures/cisco/ios_show_interfaces/ios_15_standard.txt",
            "--common",
            "--capture",
            capture.path().to_str().unwrap(),
            "--capture-device",
            "core-01",
            "--quiet",
        ])
        .output()
        .expect("cliscrape parse runs");

    assert!(
        output.status.success(),
        "cliscrape failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );

    let raw = fs::read_to_string(capture.path()).expect("capture file readable");
    let lines: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(
        lines.len() >= 2,
        "expected metadata + at least one record, got {}",
        lines.len(),
    );

    // Every line is valid JSON with the right source envelope.
    let frames: Vec<serde_json::Value> = lines
        .iter()
        .map(|line| serde_json::from_str(line).expect("each line is valid JSON"))
        .collect();
    for frame in &frames {
        assert_eq!(frame["source"]["name"], "cliscrape");
        assert_eq!(frame["source"]["schema"], TVFRAME_SCHEMA);
    }

    // Metadata frame: no observed_at, attributes carry command/template/device.
    let metadata = &frames[0];
    assert!(
        metadata["observed_at"].is_null(),
        "metadata frame should have no observed_at",
    );
    assert!(
        metadata["attributes"]["cliscrape.command"].is_string(),
        "metadata frame carries the cliscrape command-line",
    );
    assert_eq!(
        metadata["attributes"]["cliscrape.device"], "core-01",
        "device passed via --capture-device lands in metadata",
    );

    // Subsequent frames carry observed_at and per-record attributes.
    for frame in frames.iter().skip(1) {
        assert!(
            frame["observed_at"].is_string(),
            "non-metadata frame should have observed_at",
        );
        assert!(
            frame["attributes"]
                .as_object()
                .is_some_and(|attrs| attrs.iter().any(|(k, _)| k.starts_with("cliscrape."))),
            "record frame should carry cliscrape.<key> attributes",
        );
    }
}

#[test]
fn capture_requires_common_flag() {
    // --capture without --common is rejected because the schema slot would
    // otherwise carry vendor-specific keys, violating ADR-0002's vendor-neutral
    // envelope requirement.
    let capture = tempfile::NamedTempFile::with_suffix(".tvframe.ndjson")
        .expect("temp file creates");

    let output = Command::new(assert_cmd::cargo::cargo_bin!("cliscrape"))
        .args([
            "parse",
            "--template",
            "cisco_ios_show_interfaces.yaml",
            "tests/fixtures/cisco/ios_show_interfaces/ios_15_standard.txt",
            "--capture",
            capture.path().to_str().unwrap(),
        ])
        .output()
        .expect("cliscrape parse runs");

    assert!(
        !output.status.success(),
        "--capture without --common should be rejected",
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--capture requires --common"),
        "error message should explain the constraint, got: {stderr}",
    );
}
