use std::collections::HashMap;
use tforge::prompts::{parameter_keys_in_prompt_order, should_prompt_parameter};
use tforge::types::TemplateManifest;

#[test]
fn test_parameter_prompt_order_is_lexical() {
    let manifest: TemplateManifest = toml::from_str(
        r#"
[template]
name = "ordered"
description = "ordered"
category = "test"
provider = "command"

[dependencies]

[parameters]
zeta = { type = "string", prompt = "z" }
alpha = { type = "string", prompt = "a" }
beta = { type = "string", prompt = "b" }

[[steps]]
type = "command"
command = "echo ok"
"#,
    )
    .unwrap();

    assert_eq!(
        parameter_keys_in_prompt_order(&manifest),
        vec!["alpha", "beta", "zeta"]
    );
}

#[test]
fn test_prompt_condition_respects_current_vars() {
    let manifest: TemplateManifest = toml::from_str(
        r#"
[template]
name = "gcp-appengine"
description = "App Engine"
category = "cloud"
provider = "command"

[dependencies]

[parameters]
deploy_now = { type = "bool", prompt = "Deploy now?", default = false }
promote_traffic = { type = "bool", prompt = "Promote traffic?", default = true, when = "deploy_now == 'true'" }

[[steps]]
type = "command"
command = "echo ok"
"#,
    )
    .unwrap();

    let promote_traffic = manifest.parameters.get("promote_traffic").unwrap();

    let mut vars = HashMap::new();
    vars.insert("deploy_now".to_string(), "false".to_string());
    assert!(
        !should_prompt_parameter(
            &manifest.template.name,
            "promote_traffic",
            promote_traffic,
            &vars
        )
        .unwrap()
    );

    vars.insert("deploy_now".to_string(), "true".to_string());
    assert!(
        should_prompt_parameter(
            &manifest.template.name,
            "promote_traffic",
            promote_traffic,
            &vars
        )
        .unwrap()
    );
}

#[test]
fn test_prompt_condition_missing_variable_returns_contextual_error() {
    let manifest: TemplateManifest = toml::from_str(
        r#"
[template]
name = "gcp-appengine"
description = "App Engine"
category = "cloud"
provider = "command"

[dependencies]

[parameters]
runtime_standard = { type = "select", prompt = "Runtime", options = ["python312"], when = "appengine_environment == 'standard'" }

[[steps]]
type = "command"
command = "echo ok"
"#,
    )
    .unwrap();

    let runtime_standard = manifest.parameters.get("runtime_standard").unwrap();
    let err = should_prompt_parameter(
        &manifest.template.name,
        "runtime_standard",
        runtime_standard,
        &HashMap::new(),
    )
    .unwrap_err();
    let err_msg = format!("{err:#}");

    assert!(err_msg.contains("template 'gcp-appengine' parameter 'runtime_standard'"));
    assert!(err_msg.contains("variable 'appengine_environment' not found"));
}
