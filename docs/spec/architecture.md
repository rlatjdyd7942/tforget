# Architecture

Single Rust binary. Monolithic pipeline: user input → recipe resolution → dependency sort → step execution → output.

## Pipeline

```
tforge new my-project
  → TUI prompts (or LLM-assisted via --ai flag)
  → Resolves to a Recipe (list of Templates + config)
  → Expands required template dependencies
  → Verifies required external tools are installed
  → Saves `tforge.toml` and `.tforge-state.json`
  → Engine executes each Template's Steps in dependency order
  → Each Step: check prerequisites → execute → report status
  → Output: generated files/resources driven by executed commands
```

## Module Responsibilities

- `cli.rs` — clap command definitions
- `types.rs` — `TemplateManifest`, `StepDef`, `ParamDef` (serde-deserialized from TOML, including parameter prompt conditions)
- `registry.rs` — discovers templates from directories and embedded assets
- `renderer.rs` — minijinja-based `{{variable}}` rendering in step commands
- `resolver.rs` — topological sort of templates by `requires_templates`
- `condition.rs` — evaluates condition expressions for step execution and parameter prompt visibility (`services contains 'crashlytics'`, `db_engine == 'mysql-9.0'`)
- `executor.rs` — runs individual steps (`command`, `git`) with idempotency checks; `bundled` steps are accepted but currently execute as placeholders
- `engine.rs` — orchestrates the full pipeline: resolve order → render variables → evaluate conditions → execute steps
- `state.rs` — persists step completion to `.tforge-state.json` for `tforge resume`
- `prompts.rs` — inquire-based interactive TUI with deterministic parameter ordering and conditional prompt gating
- `config.rs` — `~/.config/tforge/config.toml` management (LLM settings)
- `llm/` — rig-core-based LLM invocation layer for Anthropic, OpenAI, Gemini, and Ollama in `--ai` mode
- `embedded.rs` — rust-embed loader for bundled template manifests
- `remote.rs` — remote template fetching, caching, and search
- `toolcheck.rs` — validates required external tools declared by selected templates

## Module Dependency Graph

```
cli.rs
  → prompts.rs OR llm/
    → registry.rs
      → types.rs
      → embedded.rs
      → remote.rs
    → resolver.rs
    → engine.rs
      → renderer.rs
      → condition.rs
      → executor.rs
      → state.rs
    → config.rs
    → toolcheck.rs
```

## Key Runtime Types

| Type | Module | Purpose |
|------|--------|---------|
| `TemplateManifest` | `types.rs` | Template metadata + dependencies + parameters + steps |
| `TemplateInfo` | `types.rs` | Template identity and catalog metadata |
| `Provider` | `types.rs` | Provider metadata enum (`bundled`, `git`, `command`) |
| `Dependencies` | `types.rs` | Required tools + template dependencies |
| `ParamDef` | `types.rs` | Prompt and parameter schema, including `when` |
| `StepDef` | `types.rs` | Executable step schema (`type`, `command`, `check`, etc.) |
| `Registry` | `registry.rs` | Runtime template catalog and search surface |
| `Renderer` | `renderer.rs` | Variable rendering for step fields |
| `Engine` | `engine.rs` | Pipeline orchestrator for ordered template execution |
| `StepContext` | `executor.rs` | Per-step execution context (`project_dir`, `vars`) |
| `PipelineState` | `state.rs` | Persisted progress/failure model for resume/status |
| `TforgeConfig` | `config.rs` | Global user settings model |
| `LlmConfig` | `config.rs` | LLM provider/model/auth endpoint settings |
| `LlmRecipe` | `llm/mod.rs` | Parsed AI recipe output (`templates`, `parameters`) |
| `RecipeSelection` | `prompts.rs` | Prompt-selected templates and shared variables |

## Dependency Baseline

- Core CLI/prompt/rendering: `clap`, `inquire`, `minijinja`, `toml`, `serde`, `serde_json`
- Async/integration: `tokio`, `reqwest`, `rig-core`
- Terminal UX: `indicatif`, `owo-colors`
- Infrastructure/utilities: `rust-embed`, `dirs`, `keyring`, `thiserror`, `anyhow`
- Test support: `assert_cmd`, `predicates`, `tempfile`

## Data Flow (`tforge new`)

1. Load registry (bundled + cached remote templates)
2. Interactive prompts OR LLM parses natural language → `RecipeSelection` (templates + vars), including conditional parameter prompts (`when`)
3. Expand dependency templates (`requires_templates`) and validate external tools
4. Persist recipe and initial state (`tforge.toml`, `.tforge-state.json`)
5. `resolver::resolve_order()` — topological sort
6. `engine::run_with_state()` — for each template in order, for each step: render variables → check condition → check idempotency → execute
7. Track progress/failures in `.tforge-state.json` for `status`/`resume`

## LLM Runtime (rig-core)

- `llm::query_llm` uses `rig-core` provider clients as the only inference execution path.
- Provider-specific raw HTTP request code has been removed.
- `LlmConfig.endpoint` is applied as a rig client base URL override via `ClientBuilder::base_url()`.
- LLM output contract: JSON-only recipe output `{"templates":[...],"parameters":{...}}`.

## Error Recovery

- Step state persisted to `.tforge-state.json` in the invocation directory
- `tforge resume` retries from last failed step
- No automatic rollback (too dangerous for cloud resources)
- `tforge status` shows per-template progress/failure from saved state and recipe

## Idempotency

Steps can have an optional `check` field. If the check command succeeds (exit 0), the step is skipped:

```toml
[[steps]]
type = "command"
check = "gcloud projects describe {{gcp_project_id}} --format='value(projectId)'"
command = "gcloud projects create {{gcp_project_id}}"
```
