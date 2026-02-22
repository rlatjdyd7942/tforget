use tforge::config::{TforgeConfig, LlmConfig, LlmProvider};
use tempfile::TempDir;

#[test]
fn test_default_config() {
    let config = TforgeConfig::default();
    assert!(config.llm.is_none());
}

#[test]
fn test_save_and_load_config() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("config.toml");

    let config = TforgeConfig {
        llm: Some(LlmConfig {
            provider: LlmProvider::Anthropic,
            model: "claude-sonnet-4-6".into(),
            api_key_env: Some("ANTHROPIC_API_KEY".into()),
            endpoint: None,
        }),
    };
    config.save(&config_path).unwrap();

    let loaded = TforgeConfig::load(&config_path).unwrap();
    assert!(loaded.llm.is_some());
    let llm = loaded.llm.unwrap();
    assert_eq!(llm.provider, LlmProvider::Anthropic);
    assert_eq!(llm.model, "claude-sonnet-4-6");
}

#[test]
fn test_load_nonexistent_returns_default() {
    let tmp = TempDir::new().unwrap();
    let config_path = tmp.path().join("nonexistent.toml");
    let config = TforgeConfig::load(&config_path).unwrap();
    assert!(config.llm.is_none());
}
