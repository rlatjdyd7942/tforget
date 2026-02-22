# Architecture

Single Rust binary. Monolithic pipeline: user input → recipe resolution → dependency sort → step execution → output.

## Pipeline

```
tforge new my-project
  → TUI prompts (or LLM-assisted via --ai flag)
  → Resolves to a Recipe (list of Templates + config)
  → Engine executes each Template's Steps in dependency order
  → Each Step: check prerequisites → execute → report status
  → Output: project directory + provisioned cloud resources
```

## Module Responsibilities

- `cli.rs` — clap command definitions
- `types.rs` — `TemplateManifest`, `StepDef`, `ParamDef` (serde-deserialized from TOML)
- `registry.rs` — discovers templates from directories and embedded assets
- `renderer.rs` — minijinja-based `{{variable}}` rendering in step commands
- `resolver.rs` — topological sort of templates by `requires_templates`
- `condition.rs` — evaluates step conditions (`services contains 'crashlytics'`, `db_engine == 'mysql-9.0'`)
- `executor.rs` — runs individual steps (shell commands, file copies, git clones) with idempotency checks
- `engine.rs` — orchestrates the full pipeline: resolve order → render variables → evaluate conditions → execute steps
- `state.rs` — persists step completion to `.tforge-state.json` for `tforge resume`
- `prompts.rs` — inquire-based interactive TUI
- `config.rs` — `~/.config/tforge/config.toml` management
- `llm/` — rig-core-based LLM invocation layer for Anthropic, OpenAI, Gemini, and Ollama in `--ai` mode
- `embedded.rs` — rust-embed for bundled templates

## Data Flow (`tforge new`)

1. Load registry (bundled + cached remote templates)
2. Interactive prompts OR LLM parses natural language → `RecipeSelection` (templates + vars)
3. Check required tool dependencies → print install hints if missing
4. `resolver::resolve_order()` — topological sort
5. `engine::run()` — for each template in order, for each step: render variables → check condition → check idempotency → execute
6. Track state in `.tforge-state.json`

## LLM Runtime Constraints

- `llm::query`/`llm::query_llm` must use `rig-core` provider clients as the only inference execution path.
- Provider-specific raw HTTP request code is not part of the target architecture.
- `LlmConfig.endpoint` is applied as a rig client base URL override when configured.
- LLM output contract remains JSON-only recipe output: `{"templates":[...],"parameters":{...}}`.

## Error Recovery

- Step state persisted to `.tforge-state.json`
- `tforge resume` retries from last failed step
- No automatic rollback (too dangerous for cloud resources)
- `tforge status` shows what was created

## Idempotency

Steps can have an optional `check` field. If the check command succeeds (exit 0), the step is skipped:

```toml
[[steps]]
type = "command"
check = "gcloud projects describe {{gcp_project_id}} --format='value(projectId)'"
command = "gcloud projects create {{gcp_project_id}}"
```
