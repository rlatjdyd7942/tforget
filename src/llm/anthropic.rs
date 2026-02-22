use crate::config::LlmConfig;
use anyhow::{bail, Context, Result};
use serde_json::json;

pub async fn query(config: &LlmConfig, system: &str, user_msg: &str) -> Result<String> {
    let api_key = config
        .api_key_env
        .as_ref()
        .and_then(|env_var| std::env::var(env_var).ok())
        .ok_or_else(|| anyhow::anyhow!("API key not found. Run `tforge config llm` to configure."))?;

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&json!({
            "model": config.model,
            "max_tokens": 1024,
            "system": system,
            "messages": [{"role": "user", "content": user_msg}]
        }))
        .send()
        .await
        .context("failed to call Anthropic API")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("Anthropic API error ({status}): {body}");
    }

    let body: serde_json::Value = resp.json().await?;
    let text = body["content"][0]["text"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("unexpected Anthropic response format"))?;
    Ok(text.to_string())
}
