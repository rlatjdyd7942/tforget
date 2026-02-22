pub mod anthropic;
pub mod openai;

use crate::config::{LlmConfig, LlmProvider};
use crate::registry::Registry;
use anyhow::Result;
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

pub async fn query_llm(config: &LlmConfig, system: &str, user_msg: &str) -> Result<String> {
    match config.provider {
        LlmProvider::Anthropic => anthropic::query(config, system, user_msg).await,
        LlmProvider::Openai | LlmProvider::Gemini => openai::query(config, system, user_msg).await,
        LlmProvider::Ollama => openai::query(config, system, user_msg).await,
    }
}
