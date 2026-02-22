use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_no_args_shows_help() {
    Command::cargo_bin("tforge")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn test_version_flag() {
    Command::cargo_bin("tforge")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("tforge"));
}

#[test]
fn test_list_subcommand() {
    Command::cargo_bin("tforge")
        .unwrap()
        .arg("list")
        .assert()
        .success();
}
