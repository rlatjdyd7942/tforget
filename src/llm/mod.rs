use crate::config::{LlmConfig, LlmProvider};
use crate::registry::Registry;
use anyhow::{Context, Result};
use rig::client::CompletionClient;
use rig::completion::Prompt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmRecipe {
    pub templates: Vec<String>,
    pub parameters: HashMap<String, String>,
}

pub fn parse_llm_recipe_response(json: &str) -> Result<LlmRecipe> {
    let recipe: LlmRecipe = serde_json::from_str(json)?;
    Ok(recipe)
}

pub fn build_system_prompt(registry: &Registry) -> String {
    let mut prompt = String::from(
        "You are a project configuration assistant for tforge. \
         Given a user's description of what they want to build, \
         respond with a JSON object containing the template selections and parameter values.\n\n\
         Available templates:\n",
    );

    for tmpl in registry.templates() {
        prompt.push_str(&format!(
            "- {} ({}): {}\n",
            tmpl.template.name, tmpl.template.category, tmpl.template.description
        ));
        for (key, param) in &tmpl.parameters {
            prompt.push_str(&format!("  param '{}': {} ", key, param.prompt));
            if !param.options.is_empty() {
                prompt.push_str(&format!("options: [{}] ", param.options.join(", ")));
            }
            prompt.push('\n');
        }
        if !tmpl.dependencies.requires_templates.is_empty() {
            prompt.push_str(&format!(
                "  requires: [{}]\n",
                tmpl.dependencies.requires_templates.join(", ")
            ));
        }
    }

    prompt.push_str(
        "\nRespond ONLY with a JSON object: {\"templates\": [...], \"parameters\": {...}}\n",
    );
    prompt
}

fn resolve_api_key(config: &LlmConfig) -> Result<String> {
    let env_var = config
        .api_key_env
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("API key env var not configured. Run `tforge config llm`."))?;
    std::env::var(env_var)
        .with_context(|| format!("API key not found in environment variable '{env_var}'"))
}

pub async fn query_llm(config: &LlmConfig, system: &str, user_msg: &str) -> Result<String> {
    match config.provider {
        LlmProvider::Anthropic => {
            let api_key = resolve_api_key(config)?;
            let builder = rig::providers::anthropic::Client::builder().api_key(api_key);
            let builder = if let Some(endpoint) = &config.endpoint {
                builder.base_url(endpoint)
            } else {
                builder
            };
            let client: rig::providers::anthropic::Client =
                builder.build().context("failed to build Anthropic client")?;
            let agent = client.agent(&config.model).preamble(system).build();
            agent
                .prompt(user_msg)
                .await
                .context("Anthropic API call failed")
        }
        LlmProvider::Openai | LlmProvider::Gemini => {
            let api_key = resolve_api_key(config)?;
            let builder =
                rig::providers::openai::CompletionsClient::builder().api_key(api_key);
            let builder = if let Some(endpoint) = &config.endpoint {
                builder.base_url(endpoint)
            } else {
                builder
            };
            let client: rig::providers::openai::CompletionsClient =
                builder.build().context("failed to build OpenAI-compatible client")?;
            let agent = client.agent(&config.model).preamble(system).build();
            agent
                .prompt(user_msg)
                .await
                .context("OpenAI-compatible API call failed")
        }
        LlmProvider::Ollama => {
            let builder =
                rig::providers::ollama::Client::builder().api_key(rig::client::Nothing);
            let builder = if let Some(endpoint) = &config.endpoint {
                builder.base_url(endpoint)
            } else {
                builder
            };
            let client: rig::providers::ollama::Client =
                builder.build().context("failed to build Ollama client")?;
            let agent = client.agent(&config.model).preamble(system).build();
            agent
                .prompt(user_msg)
                .await
                .context("Ollama API call failed")
        }
    }
}
