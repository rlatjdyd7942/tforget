use tforge::types::{Provider, TemplateManifest};

#[test]
fn test_deserialize_command_template() {
    let toml_str = r#"
[template]
name = "flutter-app"
description = "Flutter mobile application"
category = "mobile"
provider = "command"

[dependencies]
required_tools = ["flutter"]

[parameters]
org = { type = "string", prompt = "Organization name", default = "com.example" }

[[steps]]
type = "command"
command = "flutter create --org {{org}} {{project_name}}"
"#;
    let manifest: TemplateManifest = toml::from_str(toml_str).unwrap();
    assert_eq!(manifest.template.name, "flutter-app");
    assert_eq!(manifest.template.provider, Provider::Command);
    assert_eq!(manifest.dependencies.required_tools, vec!["flutter"]);
    assert_eq!(manifest.steps.len(), 1);
    assert!(manifest.parameters.contains_key("org"));
}

#[test]
fn test_deserialize_template_with_conditions() {
    let toml_str = r#"
[template]
name = "firebase-flutter"
description = "Firebase for Flutter"
category = "integration"
provider = "command"

[dependencies]
required_tools = ["firebase"]
requires_templates = ["flutter-app"]

[parameters]
services = { type = "multi-select", prompt = "Services?", options = ["crashlytics", "auth"], default = ["crashlytics"] }

[[steps]]
type = "command"
command = "flutter pub add firebase_crashlytics"
condition = "services contains 'crashlytics'"
"#;
    let manifest: TemplateManifest = toml::from_str(toml_str).unwrap();
    assert_eq!(
        manifest.dependencies.requires_templates,
        vec!["flutter-app"]
    );
    assert!(manifest.steps[0].condition.is_some());
}

#[test]
fn test_deserialize_parameter_when_condition() {
    let toml_str = r#"
[template]
name = "gcp-appengine"
description = "App Engine"
category = "cloud"
provider = "command"

[dependencies]

[parameters]
appengine_environment = { type = "select", prompt = "Environment", options = ["standard", "flexible"], default = "standard" }
runtime_standard = { type = "select", prompt = "Runtime", options = ["python312"], when = "appengine_environment == 'standard'" }

[[steps]]
type = "command"
command = "echo ok"
"#;

    let manifest: TemplateManifest = toml::from_str(toml_str).unwrap();
    let runtime_standard = manifest.parameters.get("runtime_standard").unwrap();
    assert_eq!(
        runtime_standard.when.as_deref(),
        Some("appengine_environment == 'standard'")
    );
}

#[test]
fn test_deserialize_bundled_step() {
    let toml_str = r#"
[template]
name = "test"
description = "test"
category = "test"
provider = "bundled"

[dependencies]

[[steps]]
type = "bundled"
action = "overlay"
source = "files/"
"#;
    let manifest: TemplateManifest = toml::from_str(toml_str).unwrap();
    assert_eq!(manifest.steps[0].step_type, "bundled");
    assert_eq!(manifest.steps[0].action.as_deref(), Some("overlay"));
}
