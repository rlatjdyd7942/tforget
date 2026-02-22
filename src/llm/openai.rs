use crate::config::LlmConfig;
use anyhow::{bail, Context, Result};
use serde_json::json;

pub async fn query(config: &LlmConfig, system: &str, user_msg: &str) -> Result<String> {
    let api_key = config
        .api_key_env
        .as_ref()
        .and_then(|env_var| std::env::var(env_var).ok())
        .unwrap_or_default();

    let endpoint = config
        .endpoint
        .as_deref()
        .unwrap_or("https://api.openai.com/v1");

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{endpoint}/chat/completions"))
        .header("Authorization", format!("Bearer {api_key}"))
        .header("content-type", "application/json")
        .json(&json!({
            "model": config.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user_msg}
            ]
        }))
        .send()
        .await
        .context("failed to call OpenAI-compatible API")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("API error ({status}): {body}");
    }

    let body: serde_json::Value = resp.json().await?;
    let text = body["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("unexpected response format"))?;
    Ok(text.to_string())
}
