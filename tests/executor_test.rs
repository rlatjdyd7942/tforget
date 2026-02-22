use tforge::executor::{execute_step, StepContext};
use tforge::types::StepDef;
use std::collections::HashMap;
use tempfile::TempDir;

fn make_command_step(cmd: &str) -> StepDef {
    toml::from_str(&format!(
        r#"
type = "command"
command = "{cmd}"
"#
    ))
    .unwrap()
}

#[test]
fn test_execute_simple_command() {
    let tmp = TempDir::new().unwrap();
    let ctx = StepContext {
        project_dir: tmp.path().to_path_buf(),
        vars: HashMap::new(),
    };
    let step = make_command_step("echo hello");
    let result = execute_step(&step, &ctx);
    assert!(result.is_ok());
}

#[test]
fn test_execute_command_with_working_dir() {
    let tmp = TempDir::new().unwrap();
    let sub = tmp.path().join("subdir");
    std::fs::create_dir(&sub).unwrap();

    let ctx = StepContext {
        project_dir: tmp.path().to_path_buf(),
        vars: HashMap::new(),
    };
    let step: StepDef = toml::from_str(
        r#"
type = "command"
command = "pwd"
working_dir = "subdir"
"#,
    )
    .unwrap();
    let result = execute_step(&step, &ctx);
    assert!(result.is_ok());
}

#[test]
fn test_execute_failing_command() {
    let tmp = TempDir::new().unwrap();
    let ctx = StepContext {
        project_dir: tmp.path().to_path_buf(),
        vars: HashMap::new(),
    };
    let step = make_command_step("false");
    let result = execute_step(&step, &ctx);
    assert!(result.is_err());
}

#[test]
fn test_execute_with_check_skips() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("exists.txt"), "hi").unwrap();

    let ctx = StepContext {
        project_dir: tmp.path().to_path_buf(),
        vars: HashMap::new(),
    };
    let step: StepDef = toml::from_str(
        r#"
type = "command"
check = "test -f exists.txt"
command = "echo should-be-skipped"
"#,
    )
    .unwrap();
    let result = execute_step(&step, &ctx);
    assert!(result.is_ok());
}
