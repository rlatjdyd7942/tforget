use tforge::resolver::resolve_order;
use tforge::types::TemplateManifest;

fn make_manifest(name: &str, requires: Vec<&str>) -> TemplateManifest {
    let toml_str = format!(
        r#"
[template]
name = "{name}"
description = "test"
category = "test"
provider = "command"

[dependencies]
requires_templates = [{requires}]

[[steps]]
type = "command"
command = "echo {name}"
"#,
        name = name,
        requires = requires
            .iter()
            .map(|r| format!("\"{r}\""))
            .collect::<Vec<_>>()
            .join(", ")
    );
    toml::from_str(&toml_str).unwrap()
}

#[test]
fn test_no_dependencies() {
    let templates = vec![make_manifest("a", vec![])];
    let order = resolve_order(&templates).unwrap();
    assert_eq!(order, vec!["a"]);
}

#[test]
fn test_simple_dependency() {
    let templates = vec![
        make_manifest("firebase-flutter", vec!["flutter-app"]),
        make_manifest("flutter-app", vec![]),
    ];
    let order = resolve_order(&templates).unwrap();
    let a_pos = order.iter().position(|n| n == "flutter-app").unwrap();
    let b_pos = order.iter().position(|n| n == "firebase-flutter").unwrap();
    assert!(a_pos < b_pos);
}

#[test]
fn test_chain_dependency() {
    let templates = vec![
        make_manifest("gcp-cloudsql", vec!["gcp-project"]),
        make_manifest("gcp-project", vec![]),
        make_manifest("axum-server", vec![]),
    ];
    let order = resolve_order(&templates).unwrap();
    let proj_pos = order.iter().position(|n| n == "gcp-project").unwrap();
    let sql_pos = order.iter().position(|n| n == "gcp-cloudsql").unwrap();
    assert!(proj_pos < sql_pos);
}

#[test]
fn test_circular_dependency_errors() {
    let templates = vec![make_manifest("a", vec!["b"]), make_manifest("b", vec!["a"])];
    let result = resolve_order(&templates);
    assert!(result.is_err());
}
