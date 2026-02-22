use crate::condition::evaluate_condition;
use crate::registry::Registry;
use crate::types::{ParamDef, ParamType, TemplateManifest};
use anyhow::{Context, Result};
use inquire::{Confirm, MultiSelect, Select, Text};
use std::collections::HashMap;

pub struct RecipeSelection {
    pub templates: Vec<TemplateManifest>,
    pub vars: HashMap<String, String>,
}

pub fn parameter_keys_in_prompt_order(template: &TemplateManifest) -> Vec<String> {
    let mut keys: Vec<String> = template.parameters.keys().cloned().collect();
    keys.sort();
    keys
}

pub fn should_prompt_parameter(
    template_name: &str,
    param_key: &str,
    param: &ParamDef,
    vars: &HashMap<String, String>,
) -> Result<bool> {
    let Some(condition) = &param.when else {
        return Ok(true);
    };

    evaluate_condition(condition, vars).with_context(|| {
        format!(
            "failed to evaluate prompt condition for template '{template_name}' parameter '{param_key}'"
        )
    })
}

pub fn prompt_recipe(registry: &Registry, project_name: &str) -> Result<RecipeSelection> {
    let mut selected_templates: Vec<TemplateManifest> = Vec::new();
    let mut vars = HashMap::new();

    vars.insert("project_name".into(), project_name.into());

    // Step 1: Select primary templates by category
    let categories = registry.categories();
    for category in &categories {
        if category == "integration" {
            continue;
        }

        let templates = registry.by_category(category);
        if templates.is_empty() {
            continue;
        }

        let names: Vec<String> = templates
            .iter()
            .map(|t| format!("{} — {}", t.template.name, t.template.description))
            .collect();

        let selections = MultiSelect::new(&format!("Select {category} templates:"), names.clone())
            .prompt()
            .context("template selection cancelled")?;

        for selection in &selections {
            let idx = names.iter().position(|n| n == selection).unwrap();
            let template = templates[idx];
            if selected_templates
                .iter()
                .all(|t| t.template.name != template.template.name)
            {
                selected_templates.push(template.clone());
            }
        }
    }

    // Step 2: Check for available integrations
    let integration_templates: Vec<&TemplateManifest> = registry
        .by_category("integration")
        .into_iter()
        .filter(|t| {
            t.dependencies
                .requires_templates
                .iter()
                .all(|req| selected_templates.iter().any(|s| s.template.name == *req))
        })
        .collect();

    if !integration_templates.is_empty() {
        let names: Vec<String> = integration_templates
            .iter()
            .map(|t| format!("{} — {}", t.template.name, t.template.description))
            .collect();

        let selections = MultiSelect::new("Add integrations?", names.clone())
            .prompt()
            .context("integration selection cancelled")?;

        for selection in &selections {
            let idx = names.iter().position(|n| n == selection).unwrap();
            let template = integration_templates[idx];
            if selected_templates
                .iter()
                .all(|t| t.template.name != template.template.name)
            {
                selected_templates.push(template.clone());
            }
        }
    }

    // Step 3: Collect parameters for all selected templates
    for tmpl in &selected_templates {
        for key in parameter_keys_in_prompt_order(tmpl) {
            if vars.contains_key(&key) {
                continue;
            }

            let param = tmpl.parameters.get(&key).expect("parameter key must exist");
            if !should_prompt_parameter(&tmpl.template.name, &key, param, &vars)? {
                continue;
            }

            let value = match &param.param_type {
                ParamType::String => {
                    let mut prompt = Text::new(&param.prompt);
                    if let Some(toml::Value::String(d)) = &param.default {
                        prompt = prompt.with_default(d);
                    }
                    prompt.prompt().context("input cancelled")?
                }
                ParamType::Select => {
                    let selected = Select::new(&param.prompt, param.options.clone())
                        .prompt()
                        .context("selection cancelled")?;
                    selected
                }
                ParamType::MultiSelect => {
                    let selected = MultiSelect::new(&param.prompt, param.options.clone())
                        .prompt()
                        .context("selection cancelled")?;
                    selected.join(",")
                }
                ParamType::Bool => {
                    let default_val = param
                        .default
                        .as_ref()
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let result = Confirm::new(&param.prompt)
                        .with_default(default_val)
                        .prompt()
                        .context("confirm cancelled")?;
                    result.to_string()
                }
            };

            vars.insert(key, value);
        }
    }

    Ok(RecipeSelection {
        templates: selected_templates,
        vars,
    })
}
