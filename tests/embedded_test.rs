use tforge::embedded::load_embedded_templates;
use tforge::registry::Registry;

#[test]
fn test_embedded_templates_load() {
    let templates = load_embedded_templates().unwrap();
    assert!(
        templates.len() >= 7,
        "expected at least 7 embedded templates, got {}",
        templates.len()
    );
}

#[test]
fn test_embedded_templates_include_known_names() {
    let templates = load_embedded_templates().unwrap();
    let names: Vec<&str> = templates.iter().map(|t| t.template.name.as_str()).collect();
    assert!(names.contains(&"flutter-app"), "missing flutter-app");
    assert!(names.contains(&"axum-server"), "missing axum-server");
    assert!(names.contains(&"gcp-project"), "missing gcp-project");
}

#[test]
fn test_registry_from_embedded() {
    let registry = Registry::from_embedded().unwrap();
    assert!(registry.templates().len() >= 7);
    assert!(registry.find("flutter-app").is_some());
}

#[test]
fn test_registry_merge_deduplicates() {
    let mut reg1 = Registry::from_embedded().unwrap();
    let reg2 = Registry::from_embedded().unwrap();
    let count_before = reg1.templates().len();
    reg1.merge(reg2);
    assert_eq!(
        reg1.templates().len(),
        count_before,
        "merge should deduplicate"
    );
}
