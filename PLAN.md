# PLAN.md - tforge Implementation

> **Project:** tforge — a Rust CLI tool that scaffolds multi-stack projects with cloud infrastructure provisioning.

## Overview

Build a monolithic Rust CLI with a step-based pipeline execution engine. Templates are TOML manifests with three provider types (bundled, git, command). Composable via dependency declarations and conditional steps. LLM integration is pluggable and optional.

**Full Specification:** `docs/spec/` (architecture, templates, features, project overview)
**Implementation Plan:** `docs/plans/tforge-implementation-plan.md` (22 tasks, 10 phases, TDD approach)

---

## Architecture

### Pipeline Flow

```
User Input → Recipe Resolution → Dependency Sort → Step Execution → Output
```

1. Load registry (bundled + cached remote templates)
2. Interactive prompts OR LLM parses natural language → `RecipeSelection` (templates + vars)
3. Check required tool dependencies → print install hints if missing
4. `resolver::resolve_order()` — topological sort by `requires_templates`
5. `engine::run()` — for each template in order, for each step: render vars → check condition → check idempotency → execute
6. Track state in `.tforge-state.json`

### Module Dependency Graph

```
cli.rs (user input)
  → prompts.rs OR llm/ (recipe selection)
    → registry.rs (template discovery)
      → types.rs (manifest deserialization)
    → resolver.rs (topological sort)
    → engine.rs (orchestration)
      → renderer.rs (variable substitution)
      → condition.rs (step filtering)
      → executor.rs (step execution)
      → state.rs (progress tracking)
    → config.rs (settings)
    → toolcheck.rs (dependency verification)
    → embedded.rs (bundled assets)
```

### Key Types

| Type | Module | Description |
|------|--------|-------------|
| `TemplateManifest` | types.rs | Top-level: template info + dependencies + parameters + steps |
| `TemplateInfo` | types.rs | name, description, category, provider |
| `Provider` | types.rs | Enum: Bundled, Git, Command |
| `Dependencies` | types.rs | required_tools, requires_templates |
| `ParamDef` | types.rs | param_type (String/Select/MultiSelect/Bool), prompt, default, options |
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

# Async/HTTP
reqwest = "0.12"     # HTTP client (rustls-tls)
tokio = "1"          # Async runtime

# UI
indicatif = "0.17"   # Progress bars/spinners
owo-colors = "4"     # Terminal colors

# Infrastructure
rust-embed = "8"     # Bundle templates in binary
keyring = "3"        # Secure API key storage
dirs = "6"           # Platform config directories
thiserror = "2"      # Typed errors
anyhow = "1"         # Error context

# Dev
assert_cmd = "2"     # CLI integration tests
predicates = "3"     # Test assertions
tempfile = "3"       # Temporary directories
```

---

## Task Status

### Phase 1: Foundation

| # | Task | Files | Status |
|---|------|-------|--------|
| 1 | Initialize Rust project & deps | `Cargo.toml`, `src/main.rs`, `.gitignore` | ✅ Done |
| 2 | Define core data types | `src/types.rs`, `src/lib.rs`, `tests/types_test.rs` | ✅ Done |
| 3 | CLI argument parsing (clap) | `src/cli.rs`, `src/main.rs`, `tests/cli_test.rs` | ✅ Done |

### Phase 2: Template Engine

| # | Task | Files | Status |
|---|------|-------|--------|
| 4 | Template discovery (registry) | `src/registry.rs`, `tests/registry_test.rs`, `tests/fixtures/` | ✅ Done |
| 5 | Variable rendering (minijinja) | `src/renderer.rs`, `tests/renderer_test.rs` | ✅ Done |
| 6 | Dependency resolution (toposort) | `src/resolver.rs`, `tests/resolver_test.rs` | ✅ Done |
| 7 | Condition evaluator | `src/condition.rs`, `tests/condition_test.rs` | ✅ Done |

### Phase 3: Execution Engine

| # | Task | Files | Status |
|---|------|-------|--------|
| 8 | Tool dependency checker | `src/toolcheck.rs`, `tests/toolcheck_test.rs` | ✅ Done |
| 9 | Step executor | `src/executor.rs`, `tests/executor_test.rs` | ✅ Done |
| 10 | Pipeline engine | `src/engine.rs`, `tests/engine_test.rs` | ✅ Done |

### Phase 4: State Tracking

| # | Task | Files | Status |
|---|------|-------|--------|
| 11 | State persistence & resume | `src/state.rs`, `tests/state_test.rs` | ✅ Done |

### Phase 5: Interactive TUI

| # | Task | Files | Status |
|---|------|-------|--------|
| 12 | Interactive prompt flow | `src/prompts.rs` | ✅ Done |

### Phase 6: Configuration & LLM

| # | Task | Files | Status |
|---|------|-------|--------|
| 13 | Config management | `src/config.rs`, `tests/config_test.rs` | ✅ Done |
| 14 | LLM provider abstraction | `src/llm/mod.rs`, `src/llm/anthropic.rs`, `src/llm/openai.rs`, `tests/llm_test.rs` | ✅ Done |

### Phase 7: Wire Everything

| # | Task | Files | Status |
|---|------|-------|--------|
| 15 | Complete `main.rs` — full flow | `src/main.rs` | ⬜ Wave 2 |

### Phase 8: Bundled Templates

| # | Task | Files | Status |
|---|------|-------|--------|
| 16 | Flutter app template | `templates/flutter-app/template.toml`, `templates/flutter-app/files/` | ✅ Done |
| 17 | Axum server template | `templates/axum-server/template.toml`, `templates/axum-server/files/` | ✅ Done |
| 18 | GCP templates | `templates/gcp-project/`, `templates/gcp-cloudsql/`, `templates/gcp-appengine/` | ✅ Done |
| 19 | Firebase templates | `templates/firebase-project/`, `templates/firebase-flutter/` | ✅ Done |

### Phase 9: Template Registry

| # | Task | Files | Status |
|---|------|-------|--------|
| 20 | Embed templates (rust-embed) | `src/embedded.rs`, modify `src/registry.rs` | ⬜ Wave 2 |
| 21 | Remote registry & caching | `src/remote.rs` | ⬜ Wave 2 |

### Phase 10: Polish

| # | Task | Files | Status |
|---|------|-------|--------|
| 22 | Error messages, help, README | Various | ⬜ Wave 2 |

---

## Completion Summary

### Wave 1: Core Implementation — COMPLETE

All 19 tasks (2-14, 16-19) implemented with 39 passing tests.

**Modules implemented:**
- `src/{types,cli,registry,renderer,resolver,condition,toolcheck,executor,engine,state,config,prompts}.rs`
- `src/llm/{mod,anthropic,openai}.rs`
- 7 bundled templates in `templates/`
- 12 test files in `tests/`

### Wave 2: Remaining Tasks

| Task | Description | Status |
|------|-------------|--------|
| 15 | Complete `main.rs` — full `tforge new` flow with progress display | Pending |
| 20 | Embed bundled templates with rust-embed | Pending |
| 21 | Remote registry fetching and template caching | Pending |
| 22 | Error messages, help text, and README | Pending |

---

## Error Recovery Design

- **No rollback** for cloud resources (too dangerous)
- State persisted to `.tforge-state.json` after each step
- `tforge resume` retries from last failed step
- `tforge status` shows what was created
- Idempotency via optional `check` field on steps (exits 0 → skip)

## Template System

- Templates are TOML manifests at `templates/<name>/template.toml`
- Three providers: **Bundled** (rust-embed), **Git** (clone), **Command** (shell)
- Composability via `requires_templates` + conditional steps
- V1 bundled templates: flutter-app, axum-server, gcp-project, gcp-cloudsql, gcp-appengine, firebase-project, firebase-flutter
