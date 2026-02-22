use std::collections::HashMap;
use tempfile::TempDir;
use tforge::engine::Engine;
use tforge::types::TemplateManifest;

#[test]
fn test_engine_runs_single_template() {
    let tmp = TempDir::new().unwrap();
    let manifest: TemplateManifest = toml::from_str(
        r#"
[template]
name = "test"
description = "test"
category = "test"
provider = "command"

[dependencies]

[[steps]]
type = "command"
command = "mkdir -p app && echo 'hello' > app/README.md"
"#,
    )
    .unwrap();

    let mut vars = HashMap::new();
    vars.insert("project_name".into(), "test-project".into());

    let engine = Engine::new(tmp.path().to_path_buf());
    engine.run(&[manifest], &vars).unwrap();

    assert!(tmp.path().join("app/README.md").exists());
}

#[test]
fn test_engine_respects_dependency_order() {
    let tmp = TempDir::new().unwrap();

    let first: TemplateManifest = toml::from_str(
        r#"
[template]
name = "base"
description = "base"
category = "test"
provider = "command"
[dependencies]
[[steps]]
type = "command"
command = "echo 'first' > order.txt"
"#,
    )
    .unwrap();

    let second: TemplateManifest = toml::from_str(
        r#"
[template]
name = "addon"
description = "addon"
category = "test"
provider = "command"
[dependencies]
requires_templates = ["base"]
[[steps]]
type = "command"
command = "echo 'second' >> order.txt"
"#,
    )
    .unwrap();

    let vars = HashMap::new();
    let engine = Engine::new(tmp.path().to_path_buf());
    engine.run(&[second, first], &vars).unwrap();

    let content = std::fs::read_to_string(tmp.path().join("order.txt")).unwrap();
    assert!(content.contains("first"));
    assert!(content.contains("second"));
}

#[test]
fn test_engine_skips_conditional_steps() {
    let tmp = TempDir::new().unwrap();
    let manifest: TemplateManifest = toml::from_str(
        r#"
[template]
name = "cond"
description = "test"
category = "test"
provider = "command"
[dependencies]

[parameters]
services = { type = "multi-select", prompt = "?", options = ["a", "b"] }

[[steps]]
type = "command"
command = "touch a.txt"
condition = "services contains 'a'"

[[steps]]
type = "command"
command = "touch b.txt"
condition = "services contains 'b'"
"#,
    )
    .unwrap();

    let mut vars = HashMap::new();
    vars.insert("services".into(), "a".into());

    let engine = Engine::new(tmp.path().to_path_buf());
    engine.run(&[manifest], &vars).unwrap();

    assert!(tmp.path().join("a.txt").exists());
    assert!(!tmp.path().join("b.txt").exists());
}

#[test]
fn test_engine_maps_appengine_target_directory() {
    let tmp = TempDir::new().unwrap();
    let manifest: TemplateManifest = toml::from_str(
        r#"
[template]
name = "appengine-target"
description = "test"
category = "cloud"
provider = "command"
[dependencies]

[[steps]]
type = "command"
command = """
TARGET_DIR="."
if [ "{{deploy_target}}" = "flutter-app" ]; then
  TARGET_DIR="{{project_name}}"
elif [ "{{deploy_target}}" = "axum-server" ]; then
  TARGET_DIR="{{project_name}}-server"
elif [ "{{deploy_target}}" = "custom-path" ]; then
  TARGET_DIR="{{deploy_target_path}}"
fi
mkdir -p "$TARGET_DIR"
echo "runtime: python312" > "$TARGET_DIR/app.yaml"
"""
"#,
    )
    .unwrap();

    let mut vars = HashMap::new();
    vars.insert("project_name".into(), "demo".into());
    vars.insert("deploy_target".into(), "axum-server".into());
    vars.insert("deploy_target_path".into(), "custom/api".into());

    let engine = Engine::new(tmp.path().to_path_buf());
    engine.run(&[manifest], &vars).unwrap();

    assert!(tmp.path().join("demo-server/app.yaml").exists());
}

#[test]
fn test_engine_respects_deploy_profile_condition() {
    let manifest: TemplateManifest = toml::from_str(
        r#"
[template]
name = "appengine-deploy"
description = "test"
category = "cloud"
provider = "command"
[dependencies]

[[steps]]
type = "command"
condition = "deploy_now == 'true'"
command = "touch deployed.txt"
"#,
    )
    .unwrap();

    let tmp_false = TempDir::new().unwrap();
    let mut vars_false = HashMap::new();
    vars_false.insert("deploy_now".into(), "false".into());
    Engine::new(tmp_false.path().to_path_buf())
        .run(&[manifest.clone()], &vars_false)
        .unwrap();
    assert!(!tmp_false.path().join("deployed.txt").exists());

    let tmp_true = TempDir::new().unwrap();
    let mut vars_true = HashMap::new();
    vars_true.insert("deploy_now".into(), "true".into());
    Engine::new(tmp_true.path().to_path_buf())
        .run(&[manifest], &vars_true)
        .unwrap();
    assert!(tmp_true.path().join("deployed.txt").exists());
}
