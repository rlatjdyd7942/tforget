use std::collections::HashMap;
use tforge::renderer::Renderer;

#[test]
fn test_render_simple_variable() {
    let renderer = Renderer::new();
    let mut vars = HashMap::new();
    vars.insert("project_name".into(), "my-app".into());
    let result = renderer
        .render_string("hello {{project_name}}", &vars)
        .unwrap();
    assert_eq!(result, "hello my-app");
}

#[test]
fn test_render_multiple_variables() {
    let renderer = Renderer::new();
    let mut vars = HashMap::new();
    vars.insert("org".into(), "com.example".into());
    vars.insert("project_name".into(), "my-app".into());
    let result = renderer
        .render_string("flutter create --org {{org}} {{project_name}}", &vars)
        .unwrap();
    assert_eq!(result, "flutter create --org com.example my-app");
}

#[test]
fn test_render_with_join_filter() {
    let renderer = Renderer::new();
    let mut vars = HashMap::new();
    vars.insert("platforms".into(), "ios,android".into());
    let result = renderer
        .render_string("--platforms {{platforms}}", &vars)
        .unwrap();
    assert_eq!(result, "--platforms ios,android");
}

#[test]
fn test_render_missing_variable_errors() {
    let renderer = Renderer::new();
    let vars = HashMap::new();
    let result = renderer.render_string("hello {{missing}}", &vars);
    assert!(result.is_err());
}
