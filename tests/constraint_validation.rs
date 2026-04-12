use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_min_constraint_pass() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cliscrape"));
    cmd.args(&[
        "parse",
        "tests/fixtures/inputs/constraints/min_pass.txt",
        "--template",
        "tests/fixtures/templates/constraints.yaml",
    ]);
    cmd.assert().success();
}

#[test]
fn test_min_constraint_fail_warning() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cliscrape"));
    cmd.args(&[
        "parse",
        "tests/fixtures/inputs/constraints/min_fail.txt",
        "--template",
        "tests/fixtures/templates/constraints.yaml",
    ]);
    cmd.assert()
        .success()
        .stderr(predicates::str::contains("ConstraintViolation"));
}

#[test]
fn test_min_constraint_fail_strict() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cliscrape"));
    cmd.args(&[
        "parse",
        "tests/fixtures/inputs/constraints/min_fail.txt",
        "--template",
        "tests/fixtures/templates/constraints.yaml",
        "--strict-policy",
    ]);
    cmd.assert().failure().stderr(predicates::str::contains(
        "Constraint violation failed strict policy",
    ));
}

#[test]
fn test_choices_constraint_fail_strict() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cliscrape"));
    cmd.args(&[
        "parse",
        "tests/fixtures/inputs/constraints/choices_fail.txt",
        "--template",
        "tests/fixtures/templates/constraints.yaml",
        "--strict-policy",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("is not in allowed choices"));
}
