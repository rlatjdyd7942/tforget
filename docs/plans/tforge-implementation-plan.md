# tforge Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust CLI tool that scaffolds multi-stack projects with cloud infrastructure provisioning, supporting bundled templates, git repos, and command-based scaffolding with optional LLM assistance.

**Architecture:** Monolithic pipeline with step-based execution engine. Templates are TOML manifests with three providers (bundled, git, command). Composable via dependency declarations and conditional steps. LLM integration is pluggable and optional.

**Tech Stack:** Rust, clap (CLI), inquire (TUI prompts), minijinja (templating), toml+serde (config), reqwest+tokio (HTTP/async), indicatif (progress), rust-embed (bundled files), keyring (secrets), owo-colors (terminal colors)

---

## Phase 1: Project Foundation

### Task 1: Initialize Rust project and dependencies (done)

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `.gitignore`

**Step 1: Initialize cargo project**

Run: `cargo init --name tforge`

**Step 2: Add dependencies to Cargo.toml**

```toml
[package]
name = "tforge"
version = "0.1.0"
edition = "2024"
description = "Project builder CLI — scaffold multi-stack projects with cloud provisioning"
license = "MIT"

[dependencies]
clap = { version = "4", features = ["derive"] }
inquire = "0.9"
minijinja = { version = "2", features = ["builtins"] }
toml = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json", "rustls-tls", "stream"], default-features = false }
tokio = { version = "1", features = ["rt", "macros", "process", "fs"] }
rig-core = "0.31"
indicatif = "0.17"
owo-colors = "4"
rust-embed = "8"
keyring = "3"
dirs = "6"
thiserror = "2"
anyhow = "1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

**Step 3: Create minimal main.rs**

```rust
fn main() {
    println!("tforge v0.1.0");
}
```

**Step 4: Verify it builds**

Run: `cargo build`
Expected: Compiles successfully.

**Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs .gitignore
git commit -m "feat: initialize tforge project with dependencies"
```

---

### Task 2: Define core data types

**Files:**
- Create: `src/types.rs`
- Modify: `src/main.rs` (add module)
- Create: `tests/types_test.rs`

**Step 1: Write tests for template manifest deserialization**

```rust
// tests/types_test.rs
use tforge::types::{TemplateManifest, Provider, StepDef, ParamDef, ParamType};

#[test]
fn test_deserialize_command_template() {
    let toml_str = r#"
[template]
name = "flutter-app"
description = "Flutter mobile application"
category = "mobile"
provider = "command"

[dependencies]
required_tools = ["flutter"]

[parameters]
org = { type = "string", prompt = "Organization name", default = "com.example" }

[[steps]]
type = "command"
command = "flutter create --org {{org}} {{project_name}}"
"#;
    let manifest: TemplateManifest = toml::from_str(toml_str).unwrap();
    assert_eq!(manifest.template.name, "flutter-app");
    assert_eq!(manifest.template.provider, Provider::Command);
    assert_eq!(manifest.dependencies.required_tools, vec!["flutter"]);
    assert_eq!(manifest.steps.len(), 1);
    assert!(manifest.parameters.contains_key("org"));
}

#[test]
fn test_deserialize_template_with_conditions() {
    let toml_str = r#"
[template]
name = "firebase-flutter"
description = "Firebase for Flutter"
category = "integration"
provider = "command"

[dependencies]
required_tools = ["firebase"]
requires_templates = ["flutter-app"]

[parameters]
services = { type = "multi-select", prompt = "Services?", options = ["crashlytics", "auth"], default = ["crashlytics"] }

[[steps]]
type = "command"
command = "flutter pub add firebase_crashlytics"
condition = "services contains 'crashlytics'"
"#;
    let manifest: TemplateManifest = toml::from_str(toml_str).unwrap();
    assert_eq!(manifest.dependencies.requires_templates, vec!["flutter-app"]);
    assert!(manifest.steps[0].condition.is_some());
}

#[test]
fn test_deserialize_bundled_step() {
    let toml_str = r#"
[template]
name = "test"
description = "test"
category = "test"
provider = "bundled"

[dependencies]

[[steps]]
type = "bundled"
action = "overlay"
source = "files/"
"#;
    let manifest: TemplateManifest = toml::from_str(toml_str).unwrap();
    assert_eq!(manifest.steps[0].step_type, "bundled");
    assert_eq!(manifest.steps[0].action.as_deref(), Some("overlay"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test types_test`
Expected: FAIL — module `tforge::types` not found.

**Step 3: Implement core types**

```rust
// src/types.rs
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct TemplateManifest {
    pub template: TemplateInfo,
    #[serde(default)]
    pub dependencies: Dependencies,
    #[serde(default)]
    pub parameters: HashMap<String, ParamDef>,
    #[serde(default)]
    pub steps: Vec<StepDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
    pub category: String,
    pub provider: Provider,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Bundled,
    Git,
    Command,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Dependencies {
    #[serde(default)]
    pub required_tools: Vec<String>,
    #[serde(default)]
    pub requires_templates: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ParamDef {
    #[serde(rename = "type")]
    pub param_type: ParamType,
    pub prompt: String,
    #[serde(default)]
    pub default: Option<toml::Value>,
    #[serde(default)]
    pub options: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ParamType {
    String,
    Select,
    MultiSelect,
    Bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StepDef {
    #[serde(rename = "type")]
    pub step_type: String,
    pub command: Option<String>,
    pub condition: Option<String>,
    pub check: Option<String>,
    pub working_dir: Option<String>,
    pub action: Option<String>,
    pub source: Option<String>,
    pub url: Option<String>,
}
```

**Step 4: Add module to main.rs and make it a library**

```rust
// src/main.rs
pub mod types;

fn main() {
    println!("tforge v0.1.0");
}
```

Also create `src/lib.rs`:

```rust
// src/lib.rs
pub mod types;
```

Remove `pub mod types;` from main.rs, and in main.rs:

```rust
// src/main.rs
fn main() {
    println!("tforge v0.1.0");
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test --test types_test`
Expected: All 3 tests PASS.

**Step 6: Commit**

```bash
git add src/types.rs src/lib.rs src/main.rs tests/types_test.rs
git commit -m "feat: add core template manifest types with deserialization"
```

---

### Task 3: CLI argument parsing with clap

**Files:**
- Create: `src/cli.rs`
- Modify: `src/main.rs`
- Modify: `src/lib.rs`
- Create: `tests/cli_test.rs`

**Step 1: Write CLI integration tests**

```rust
// tests/cli_test.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_no_args_shows_help() {
    Command::cargo_bin("tforge")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn test_version_flag() {
    Command::cargo_bin("tforge")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("tforge"));
}

#[test]
fn test_list_subcommand() {
    Command::cargo_bin("tforge")
        .unwrap()
        .arg("list")
        .assert()
        .success();
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test cli_test`
Expected: FAIL — no subcommands defined.

**Step 3: Implement CLI definition**

```rust
// src/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tforge", version, about = "Project builder CLI — scaffold multi-stack projects with cloud provisioning")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new project
    New {
        /// Project name
        name: String,
        /// Use LLM to interpret a natural language description
        #[arg(long)]
        ai: Option<String>,
    },
    /// Resume from last failed step
    Resume,
    /// Show current project state
    Status,
    /// List available templates
    List,
    /// Search template registry
    Search {
        /// Search query
        query: String,
    },
    /// Add a community template from a git URL
    Add {
        /// Git URL of the template repository
        url: String,
    },
    /// Update registry and cached templates
    Update,
    /// Configure tforge settings
    Config {
        /// Configuration target (e.g., "llm")
        target: String,
        /// Show current config
        #[arg(long)]
        show: bool,
    },
}
```

**Step 4: Wire CLI into main.rs**

```rust
// src/main.rs
use clap::Parser;
use tforge::cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::New { name, ai } => {
            println!("Creating project: {name}");
            if let Some(desc) = ai {
                println!("AI mode: {desc}");
            }
        }
        Commands::List => {
            println!("Available templates:");
            println!("  (none bundled yet)");
        }
        Commands::Search { query } => {
            println!("Searching for: {query}");
        }
        Commands::Add { url } => {
            println!("Adding template from: {url}");
        }
        Commands::Resume => {
            println!("Resuming...");
        }
        Commands::Status => {
            println!("Status: no active project");
        }
        Commands::Update => {
            println!("Updating registry...");
        }
        Commands::Config { target, show } => {
            if show {
                println!("Current {target} config: (none)");
            } else {
                println!("Configure {target}...");
            }
        }
    }
}
```

Add to lib.rs:

```rust
// src/lib.rs
pub mod cli;
pub mod types;
```

**Step 5: Run tests to verify they pass**

Run: `cargo test --test cli_test`
Expected: All 3 tests PASS.

**Step 6: Commit**

```bash
git add src/cli.rs src/main.rs src/lib.rs tests/cli_test.rs
git commit -m "feat: add CLI argument parsing with clap subcommands"
```

---

## Phase 2: Template Engine

### Task 4: Template discovery — find and parse bundled templates

**Files:**
- Create: `src/registry.rs`
- Create: `templates/` directory structure (empty initially)
- Modify: `src/lib.rs`
- Create: `tests/registry_test.rs`

**Step 1: Write tests for template discovery**

```rust
// tests/registry_test.rs
use tforge::registry::Registry;
use std::path::PathBuf;

#[test]
fn test_load_templates_from_directory() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/templates");
    let registry = Registry::from_directory(&fixtures).unwrap();
    assert!(registry.templates().len() >= 1);
}

#[test]
fn test_find_template_by_name() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/templates");
    let registry = Registry::from_directory(&fixtures).unwrap();
    let tmpl = registry.find("test-app");
    assert!(tmpl.is_some());
    assert_eq!(tmpl.unwrap().template.name, "test-app");
}

#[test]
fn test_find_nonexistent_template() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/templates");
    let registry = Registry::from_directory(&fixtures).unwrap();
    assert!(registry.find("nonexistent").is_none());
}

#[test]
fn test_filter_by_category() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/templates");
    let registry = Registry::from_directory(&fixtures).unwrap();
    let mobile = registry.by_category("mobile");
    assert!(mobile.iter().all(|t| t.template.category == "mobile"));
}
```

**Step 2: Create test fixture template**

Create `tests/fixtures/templates/test-app/template.toml`:

```toml
[template]
name = "test-app"
description = "Test application"
category = "mobile"
provider = "command"

[dependencies]
required_tools = []

[[steps]]
type = "command"
command = "echo hello"
```

**Step 3: Run tests to verify they fail**

Run: `cargo test --test registry_test`
Expected: FAIL — module not found.

**Step 4: Implement Registry**

```rust
// src/registry.rs
use crate::types::TemplateManifest;
use anyhow::{Context, Result};
use std::path::Path;

pub struct Registry {
    templates: Vec<TemplateManifest>,
}

impl Registry {
    pub fn from_directory(path: &Path) -> Result<Self> {
        let mut templates = Vec::new();

        if !path.exists() {
            return Ok(Self { templates });
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let template_toml = entry.path().join("template.toml");
            if template_toml.exists() {
                let content = std::fs::read_to_string(&template_toml)
                    .with_context(|| format!("reading {}", template_toml.display()))?;
                let manifest: TemplateManifest = toml::from_str(&content)
                    .with_context(|| format!("parsing {}", template_toml.display()))?;
                templates.push(manifest);
            }
        }

        templates.sort_by(|a, b| a.template.name.cmp(&b.template.name));
        Ok(Self { templates })
    }

    pub fn templates(&self) -> &[TemplateManifest] {
        &self.templates
    }

    pub fn find(&self, name: &str) -> Option<&TemplateManifest> {
        self.templates.iter().find(|t| t.template.name == name)
    }

    pub fn by_category(&self, category: &str) -> Vec<&TemplateManifest> {
        self.templates
            .iter()
            .filter(|t| t.template.category == category)
            .collect()
    }

    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self
            .templates
            .iter()
            .map(|t| t.template.category.clone())
            .collect();
        cats.sort();
        cats.dedup();
        cats
    }
}
```

**Step 5: Add module to lib.rs**

```rust
// src/lib.rs
pub mod cli;
pub mod registry;
pub mod types;
```

**Step 6: Run tests**

Run: `cargo test --test registry_test`
Expected: All 4 tests PASS.

**Step 7: Commit**

```bash
git add src/registry.rs src/lib.rs tests/registry_test.rs tests/fixtures/
git commit -m "feat: add template registry with directory-based discovery"
```

---

### Task 5: Template variable rendering with minijinja

**Files:**
- Create: `src/renderer.rs`
- Modify: `src/lib.rs`
- Create: `tests/renderer_test.rs`

**Step 1: Write tests for variable rendering**

```rust
// tests/renderer_test.rs
use tforge::renderer::Renderer;
use std::collections::HashMap;

#[test]
fn test_render_simple_variable() {
    let renderer = Renderer::new();
    let mut vars = HashMap::new();
    vars.insert("project_name".into(), "my-app".into());
    let result = renderer.render_string("hello {{project_name}}", &vars).unwrap();
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
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test renderer_test`
Expected: FAIL.

**Step 3: Implement Renderer**

```rust
// src/renderer.rs
use anyhow::{Context, Result};
use minijinja::Environment;
use std::collections::HashMap;

pub struct Renderer {
    env: Environment<'static>,
}

impl Renderer {
    pub fn new() -> Self {
        let env = Environment::new();
        Self { env }
    }

    pub fn render_string(&self, template: &str, vars: &HashMap<String, String>) -> Result<String> {
        let tmpl = self
            .env
            .template_from_str(template)
            .context("failed to parse template string")?;
        let result = tmpl.render(vars).context("failed to render template")?;
        Ok(result)
    }
}
```

**Step 4: Add module to lib.rs**

Add `pub mod renderer;` to `src/lib.rs`.

**Step 5: Run tests**

Run: `cargo test --test renderer_test`
Expected: All 4 tests PASS.

**Step 6: Commit**

```bash
git add src/renderer.rs src/lib.rs tests/renderer_test.rs
git commit -m "feat: add template variable renderer using minijinja"
```

---

### Task 6: Dependency resolution — topological sort of templates

**Files:**
- Create: `src/resolver.rs`
- Modify: `src/lib.rs`
- Create: `tests/resolver_test.rs`

**Step 1: Write tests for dependency resolution**

```rust
// tests/resolver_test.rs
use tforge::resolver::resolve_order;
use tforge::types::TemplateManifest;

fn make_manifest(name: &str, requires: Vec<&str>) -> TemplateManifest {
    let toml_str = format!(
        r#"
[template]
name = "{name}"
description = "test"
category = "test"
provider = "command"

[dependencies]
requires_templates = [{requires}]

[[steps]]
type = "command"
command = "echo {name}"
"#,
        name = name,
        requires = requires
            .iter()
            .map(|r| format!("\"{r}\""))
            .collect::<Vec<_>>()
            .join(", ")
    );
    toml::from_str(&toml_str).unwrap()
}

#[test]
fn test_no_dependencies() {
    let templates = vec![make_manifest("a", vec![])];
    let order = resolve_order(&templates).unwrap();
    assert_eq!(order, vec!["a"]);
}

#[test]
fn test_simple_dependency() {
    let templates = vec![
        make_manifest("firebase-flutter", vec!["flutter-app"]),
        make_manifest("flutter-app", vec![]),
    ];
    let order = resolve_order(&templates).unwrap();
    let a_pos = order.iter().position(|n| n == "flutter-app").unwrap();
    let b_pos = order.iter().position(|n| n == "firebase-flutter").unwrap();
    assert!(a_pos < b_pos);
}

#[test]
fn test_chain_dependency() {
    let templates = vec![
        make_manifest("gcp-cloudsql", vec!["gcp-project"]),
        make_manifest("gcp-project", vec![]),
        make_manifest("axum-server", vec![]),
    ];
    let order = resolve_order(&templates).unwrap();
    let proj_pos = order.iter().position(|n| n == "gcp-project").unwrap();
    let sql_pos = order.iter().position(|n| n == "gcp-cloudsql").unwrap();
    assert!(proj_pos < sql_pos);
}

#[test]
fn test_circular_dependency_errors() {
    let templates = vec![
        make_manifest("a", vec!["b"]),
        make_manifest("b", vec!["a"]),
    ];
    let result = resolve_order(&templates);
    assert!(result.is_err());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test resolver_test`
Expected: FAIL.

**Step 3: Implement topological sort**

```rust
// src/resolver.rs
use crate::types::TemplateManifest;
use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};

pub fn resolve_order(templates: &[TemplateManifest]) -> Result<Vec<String>> {
    let names: HashSet<&str> = templates.iter().map(|t| t.template.name.as_str()).collect();
    let deps: HashMap<&str, Vec<&str>> = templates
        .iter()
        .map(|t| {
            let name = t.template.name.as_str();
            let reqs: Vec<&str> = t
                .dependencies
                .requires_templates
                .iter()
                .filter(|r| names.contains(r.as_str()))
                .map(|r| r.as_str())
                .collect();
            (name, reqs)
        })
        .collect();

    let mut order = Vec::new();
    let mut visited = HashSet::new();
    let mut in_stack = HashSet::new();

    for name in &names {
        if !visited.contains(name) {
            visit(name, &deps, &mut visited, &mut in_stack, &mut order)?;
        }
    }

    Ok(order)
}

fn visit<'a>(
    node: &'a str,
    deps: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    in_stack: &mut HashSet<&'a str>,
    order: &mut Vec<String>,
) -> Result<()> {
    if in_stack.contains(node) {
        bail!("circular dependency detected involving '{node}'");
    }
    if visited.contains(node) {
        return Ok(());
    }

    in_stack.insert(node);

    if let Some(node_deps) = deps.get(node) {
        for dep in node_deps {
            visit(dep, deps, visited, in_stack, order)?;
        }
    }

    in_stack.remove(node);
    visited.insert(node);
    order.push(node.to_string());
    Ok(())
}
```

**Step 4: Add module to lib.rs**

Add `pub mod resolver;` to `src/lib.rs`.

**Step 5: Run tests**

Run: `cargo test --test resolver_test`
Expected: All 4 tests PASS.

**Step 6: Commit**

```bash
git add src/resolver.rs src/lib.rs tests/resolver_test.rs
git commit -m "feat: add template dependency resolver with cycle detection"
```

---

### Task 7: Condition evaluator for conditional steps

**Files:**
- Create: `src/condition.rs`
- Modify: `src/lib.rs`
- Create: `tests/condition_test.rs`

**Step 1: Write tests for condition evaluation**

```rust
// tests/condition_test.rs
use tforge::condition::evaluate_condition;
use std::collections::HashMap;

#[test]
fn test_contains_true() {
    let mut vars = HashMap::new();
    vars.insert("services".into(), "crashlytics,auth,analytics".into());
    assert!(evaluate_condition("services contains 'crashlytics'", &vars).unwrap());
}

#[test]
fn test_contains_false() {
    let mut vars = HashMap::new();
    vars.insert("services".into(), "auth,analytics".into());
    assert!(!evaluate_condition("services contains 'crashlytics'", &vars).unwrap());
}

#[test]
fn test_equals_true() {
    let mut vars = HashMap::new();
    vars.insert("db_engine".into(), "mysql-9.0".into());
    assert!(evaluate_condition("db_engine == 'mysql-9.0'", &vars).unwrap());
}

#[test]
fn test_equals_false() {
    let mut vars = HashMap::new();
    vars.insert("db_engine".into(), "postgres-16".into());
    assert!(!evaluate_condition("db_engine == 'mysql-9.0'", &vars).unwrap());
}

#[test]
fn test_missing_variable() {
    let vars = HashMap::new();
    let result = evaluate_condition("missing contains 'x'", &vars);
    assert!(result.is_err());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test condition_test`
Expected: FAIL.

**Step 3: Implement condition evaluator**

```rust
// src/condition.rs
use anyhow::{bail, Result};
use std::collections::HashMap;

pub fn evaluate_condition(condition: &str, vars: &HashMap<String, String>) -> Result<bool> {
    let condition = condition.trim();

    if let Some((var, value)) = parse_contains(condition) {
        let var_value = vars
            .get(var)
            .ok_or_else(|| anyhow::anyhow!("variable '{var}' not found"))?;
        Ok(var_value.split(',').any(|v| v.trim() == value))
    } else if let Some((var, value)) = parse_equals(condition) {
        let var_value = vars
            .get(var)
            .ok_or_else(|| anyhow::anyhow!("variable '{var}' not found"))?;
        Ok(var_value == value)
    } else if let Some((var, value)) = parse_not_equals(condition) {
        let var_value = vars
            .get(var)
            .ok_or_else(|| anyhow::anyhow!("variable '{var}' not found"))?;
        Ok(var_value != value)
    } else {
        bail!("unsupported condition syntax: '{condition}'")
    }
}

fn parse_contains(s: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = s.splitn(2, " contains ").collect();
    if parts.len() == 2 {
        Some((parts[0].trim(), strip_quotes(parts[1].trim())))
    } else {
        None
    }
}

fn parse_equals(s: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = s.splitn(2, " == ").collect();
    if parts.len() == 2 {
        Some((parts[0].trim(), strip_quotes(parts[1].trim())))
    } else {
        None
    }
}

fn parse_not_equals(s: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = s.splitn(2, " != ").collect();
    if parts.len() == 2 {
        Some((parts[0].trim(), strip_quotes(parts[1].trim())))
    } else {
        None
    }
}

fn strip_quotes(s: &str) -> &str {
    s.trim_matches('\'').trim_matches('"')
}
```

**Step 4: Add module, run tests, commit**

Add `pub mod condition;` to lib.rs.

Run: `cargo test --test condition_test`
Expected: All 5 tests PASS.

```bash
git add src/condition.rs src/lib.rs tests/condition_test.rs
git commit -m "feat: add condition evaluator for conditional template steps"
```

---

## Phase 3: Step Execution Engine

### Task 8: Tool dependency checker

**Files:**
- Create: `src/toolcheck.rs`
- Modify: `src/lib.rs`
- Create: `tests/toolcheck_test.rs`

**Step 1: Write tests**

```rust
// tests/toolcheck_test.rs
use tforge::toolcheck::{check_tool, ToolStatus};

#[test]
fn test_check_existing_tool() {
    // 'echo' should always exist
    let status = check_tool("echo");
    assert!(matches!(status, ToolStatus::Found(_)));
}

#[test]
fn test_check_nonexistent_tool() {
    let status = check_tool("nonexistent_tool_xyz_12345");
    assert!(matches!(status, ToolStatus::NotFound));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test toolcheck_test`
Expected: FAIL.

**Step 3: Implement tool checker**

```rust
// src/toolcheck.rs
use std::process::Command;

pub enum ToolStatus {
    Found(String), // version or path
    NotFound,
}

pub fn check_tool(name: &str) -> ToolStatus {
    match Command::new("which").arg(name).output() {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            ToolStatus::Found(path)
        }
        _ => ToolStatus::NotFound,
    }
}

pub fn install_hint(tool: &str) -> &str {
    match tool {
        "flutter" => "Install Flutter: https://docs.flutter.dev/get-started/install",
        "gcloud" => "Install gcloud CLI: https://cloud.google.com/sdk/docs/install",
        "firebase" => "Install Firebase CLI: npm install -g firebase-tools",
        "flutterfire" => "Install FlutterFire CLI: dart pub global activate flutterfire_cli",
        "node" | "npm" | "npx" => "Install Node.js: https://nodejs.org/",
        "cargo" => "Install Rust: https://rustup.rs/",
        "docker" => "Install Docker: https://docs.docker.com/get-docker/",
        "terraform" => "Install Terraform: https://developer.hashicorp.com/terraform/install",
        _ => "Please install this tool and try again",
    }
}
```

**Step 4: Add module, run tests, commit**

Add `pub mod toolcheck;` to lib.rs.

Run: `cargo test --test toolcheck_test`
Expected: All 2 tests PASS.

```bash
git add src/toolcheck.rs src/lib.rs tests/toolcheck_test.rs
git commit -m "feat: add tool dependency checker with install hints"
```

---

### Task 9: Step executor — run commands, copy files, clone repos

**Files:**
- Create: `src/executor.rs`
- Modify: `src/lib.rs`
- Create: `tests/executor_test.rs`

**Step 1: Write tests**

```rust
// tests/executor_test.rs
use tforge::executor::{execute_step, StepContext};
use tforge::types::StepDef;
use std::collections::HashMap;
use tempfile::TempDir;

fn make_command_step(cmd: &str) -> StepDef {
    toml::from_str(&format!(
        r#"
type = "command"
command = "{cmd}"
"#
    ))
    .unwrap()
}

#[test]
fn test_execute_simple_command() {
    let tmp = TempDir::new().unwrap();
    let ctx = StepContext {
        project_dir: tmp.path().to_path_buf(),
        vars: HashMap::new(),
    };
    let step = make_command_step("echo hello");
    let result = execute_step(&step, &ctx);
    assert!(result.is_ok());
}

#[test]
fn test_execute_command_with_working_dir() {
    let tmp = TempDir::new().unwrap();
    let sub = tmp.path().join("subdir");
    std::fs::create_dir(&sub).unwrap();

    let ctx = StepContext {
        project_dir: tmp.path().to_path_buf(),
        vars: HashMap::new(),
    };
    let step: StepDef = toml::from_str(
        r#"
type = "command"
command = "pwd"
working_dir = "subdir"
"#,
    )
    .unwrap();
    let result = execute_step(&step, &ctx);
    assert!(result.is_ok());
}

#[test]
fn test_execute_failing_command() {
    let tmp = TempDir::new().unwrap();
    let ctx = StepContext {
        project_dir: tmp.path().to_path_buf(),
        vars: HashMap::new(),
    };
    let step = make_command_step("false");
    let result = execute_step(&step, &ctx);
    assert!(result.is_err());
}

#[test]
fn test_execute_with_check_skips() {
    let tmp = TempDir::new().unwrap();
    // Create a file so the check passes
    std::fs::write(tmp.path().join("exists.txt"), "hi").unwrap();

    let ctx = StepContext {
        project_dir: tmp.path().to_path_buf(),
        vars: HashMap::new(),
    };
    let step: StepDef = toml::from_str(
        r#"
type = "command"
check = "test -f exists.txt"
command = "echo should-be-skipped"
"#,
    )
    .unwrap();
    let result = execute_step(&step, &ctx);
    assert!(result.is_ok());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test executor_test`
Expected: FAIL.

**Step 3: Implement executor**

```rust
// src/executor.rs
use crate::types::StepDef;
use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

pub struct StepContext {
    pub project_dir: PathBuf,
    pub vars: HashMap<String, String>,
}

pub enum StepResult {
    Executed,
    Skipped,
}

pub fn execute_step(step: &StepDef, ctx: &StepContext) -> Result<StepResult> {
    let working_dir = match &step.working_dir {
        Some(dir) => ctx.project_dir.join(dir),
        None => ctx.project_dir.clone(),
    };

    // Run idempotency check if present
    if let Some(check_cmd) = &step.check {
        let status = Command::new("sh")
            .arg("-c")
            .arg(check_cmd)
            .current_dir(&working_dir)
            .status()
            .context("failed to run check command")?;
        if status.success() {
            return Ok(StepResult::Skipped);
        }
    }

    match step.step_type.as_str() {
        "command" => {
            let cmd = step
                .command
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("command step missing 'command' field"))?;
            let output = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .current_dir(&working_dir)
                .output()
                .with_context(|| format!("failed to execute: {cmd}"))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                bail!("command failed: {cmd}\n{stderr}");
            }
            Ok(StepResult::Executed)
        }
        "bundled" => {
            // Will be implemented when we add the bundled file provider
            Ok(StepResult::Executed)
        }
        "git" => {
            let url = step
                .url
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("git step missing 'url' field"))?;
            let output = Command::new("git")
                .args(["clone", "--depth", "1", url])
                .current_dir(&working_dir)
                .output()
                .with_context(|| format!("failed to clone: {url}"))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                bail!("git clone failed: {url}\n{stderr}");
            }
            Ok(StepResult::Executed)
        }
        other => bail!("unknown step type: {other}"),
    }
}
```

**Step 4: Add module, run tests, commit**

Add `pub mod executor;` to lib.rs.

Run: `cargo test --test executor_test`
Expected: All 4 tests PASS.

```bash
git add src/executor.rs src/lib.rs tests/executor_test.rs
git commit -m "feat: add step executor with command, bundled, and git providers"
```

---

### Task 10: Pipeline engine — orchestrate full recipe execution

**Files:**
- Create: `src/engine.rs`
- Modify: `src/lib.rs`
- Create: `tests/engine_test.rs`

**Step 1: Write tests for end-to-end pipeline**

```rust
// tests/engine_test.rs
use tforge::engine::Engine;
use tforge::types::TemplateManifest;
use std::collections::HashMap;
use tempfile::TempDir;

#[test]
fn test_engine_runs_single_template() {
    let tmp = TempDir::new().unwrap();
    let manifest: TemplateManifest = toml::from_str(
        r#"
[template]
name = "test"
description = "test"
category = "test"
provider = "command"

[dependencies]

[[steps]]
type = "command"
command = "mkdir -p app && echo 'hello' > app/README.md"
"#,
    )
    .unwrap();

    let mut vars = HashMap::new();
    vars.insert("project_name".into(), "test-project".into());

    let engine = Engine::new(tmp.path().to_path_buf());
    engine.run(&[manifest], &vars).unwrap();

    assert!(tmp.path().join("app/README.md").exists());
}

#[test]
fn test_engine_respects_dependency_order() {
    let tmp = TempDir::new().unwrap();

    let first: TemplateManifest = toml::from_str(
        r#"
[template]
name = "base"
description = "base"
category = "test"
provider = "command"
[dependencies]
[[steps]]
type = "command"
command = "echo 'first' > order.txt"
"#,
    )
    .unwrap();

    let second: TemplateManifest = toml::from_str(
        r#"
[template]
name = "addon"
description = "addon"
category = "test"
provider = "command"
[dependencies]
requires_templates = ["base"]
[[steps]]
type = "command"
command = "echo 'second' >> order.txt"
"#,
    )
    .unwrap();

    let vars = HashMap::new();
    let engine = Engine::new(tmp.path().to_path_buf());
    // Pass in reverse order to test sorting
    engine.run(&[second, first], &vars).unwrap();

    let content = std::fs::read_to_string(tmp.path().join("order.txt")).unwrap();
    assert!(content.contains("first"));
    assert!(content.contains("second"));
}

#[test]
fn test_engine_skips_conditional_steps() {
    let tmp = TempDir::new().unwrap();
    let manifest: TemplateManifest = toml::from_str(
        r#"
[template]
name = "cond"
description = "test"
category = "test"
provider = "command"
[dependencies]

[parameters]
services = { type = "multi-select", prompt = "?", options = ["a", "b"] }

[[steps]]
type = "command"
command = "touch a.txt"
condition = "services contains 'a'"

[[steps]]
type = "command"
command = "touch b.txt"
condition = "services contains 'b'"
"#,
    )
    .unwrap();

    let mut vars = HashMap::new();
    vars.insert("services".into(), "a".into());

    let engine = Engine::new(tmp.path().to_path_buf());
    engine.run(&[manifest], &vars).unwrap();

    assert!(tmp.path().join("a.txt").exists());
    assert!(!tmp.path().join("b.txt").exists());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test engine_test`
Expected: FAIL.

**Step 3: Implement Engine**

```rust
// src/engine.rs
use crate::condition::evaluate_condition;
use crate::executor::{execute_step, StepContext, StepResult};
use crate::renderer::Renderer;
use crate::resolver::resolve_order;
use crate::types::TemplateManifest;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Engine {
    project_dir: PathBuf,
    renderer: Renderer,
}

impl Engine {
    pub fn new(project_dir: PathBuf) -> Self {
        Self {
            project_dir,
            renderer: Renderer::new(),
        }
    }

    pub fn run(&self, templates: &[TemplateManifest], vars: &HashMap<String, String>) -> Result<()> {
        let order = resolve_order(templates)?;

        let template_map: HashMap<&str, &TemplateManifest> = templates
            .iter()
            .map(|t| (t.template.name.as_str(), t))
            .collect();

        for name in &order {
            let tmpl = template_map
                .get(name.as_str())
                .ok_or_else(|| anyhow::anyhow!("template '{name}' not found in map"))?;

            for (i, step) in tmpl.steps.iter().enumerate() {
                // Check condition
                if let Some(cond) = &step.condition {
                    let rendered_cond = self.renderer.render_string(cond, vars)
                        .with_context(|| format!("[{name}] step {}: failed to render condition", i + 1))?;
                    if !evaluate_condition(&rendered_cond, vars)? {
                        continue;
                    }
                }

                // Render step fields
                let mut rendered_step = step.clone();
                if let Some(cmd) = &step.command {
                    rendered_step.command = Some(
                        self.renderer
                            .render_string(cmd, vars)
                            .with_context(|| format!("[{name}] step {}: failed to render command", i + 1))?,
                    );
                }
                if let Some(wd) = &step.working_dir {
                    rendered_step.working_dir = Some(self.renderer.render_string(wd, vars)?);
                }
                if let Some(check) = &step.check {
                    rendered_step.check = Some(self.renderer.render_string(check, vars)?);
                }

                let ctx = StepContext {
                    project_dir: self.project_dir.clone(),
                    vars: vars.clone(),
                };

                match execute_step(&rendered_step, &ctx)
                    .with_context(|| format!("[{name}] step {} failed", i + 1))?
                {
                    StepResult::Executed => {}
                    StepResult::Skipped => {}
                }
            }
        }

        Ok(())
    }
}
```

**Step 4: Add module, run tests, commit**

Add `pub mod engine;` to lib.rs.

Run: `cargo test --test engine_test`
Expected: All 3 tests PASS.

```bash
git add src/engine.rs src/lib.rs tests/engine_test.rs
git commit -m "feat: add pipeline engine with dependency ordering and conditions"
```

---

## Phase 4: State Tracking & Resume

### Task 11: State persistence and resume

**Files:**
- Create: `src/state.rs`
- Modify: `src/lib.rs`
- Create: `tests/state_test.rs`

**Step 1: Write tests**

```rust
// tests/state_test.rs
use tforge::state::{PipelineState, StepState};
use tempfile::TempDir;

#[test]
fn test_save_and_load_state() {
    let tmp = TempDir::new().unwrap();
    let state_file = tmp.path().join(".tforge-state.json");

    let mut state = PipelineState::new();
    state.mark_completed("flutter-app", 0);
    state.mark_completed("flutter-app", 1);
    state.mark_failed("gcp-project", 0, "quota exceeded");
    state.save(&state_file).unwrap();

    let loaded = PipelineState::load(&state_file).unwrap();
    assert_eq!(loaded.get("flutter-app", 0), StepState::Completed);
    assert_eq!(loaded.get("flutter-app", 1), StepState::Completed);
    assert!(matches!(loaded.get("gcp-project", 0), StepState::Failed(_)));
    assert_eq!(loaded.get("gcp-project", 1), StepState::Pending);
}

#[test]
fn test_load_nonexistent_returns_empty() {
    let tmp = TempDir::new().unwrap();
    let state_file = tmp.path().join(".tforge-state.json");
    let state = PipelineState::load(&state_file).unwrap();
    assert_eq!(state.get("anything", 0), StepState::Pending);
}
```

**Step 2: Run tests to verify they fail, then implement**

```rust
// src/state.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PipelineState {
    steps: HashMap<String, HashMap<usize, StepStateEntry>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum StepStateEntry {
    Completed,
    Failed(String),
}

#[derive(Debug, PartialEq)]
pub enum StepState {
    Pending,
    Completed,
    Failed(String),
}

impl PipelineState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = std::fs::read_to_string(path)?;
        let state: PipelineState = serde_json::from_str(&content)?;
        Ok(state)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn mark_completed(&mut self, template: &str, step_idx: usize) {
        self.steps
            .entry(template.to_string())
            .or_default()
            .insert(step_idx, StepStateEntry::Completed);
    }

    pub fn mark_failed(&mut self, template: &str, step_idx: usize, error: &str) {
        self.steps
            .entry(template.to_string())
            .or_default()
            .insert(step_idx, StepStateEntry::Failed(error.to_string()));
    }

    pub fn get(&self, template: &str, step_idx: usize) -> StepState {
        self.steps
            .get(template)
            .and_then(|s| s.get(&step_idx))
            .map(|e| match e {
                StepStateEntry::Completed => StepState::Completed,
                StepStateEntry::Failed(msg) => StepState::Failed(msg.clone()),
            })
            .unwrap_or(StepState::Pending)
    }
}
```

**Step 3: Add module, run tests, commit**

Run: `cargo test --test state_test`
Expected: All 2 tests PASS.

```bash
git add src/state.rs src/lib.rs tests/state_test.rs
git commit -m "feat: add pipeline state persistence for resume support"
```

---

## Phase 5: Interactive TUI

### Task 12: Interactive prompt flow for `tforge new`

**Files:**
- Create: `src/prompts.rs`
- Modify: `src/lib.rs`
- Modify: `src/main.rs` — wire prompts into `Commands::New`

**Step 1: Implement interactive prompt module**

This module wraps `inquire` to collect template selections and parameter values. Hard to unit test (interactive I/O), so we test the non-interactive parts and manually verify the TUI.

```rust
// src/prompts.rs
use crate::registry::Registry;
use crate::types::{ParamType, TemplateManifest};
use anyhow::{Context, Result};
use inquire::{Confirm, MultiSelect, Select, Text};
use std::collections::HashMap;

pub struct RecipeSelection {
    pub templates: Vec<TemplateManifest>,
    pub vars: HashMap<String, String>,
}

pub fn prompt_recipe(registry: &Registry, project_name: &str) -> Result<RecipeSelection> {
    let mut selected_templates: Vec<TemplateManifest> = Vec::new();
    let mut vars = HashMap::new();

    vars.insert("project_name".into(), project_name.into());

    // Step 1: Select primary templates by category
    let categories = registry.categories();
    for category in &categories {
        let templates = registry.by_category(category);
        if templates.is_empty() {
            continue;
        }

        let names: Vec<String> = templates.iter().map(|t| {
            format!("{} — {}", t.template.name, t.template.description)
        }).collect();

        let selections = MultiSelect::new(
            &format!("Select {category} templates:"),
            names.clone(),
        )
        .prompt()
        .context("template selection cancelled")?;

        for selection in &selections {
            let idx = names.iter().position(|n| n == selection).unwrap();
            selected_templates.push(templates[idx].clone().clone());
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
            selected_templates.push(integration_templates[idx].clone().clone());
        }
    }

    // Step 3: Collect parameters for all selected templates
    for tmpl in &selected_templates {
        for (key, param) in &tmpl.parameters {
            if vars.contains_key(key) {
                continue; // Already provided by a previous template
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

            vars.insert(key.clone(), value);
        }
    }

    Ok(RecipeSelection {
        templates: selected_templates,
        vars,
    })
}
```

**Step 2: Wire into main.rs**

Update the `Commands::New` match arm to use the prompt flow, engine, and progress display. Use `indicatif` for step progress.

```rust
// In main.rs, update the New command handler:
Commands::New { name, ai } => {
    // Load registry
    // If ai mode: use LLM (Task 14)
    // Else: run interactive prompts
    // Check tool dependencies
    // Show confirmation
    // Run engine with progress display
}
```

(Full wiring deferred to when all pieces are ready — see Task 15.)

**Step 3: Commit**

```bash
git add src/prompts.rs src/lib.rs
git commit -m "feat: add interactive TUI prompt flow for recipe selection"
```

---

## Phase 6: Configuration & LLM

### Task 13: Config management

**Files:**
- Create: `src/config.rs`
- Modify: `src/lib.rs`
- Create: `tests/config_test.rs`

**Step 1: Write tests**

```rust
// tests/config_test.rs
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
```

**Step 2: Implement config**

```rust
// src/config.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TforgeConfig {
    pub llm: Option<LlmConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub model: String,
    pub api_key_env: Option<String>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    Anthropic,
    Openai,
    Gemini,
    Ollama,
}

impl TforgeConfig {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tforge")
    }

    pub fn default_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let config: TforgeConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
```

**Step 3: Add module, run tests, commit**

Run: `cargo test --test config_test`
Expected: All 3 tests PASS.

```bash
git add src/config.rs src/lib.rs tests/config_test.rs
git commit -m "feat: add configuration management with LLM provider settings"
```

---

### Task 14: LLM provider abstraction and natural language recipe parsing

**Files:**
- Create: `src/llm/mod.rs`
- Create: `src/llm/anthropic.rs`
- Create: `src/llm/openai.rs`
- Modify: `src/lib.rs`
- Create: `tests/llm_test.rs`

**Step 1: Write tests for the LLM interface (mocked)**

```rust
// tests/llm_test.rs
use tforge::llm::{parse_llm_recipe_response, LlmRecipe};

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

    let recipe: LlmRecipe = parse_llm_recipe_response(json).unwrap();
    assert_eq!(recipe.templates, vec!["flutter-app", "axum-server", "gcp-project", "firebase-flutter"]);
    assert_eq!(recipe.parameters.get("org").unwrap(), "com.example");
}

#[test]
fn test_parse_invalid_json() {
    let result = parse_llm_recipe_response("not json");
    assert!(result.is_err());
}
```

**Step 2: Implement LLM module**

```rust
// src/llm/mod.rs
pub mod anthropic;
pub mod openai;

use crate::config::{LlmConfig, LlmProvider};
use crate::registry::Registry;
use anyhow::{bail, Result};
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
```

```rust
// src/llm/anthropic.rs
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
```

```rust
// src/llm/openai.rs
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
```

**Step 3: Add module, run tests, commit**

Add `pub mod llm;` to lib.rs.

Run: `cargo test --test llm_test`
Expected: All 2 tests PASS.

```bash
git add src/llm/ src/lib.rs tests/llm_test.rs
git commit -m "feat: add pluggable LLM integration with Anthropic and OpenAI providers"
```

---

## Phase 7: Wire Everything Together

### Task 15: Complete `main.rs` — full `tforge new` flow with progress display

**Files:**
- Modify: `src/main.rs` — complete implementation of all commands

**Step 1: Implement the full `tforge new` command flow**

Wire together: registry → prompts (or LLM) → tool checks → confirmation → engine with progress display using indicatif. Also implement `tforge config llm` with interactive prompts and `tforge list`/`tforge resume`/`tforge status`.

This is the glue task — all the pieces are built, this connects them with proper error messages, progress spinners, and colored output.

**Step 2: Manual end-to-end test**

Run: `cargo run -- new test-project`
Expected: Interactive prompts appear, templates execute, project created.

Run: `cargo run -- list`
Expected: Shows bundled templates.

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire complete tforge new flow with progress display"
```

---

## Phase 8: Bundled Templates

### Task 16: Create Flutter app template

**Files:**
- Create: `templates/flutter-app/template.toml`
- Create: `templates/flutter-app/files/` (overlay files if needed)

Template manifest for Flutter with `flutter create` command, org/platforms parameters, and overlay files for common setup (analysis_options.yaml, etc.).

**Commit:** `feat: add bundled flutter-app template`

---

### Task 17: Create Axum server template

**Files:**
- Create: `templates/axum-server/template.toml`
- Create: `templates/axum-server/files/` (bundled Cargo.toml, src/main.rs, etc.)

Bundled template with a complete Axum project skeleton: Cargo.toml, src/main.rs with basic routes, .env.example, Dockerfile.

**Commit:** `feat: add bundled axum-server template`

---

### Task 18: Create GCP templates (project, Cloud SQL, App Engine)

**Files:**
- Create: `templates/gcp-project/template.toml`
- Create: `templates/gcp-cloudsql/template.toml`
- Create: `templates/gcp-appengine/template.toml`

Command-based templates using `gcloud` CLI with idempotency checks.

**Commit:** `feat: add bundled GCP templates (project, Cloud SQL, App Engine)`

---

### Task 19: Create Firebase templates

**Files:**
- Create: `templates/firebase-project/template.toml`
- Create: `templates/firebase-flutter/template.toml`

Firebase project creation and FlutterFire integration with conditional service setup.

**Commit:** `feat: add bundled Firebase templates`

---

### Task 20: Embed bundled templates with rust-embed

**Files:**
- Create: `src/embedded.rs`
- Modify: `src/registry.rs` — add method to load from embedded assets
- Modify: `src/lib.rs`

Use `rust-embed` to bundle the `templates/` directory into the binary. Registry gains `from_embedded()` constructor that reads templates from the binary.

**Commit:** `feat: embed bundled templates in binary with rust-embed`

---

## Phase 9: Template Registry (Remote)

### Task 21: Remote registry fetching and template caching

**Files:**
- Create: `src/remote.rs`
- Modify: `src/lib.rs`

Implement `tforge search`, `tforge add`, `tforge update`. Fetch registry.toml from GitHub, cache templates locally at `~/.config/tforge/templates/`.

**Commit:** `feat: add remote template registry with search and caching`

---

## Phase 10: Polish

### Task 22: Error messages, help text, and README

- Improve error messages throughout (colored, actionable)
- Add `--help` examples to clap commands
- Verify all commands work end-to-end
- Add basic README.md with usage examples

**Commit:** `docs: add README and polish CLI help text`

---

## Summary

| Phase | Tasks | What it delivers |
|-------|-------|-----------------|
| 1: Foundation | 1-3 | Rust project, core types, CLI skeleton |
| 2: Template Engine | 4-7 | Registry, renderer, resolver, conditions |
| 3: Execution | 8-10 | Tool checker, step executor, pipeline engine |
| 4: State | 11 | Resume support |
| 5: TUI | 12 | Interactive prompts |
| 6: Config & LLM | 13-14 | Settings management, LLM integration |
| 7: Wiring | 15 | Complete `tforge new` flow |
| 8: Templates | 16-20 | Flutter, Axum, GCP, Firebase templates |
| 9: Registry | 21 | Remote template discovery |
| 10: Polish | 22 | Error handling, docs |
