use std::path::PathBuf;
use tforge::registry::Registry;

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

#[test]
fn test_embedded_gcp_appengine_template_contract() {
    let registry = Registry::from_embedded().unwrap();
    let template = registry
        .find("gcp-appengine")
        .expect("missing gcp-appengine");

    assert!(
        template
            .dependencies
            .requires_templates
            .contains(&"gcp-project".to_string())
    );

    for key in [
        "deploy_target",
        "appengine_environment",
        "service",
        "version",
        "deploy_now",
        "runtime_standard",
        "runtime_flexible",
    ] {
        assert!(
            template.parameters.contains_key(key),
            "missing expected parameter '{key}'"
        );
    }

    assert_eq!(
        template
            .parameters
            .get("deploy_target_path")
            .unwrap()
            .when
            .as_deref(),
        Some("deploy_target == 'custom-path'")
    );
    assert_eq!(
        template
            .parameters
            .get("runtime_standard")
            .unwrap()
            .when
            .as_deref(),
        Some("appengine_environment == 'standard'")
    );
    assert_eq!(
        template
            .parameters
            .get("runtime_flexible")
            .unwrap()
            .when
            .as_deref(),
        Some("appengine_environment == 'flexible'")
    );

    assert!(
        template
            .steps
            .iter()
            .any(|s| s.condition.as_deref() == Some("appengine_environment == 'standard'"))
    );
    assert!(
        template
            .steps
            .iter()
            .any(|s| s.condition.as_deref() == Some("appengine_environment == 'flexible'"))
    );
    assert!(
        template
            .steps
            .iter()
            .any(|s| s.condition.as_deref() == Some("deploy_now == 'true'"))
    );
}
