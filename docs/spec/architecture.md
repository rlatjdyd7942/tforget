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
