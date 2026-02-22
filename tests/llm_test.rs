use tforge::llm::parse_llm_recipe_response;

#[test]
fn test_parse_llm_recipe_response() {
    let json = r#"{
        "templates": ["flutter-app", "axum-server", "gcp-project", "firebase-flutter"],
        "parameters": {
            "org": "com.example",
            "gcp_project_id": "my-app-prod",
            "region": "us-central1",
            "services": "crashlytics,analytics"
        }
    }"#;

    let recipe = parse_llm_recipe_response(json).unwrap();
    assert_eq!(recipe.templates, vec!["flutter-app", "axum-server", "gcp-project", "firebase-flutter"]);
    assert_eq!(recipe.parameters.get("org").unwrap(), "com.example");
}

#[test]
fn test_parse_invalid_json() {
    let result = parse_llm_recipe_response("not json");
    assert!(result.is_err());
}
