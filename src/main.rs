use anyhow::{Context, Result, bail};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Confirm, Select, Text};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tforge::cli::{Cli, Commands};
use tforge::config::{LlmConfig, LlmProvider, TforgeConfig};
use tforge::engine::Engine;
use tforge::llm::{build_system_prompt, parse_llm_recipe_response, query_llm};
use tforge::prompts::{RecipeSelection, prompt_recipe};
use tforge::registry::Registry;
use tforge::state::{PipelineState, StepState};
use tforge::toolcheck::{ToolStatus, check_tool, install_hint};
use tforge::types::TemplateManifest;

const TEMPLATE_ROOT: &str = "templates";
const STATE_FILE: &str = ".tforge-state.json";
const RECIPE_FILE: &str = "tforge.toml";

#[derive(Debug, Serialize, Deserialize)]
struct SavedRecipe {
    project_name: String,
    templates: Vec<String>,
    #[serde(default)]
    parameters: HashMap<String, String>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{}", format_error_chain(&err).red());
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, ai } => run_new(&name, ai.as_deref()).await,
        Commands::List => run_list(),
        Commands::Search { query } => run_search(&query),
        Commands::Add { url } => run_add(&url),
        Commands::Resume => run_resume(),
        Commands::Status => run_status(),
        Commands::Update => run_update(),
        Commands::Config { target, show } => run_config(&target, show),
    }
}

async fn run_new(project_name: &str, ai: Option<&str>) -> Result<()> {
    let registry = load_registry()?;

    let mut selection = match ai {
        Some(prompt) => select_recipe_with_ai(&registry, project_name, prompt).await?,
        None => prompt_recipe(&registry, project_name)?,
    };
    selection
        .vars
        .insert("project_name".to_string(), project_name.to_string());

    if selection.templates.is_empty() {
        bail!("No templates selected. Run `tforge new {project_name}` again.");
    }

    let templates = expand_required_templates(&selection.templates, &registry)?;
    ensure_tools_available(&templates)?;
    print_recipe_summary(project_name, &templates, &selection.vars);

    let confirmed = Confirm::new("Proceed with execution?")
        .with_default(true)
        .prompt()
        .context("execution confirmation cancelled")?;
    if !confirmed {
        println!("{}", "Aborted.".yellow());
        return Ok(());
    }

    let cwd = std::env::current_dir().context("failed to determine current directory")?;
    let state_path = cwd.join(STATE_FILE);
    let recipe_path = cwd.join(RECIPE_FILE);

    let saved_recipe = SavedRecipe {
        project_name: project_name.to_string(),
        templates: templates.iter().map(|t| t.template.name.clone()).collect(),
        parameters: selection.vars.clone(),
    };
    save_recipe(&saved_recipe, &recipe_path)?;

    let progress = spinner("Running template pipeline...");
    let run_result =
        Engine::new(cwd).run_with_state(&templates, &selection.vars, &state_path, false);
    match run_result {
        Ok(()) => {
            progress
                .finish_with_message(format!("Project '{project_name}' scaffolded successfully."));
            println!("Recipe saved: {}", recipe_path.display());
            println!("State saved: {}", state_path.display());
            Ok(())
        }
        Err(err) => {
            progress.abandon_with_message(
                "Pipeline failed. Run `tforge status` for details and `tforge resume` to retry."
                    .to_string(),
            );
            Err(err)
        }
    }
}

fn run_list() -> Result<()> {
    let registry = load_registry()?;

    println!("{}", "Available templates".bold());
    for category in registry.categories() {
        println!();
        println!("{} {}", category.bold(), "templates:".bold());
        for template in registry.by_category(&category) {
            println!(
                "  - {:<20} {}",
                template.template.name, template.template.description
            );
        }
    }

    Ok(())
}

fn run_search(query: &str) -> Result<()> {
    let registry = load_registry()?;
    let needle = query.to_lowercase();
    let mut matches = Vec::new();

    for template in registry.templates() {
        let name = template.template.name.to_lowercase();
        let category = template.template.category.to_lowercase();
        let description = template.template.description.to_lowercase();
        if name.contains(&needle) || category.contains(&needle) || description.contains(&needle) {
            matches.push(template);
        }
    }

    if matches.is_empty() {
        println!("No templates matched query '{query}'.");
        return Ok(());
    }

    println!("{}", "Search results".bold());
    for template in matches {
        println!(
            "  - {} ({}) — {}",
            template.template.name, template.template.category, template.template.description
        );
    }

    Ok(())
}

fn run_add(url: &str) -> Result<()> {
    let progress = spinner("Adding template...");
    match tforge::remote::add_template(url) {
        Ok(name) => {
            progress.finish_with_message(format!("Added template '{name}' to cache."));
            Ok(())
        }
        Err(err) => {
            progress.abandon_with_message("Failed to add template.".to_string());
            Err(err.context("failed to add template"))
        }
    }
}

fn run_update() -> Result<()> {
    let progress = spinner("Updating cached templates...");
    match tforge::remote::update_templates() {
        Ok(updated) => {
            if updated.is_empty() {
                progress.finish_with_message("No cached templates to update.");
            } else {
                progress.finish_with_message(format!("Updated {} template(s).", updated.len()));
                for name in &updated {
                    println!("  - {name}");
                }
            }
            Ok(())
        }
        Err(err) => {
            progress.abandon_with_message("Update failed.".to_string());
            Err(err.context("failed to update templates"))
        }
    }
}

fn run_resume() -> Result<()> {
    let cwd = std::env::current_dir().context("failed to determine current directory")?;
    let recipe_path = cwd.join(RECIPE_FILE);
    let state_path = cwd.join(STATE_FILE);
    let registry = load_registry()?;

    if !state_path.exists() {
        bail!(
            "No pipeline state found at {}. Run `tforge new <name>` first.",
            state_path.display()
        );
    }

    let recipe = load_recipe(&recipe_path)?;
    let templates = resolve_recipe_templates(&recipe.templates, &registry)?;
    let mut vars = recipe.parameters.clone();
    vars.insert("project_name".into(), recipe.project_name.clone());

    let progress = spinner("Resuming template pipeline...");
    let run_result = Engine::new(cwd).run_with_state(&templates, &vars, &state_path, true);
    match run_result {
        Ok(()) => {
            progress.finish_with_message("Resume completed successfully.");
            Ok(())
        }
        Err(err) => {
            progress.abandon_with_message(
                "Resume failed. See `tforge status` for details.".to_string(),
            );
            Err(err)
        }
    }
}

fn run_status() -> Result<()> {
    let cwd = std::env::current_dir().context("failed to determine current directory")?;
    let recipe_path = cwd.join(RECIPE_FILE);
    let state_path = cwd.join(STATE_FILE);

    if !recipe_path.exists() {
        println!("No active tforge project found in {}.", cwd.display());
        println!("Run `tforge new <name>` to start a project.");
        return Ok(());
    }

    let recipe = load_recipe(&recipe_path)?;
    let registry = load_registry()?;
    let state = PipelineState::load(&state_path)
        .with_context(|| format!("failed to load {}", state_path.display()))?;

    println!("{} {}", "Project:".bold(), recipe.project_name);
    println!("{} {}", "Recipe:".bold(), recipe_path.display());
    println!("{} {}", "State:".bold(), state_path.display());
    println!();
    println!("{}", "Template status".bold());

    for template_name in &recipe.templates {
        let Some(template) = registry.find(template_name) else {
            println!(
                "  - {}: template not found in local registry",
                template_name
            );
            continue;
        };

        let mut completed = 0usize;
        let mut pending = 0usize;
        let mut failed: Option<(usize, String)> = None;
        for idx in 0..template.steps.len() {
            match state.get(template_name, idx) {
                StepState::Completed => completed += 1,
                StepState::Pending => pending += 1,
                StepState::Failed(msg) => failed = Some((idx + 1, msg)),
            }
        }

        if let Some((step, msg)) = failed {
            println!(
                "  - {}: {} at step {} ({})",
                template_name,
                "failed".red(),
                step,
                msg
            );
        } else if pending == 0 {
            println!(
                "  - {}: {} ({}/{})",
                template_name,
                "complete".green(),
                completed,
                template.steps.len()
            );
        } else {
            println!(
                "  - {}: {} ({}/{})",
                template_name,
                "in progress".yellow(),
                completed,
                template.steps.len()
            );
        }
    }

    Ok(())
}

fn run_config(target: &str, show: bool) -> Result<()> {
    match target {
        "llm" => {
            if show {
                show_llm_config()
            } else {
                configure_llm()
            }
        }
        "reset" => reset_config(),
        other => bail!("unknown config target '{other}'. Use `llm` or `reset`."),
    }
}

fn show_llm_config() -> Result<()> {
    let path = TforgeConfig::default_path();
    let config = TforgeConfig::load(&path)
        .with_context(|| format!("failed to load config from {}", path.display()))?;

    let Some(llm) = config.llm else {
        println!("LLM config is not set.");
        return Ok(());
    };

    println!("{}", "LLM configuration".bold());
    println!("  provider: {}", provider_label(&llm.provider));
    println!("  model: {}", llm.model);
    println!(
        "  api_key_env: {}",
        llm.api_key_env.unwrap_or_else(|| "(none)".to_string())
    );
    println!(
        "  endpoint: {}",
        llm.endpoint.unwrap_or_else(|| "(default)".to_string())
    );

    Ok(())
}

fn configure_llm() -> Result<()> {
    let provider_choice = Select::new(
        "LLM provider:",
        vec!["anthropic", "openai", "gemini", "ollama"],
    )
    .prompt()
    .context("provider selection cancelled")?;

    let provider = parse_provider(provider_choice)?;
    let model = Text::new("Model:")
        .with_default(default_model_for(&provider))
        .prompt()
        .context("model input cancelled")?
        .trim()
        .to_string();

    let api_env_input = match default_api_key_env_for(&provider) {
        Some(default_env) => Text::new("API key env var (leave blank for none):")
            .with_default(default_env)
            .prompt()
            .context("API key env input cancelled")?,
        None => Text::new("API key env var (leave blank for none):")
            .prompt()
            .context("API key env input cancelled")?,
    };
    let api_key_env = normalized_opt(api_env_input);

    let endpoint_input = match default_endpoint_for(&provider) {
        Some(default_endpoint) => Text::new("Endpoint override (leave blank for default):")
            .with_default(default_endpoint)
            .prompt()
            .context("endpoint input cancelled")?,
        None => Text::new("Endpoint override (leave blank for default):")
            .prompt()
            .context("endpoint input cancelled")?,
    };
    let endpoint = normalized_opt(endpoint_input);

    let path = TforgeConfig::default_path();
    let mut config = TforgeConfig::load(&path)
        .with_context(|| format!("failed to load config from {}", path.display()))?;
    config.llm = Some(LlmConfig {
        provider,
        model,
        api_key_env,
        endpoint,
    });
    config
        .save(&path)
        .with_context(|| format!("failed to save config to {}", path.display()))?;

    println!("Saved LLM config: {}", path.display());
    Ok(())
}

fn reset_config() -> Result<()> {
    let path = TforgeConfig::default_path();
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("failed to remove {}", path.display()))?;
        println!("Removed config file: {}", path.display());
    } else {
        println!("Config file does not exist: {}", path.display());
    }
    Ok(())
}

fn load_registry() -> Result<Registry> {
    let progress = spinner("Loading template registry...");

    // Primary: embedded templates (bundled in binary)
    let mut registry = Registry::from_embedded().context("failed to load embedded templates")?;

    // Dev override: if local templates/ dir exists, merge those too
    let template_dir = Path::new(TEMPLATE_ROOT);
    if template_dir.exists() {
        if let Ok(local) = Registry::from_directory(template_dir) {
            registry.merge(local);
        }
    }

    // Merge cached remote templates
    if let Ok(cached) = Registry::from_cache_dir() {
        registry.merge(cached);
    }

    if registry.templates().is_empty() {
        progress.abandon_with_message("No templates found.".to_string());
        bail!("No templates found. Try running `tforge update` to fetch templates.");
    }

    progress.finish_with_message(format!(
        "Loaded {} template(s).",
        registry.templates().len()
    ));
    Ok(registry)
}

async fn select_recipe_with_ai(
    registry: &Registry,
    project_name: &str,
    prompt: &str,
) -> Result<RecipeSelection> {
    let config_path = TforgeConfig::default_path();
    let config = TforgeConfig::load(&config_path)
        .with_context(|| format!("failed to load config from {}", config_path.display()))?;
    let llm = config
        .llm
        .ok_or_else(|| anyhow::anyhow!("LLM is not configured. Run `tforge config llm` first."))?;

    let system_prompt = build_system_prompt(registry);
    let progress = spinner("Querying LLM for template selection...");
    let response = query_llm(&llm, &system_prompt, prompt).await;
    let response = match response {
        Ok(value) => {
            progress.finish_with_message("LLM recipe received.");
            value
        }
        Err(err) => {
            progress.abandon_with_message("LLM query failed.".to_string());
            return Err(err.context("failed to query LLM for recipe"));
        }
    };

    let parsed = parse_llm_recipe_response(&response).context(
        "LLM did not return a valid recipe JSON payload: {\"templates\": [...], \"parameters\": {...}}",
    )?;

    let mut templates = Vec::new();
    let mut seen = HashSet::new();
    for template_name in parsed.templates {
        if !seen.insert(template_name.clone()) {
            continue;
        }
        let template = registry
            .find(&template_name)
            .ok_or_else(|| anyhow::anyhow!("LLM selected unknown template '{template_name}'"))?;
        templates.push(template.clone());
    }

    let mut vars = parsed.parameters;
    vars.insert("project_name".to_string(), project_name.to_string());

    Ok(RecipeSelection { templates, vars })
}

fn ensure_tools_available(templates: &[TemplateManifest]) -> Result<()> {
    let mut required_tools = BTreeSet::new();
    for template in templates {
        for tool in &template.dependencies.required_tools {
            required_tools.insert(tool.clone());
        }
    }

    let mut missing = Vec::new();
    for tool in required_tools {
        if matches!(check_tool(&tool), ToolStatus::NotFound) {
            missing.push(tool);
        }
    }

    if missing.is_empty() {
        return Ok(());
    }

    eprintln!("{}", "Missing required tools:".red().bold());
    for tool in &missing {
        eprintln!("  - {}: {}", tool, install_hint(tool));
    }
    bail!("Install missing tools and run the command again.");
}

fn expand_required_templates(
    selected: &[TemplateManifest],
    registry: &Registry,
) -> Result<Vec<TemplateManifest>> {
    let mut ordered = Vec::new();
    let mut seen = HashSet::new();

    for template in selected {
        if seen.insert(template.template.name.clone()) {
            ordered.push(template.clone());
        }
    }

    let mut idx = 0;
    while idx < ordered.len() {
        let deps = ordered[idx].dependencies.requires_templates.clone();
        for dep in deps {
            if seen.insert(dep.clone()) {
                let dep_template = registry.find(&dep).ok_or_else(|| {
                    anyhow::anyhow!(
                        "template '{}' requires missing dependency template '{}'",
                        ordered[idx].template.name,
                        dep
                    )
                })?;
                ordered.push(dep_template.clone());
            }
        }
        idx += 1;
    }

    Ok(ordered)
}

fn resolve_recipe_templates(
    names: &[String],
    registry: &Registry,
) -> Result<Vec<TemplateManifest>> {
    let mut templates = Vec::new();
    let mut seen = HashSet::new();
    for name in names {
        if !seen.insert(name.clone()) {
            continue;
        }
        let template = registry.find(name).ok_or_else(|| {
            anyhow::anyhow!(
                "template '{name}' from recipe not found. Run `tforge list` to see available templates."
            )
        })?;
        templates.push(template.clone());
    }
    Ok(templates)
}

fn print_recipe_summary(
    project_name: &str,
    templates: &[TemplateManifest],
    vars: &HashMap<String, String>,
) {
    println!();
    println!("{} {}", "Project:".bold(), project_name);
    println!("{}", "Templates:".bold());
    for template in templates {
        println!(
            "  - {} ({})",
            template.template.name, template.template.category
        );
    }
    let param_count = vars.len().saturating_sub(1);
    println!("{} {}", "Parameters:".bold(), param_count);
}

fn save_recipe(recipe: &SavedRecipe, path: &Path) -> Result<()> {
    let content = toml::to_string_pretty(recipe).context("failed to serialize recipe")?;
    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))
}

fn load_recipe(path: &Path) -> Result<SavedRecipe> {
    if !path.exists() {
        bail!("No recipe file found at {}.", path.display());
    }

    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))
}

fn normalized_opt(input: String) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_provider(input: &str) -> Result<LlmProvider> {
    match input {
        "anthropic" => Ok(LlmProvider::Anthropic),
        "openai" => Ok(LlmProvider::Openai),
        "gemini" => Ok(LlmProvider::Gemini),
        "ollama" => Ok(LlmProvider::Ollama),
        other => bail!("unsupported provider '{other}'"),
    }
}

fn provider_label(provider: &LlmProvider) -> &'static str {
    match provider {
        LlmProvider::Anthropic => "anthropic",
        LlmProvider::Openai => "openai",
        LlmProvider::Gemini => "gemini",
        LlmProvider::Ollama => "ollama",
    }
}

fn default_model_for(provider: &LlmProvider) -> &'static str {
    match provider {
        LlmProvider::Anthropic => "claude-sonnet-4-6",
        LlmProvider::Openai => "gpt-4o-mini",
        LlmProvider::Gemini => "gemini-2.0-flash",
        LlmProvider::Ollama => "llama3.2",
    }
}

fn default_api_key_env_for(provider: &LlmProvider) -> Option<&'static str> {
    match provider {
        LlmProvider::Anthropic => Some("ANTHROPIC_API_KEY"),
        LlmProvider::Openai => Some("OPENAI_API_KEY"),
        LlmProvider::Gemini => Some("GEMINI_API_KEY"),
        LlmProvider::Ollama => None,
    }
}

fn default_endpoint_for(provider: &LlmProvider) -> Option<&'static str> {
    match provider {
        LlmProvider::Gemini => Some("https://generativelanguage.googleapis.com/v1beta/openai"),
        LlmProvider::Ollama => Some("http://localhost:11434/v1"),
        _ => None,
    }
}

fn spinner(message: &str) -> ProgressBar {
    let progress = ProgressBar::new_spinner();
    let style = ProgressStyle::with_template("{spinner} {msg}")
        .unwrap_or_else(|_| ProgressStyle::default_spinner());
    progress.set_style(style);
    progress.enable_steady_tick(Duration::from_millis(80));
    progress.set_message(message.to_string());
    progress
}

fn format_error_chain(err: &anyhow::Error) -> String {
    let mut lines = Vec::new();
    for (idx, cause) in err.chain().enumerate() {
        if idx == 0 {
            lines.push(cause.to_string());
        } else {
            lines.push(format!("  caused by: {cause}"));
        }
    }
    lines.join("\n")
}
