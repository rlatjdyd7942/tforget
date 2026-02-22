use tforge::registry::Registry;
use std::path::PathBuf;

#[test]
fn test_load_templates_from_directory() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/templates");
    let registry = Registry::from_directory(&fixtures).unwrap();
    assert!(registry.templates().len() >= 1);
}

#[test]
fn test_find_template_by_name() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/templates");
    let registry = Registry::from_directory(&fixtures).unwrap();
    let tmpl = registry.find("test-app");
    assert!(tmpl.is_some());
    assert_eq!(tmpl.unwrap().template.name, "test-app");
}

#[test]
fn test_find_nonexistent_template() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/templates");
    let registry = Registry::from_directory(&fixtures).unwrap();
    assert!(registry.find("nonexistent").is_none());
}

#[test]
fn test_filter_by_category() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/templates");
    let registry = Registry::from_directory(&fixtures).unwrap();
    let mobile = registry.by_category("mobile");
    assert!(mobile.iter().all(|t| t.template.category == "mobile"));
}
