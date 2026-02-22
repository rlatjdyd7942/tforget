use tforge::remote::{cache_dir, list_cached_templates_in, search_templates};
use tforge::registry::Registry;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_cache_dir_path() {
    let dir = cache_dir();
    assert!(dir.ends_with("tforge/templates"));
}

#[test]
fn test_list_cached_templates_empty_dir() {
    let tmp = TempDir::new().unwrap();
    let templates = list_cached_templates_in(tmp.path()).unwrap();
    assert!(templates.is_empty());
}

#[test]
fn test_list_cached_templates_nonexistent_dir() {
    let tmp = TempDir::new().unwrap();
    let nonexistent = tmp.path().join("nonexistent");
    let templates = list_cached_templates_in(&nonexistent).unwrap();
    assert!(templates.is_empty());
}

#[test]
fn test_list_cached_templates_with_manifest() {
    let tmp = TempDir::new().unwrap();
    let template_dir = tmp.path().join("my-template");
    fs::create_dir(&template_dir).unwrap();
    fs::write(
        template_dir.join("template.toml"),
        r#"
[template]
name = "my-template"
description = "A test template"
category = "test"
provider = "command"

[dependencies]
required_tools = []

[[steps]]
type = "command"
command = "echo hello"
"#,
    )
    .unwrap();

    let templates = list_cached_templates_in(tmp.path()).unwrap();
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].template.name, "my-template");
}

#[test]
fn test_search_templates_matches() {
    let registry = Registry::from_embedded().unwrap();
    let results = search_templates(&registry, "flutter");
    assert!(!results.is_empty());
    assert!(results.iter().any(|t| t.template.name.contains("flutter")));
}

#[test]
fn test_search_templates_case_insensitive() {
    let registry = Registry::from_embedded().unwrap();
    let results = search_templates(&registry, "FLUTTER");
    assert!(!results.is_empty());
}

#[test]
fn test_search_templates_no_match() {
    let registry = Registry::from_embedded().unwrap();
    let results = search_templates(&registry, "zzz_nonexistent_zzz");
    assert!(results.is_empty());
}
