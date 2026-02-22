use tforge::engine::Engine;
use tforge::types::TemplateManifest;
use std::collections::HashMap;
use tempfile::TempDir;

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
