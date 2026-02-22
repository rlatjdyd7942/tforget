# PLAN.md - tforge Implementation

> **Project:** tforge — a Rust CLI tool that scaffolds multi-stack projects with cloud infrastructure provisioning.

## Overview

Build a monolithic Rust CLI with a step-based pipeline execution engine. Templates are TOML manifests with provider metadata (`bundled`, `git`, `command`) and executable `[[steps]]`. Templates are composable via dependency declarations and conditional steps. LLM integration is pluggable and optional, with `rig-core` as the required invocation layer.

**Full Specification:** `docs/spec/` (architecture, templates, features, project overview)
**Implementation Plan:** `PLAN.md` (28 tasks, 12 phases, TDD approach)

---

## Architecture

### Pipeline Flow

```
User Input → Recipe Resolution → Dependency Sort → Step Execution → Output
```

1. Load registry (bundled + cached remote templates)
2. Interactive prompts OR LLM parses natural language → `RecipeSelection` (templates + vars, including conditional prompt gating)
3. Expand dependency templates and check required tools
4. Save `tforge.toml` and initial `.tforge-state.json`
5. `resolver::resolve_order()` — topological sort by `requires_templates`
6. `engine::run_with_state()` — for each template in order, for each step: render vars → check condition → check idempotency → execute
7. Track state in `.tforge-state.json`

### Module Dependency Graph

```
cli.rs (user input)
  → prompts.rs OR llm/ (rig-based recipe selection)
    → registry.rs (template discovery)
      → types.rs (manifest deserialization)
      → embedded.rs (bundled assets)
      → remote.rs (cached remote templates)
    → resolver.rs (topological sort)
    → engine.rs (orchestration)
      → renderer.rs (variable substitution)
      → condition.rs (step filtering)
      → executor.rs (step execution)
      → state.rs (progress tracking)
    → config.rs (settings)
    → toolcheck.rs (dependency verification)
```

### Key Types

| Type | Module | Description |
|------|--------|-------------|
| `TemplateManifest` | types.rs | Top-level: template info + dependencies + parameters + steps |
| `TemplateInfo` | types.rs | name, description, category, provider |
| `Provider` | types.rs | Enum: Bundled, Git, Command |
| `Dependencies` | types.rs | required_tools, requires_templates |
| `ParamDef` | types.rs | param_type (String/Select/MultiSelect/Bool), prompt, default, options, optional `when` |
| `StepDef` | types.rs | step_type, command, condition, check, working_dir, action, source, url |
| `Registry` | registry.rs | Template collection with find/by_category/categories methods |
| `Renderer` | renderer.rs | minijinja wrapper for `{{variable}}` substitution |
| `Engine` | engine.rs | Pipeline orchestrator: project_dir + Renderer |
| `StepContext` | executor.rs | project_dir + vars HashMap for step execution |
| `PipelineState` | state.rs | Step completion tracking for resume support |
| `TforgeConfig` | config.rs | Settings with optional LlmConfig |
| `LlmConfig` | config.rs | provider, model, api_key_env, endpoint |
| `LlmRecipe` | llm/mod.rs | Parsed LLM response: templates + parameters |
| `RecipeSelection` | prompts.rs | User selections: templates + vars |

### Tech Stack

```toml
# Core
clap = "4"           # CLI argument parsing (derive)
inquire = "0.9"      # Interactive TUI prompts
minijinja = "2"      # Template variable rendering
toml = "0.8"         # Manifest parsing
serde = "1"          # Serialization
serde_json = "1"     # JSON for state & LLM

# Async/Integration
reqwest = "0.12"     # HTTP client for non-LLM networking (e.g. registry/cache)
tokio = "1"          # Async runtime
rig-core = "0.31"    # Unified LLM provider invocation layer

# UI
indicatif = "0.17"   # Progress bars/spinners
owo-colors = "4"     # Terminal colors

# Infrastructure
rust-embed = "8"     # Bundle templates in binary
keyring = "3"        # Secure API key storage
                       # (current runtime uses api_key_env, keyring wiring is not active)
dirs = "6"           # Platform config directories
thiserror = "2"      # Typed errors
anyhow = "1"         # Error context

# Dev
assert_cmd = "2"     # CLI integration tests
predicates = "3"     # Test assertions
tempfile = "3"       # Temporary directories
```

---

## 현재 스프린트

---

## Error Recovery Design

- **No rollback** for cloud resources (too dangerous)
- State persisted to `.tforge-state.json` after each step
- `tforge resume` retries from last failed step
- `tforge status` reports per-template progress/failure from saved recipe + state
- Idempotency via optional `check` field on steps (exits 0 → skip)

## Template System

- Templates are TOML manifests at `templates/<name>/template.toml`
- Three providers: **Bundled** (rust-embed), **Git** (clone), **Command** (shell)
- Runtime step execution currently uses `command` and `git` step behavior (`bundled` step type is reserved/placeholder)
- Composability via `requires_templates` + conditional steps
- V1 bundled templates: flutter-app, axum-server, gcp-project, gcp-cloudsql, gcp-appengine, firebase-project, firebase-flutter
