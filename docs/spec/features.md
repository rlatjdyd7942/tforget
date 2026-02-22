# Features

## CLI Commands

```
tforge new <name>            # create new project (interactive)
tforge new <name> --ai "..." # create with LLM assistance
tforge resume                # retry from last failed step
tforge status                # show current project state
tforge list                  # list available templates
tforge search <query>        # search template registry
tforge add <git-url>         # add community template
tforge update                # update registry + templates
tforge config llm            # configure LLM provider
tforge config llm --show     # show current LLM config
tforge config reset          # reset config
```

## TUI Interactive Prompts

`inquire`-based interactive prompts handle template selection and parameter input in the default (non-AI) flow.

Prompt behavior requirements:
- Parameter prompts are deterministic (lexical key order within a template).
- Parameters may be conditionally shown through manifest-level `when` expressions.
- `when` expressions use the same condition grammar as step conditions.

## LLM Integration

Optional. The tool works fully without it via TUI prompts.

### Supported Mode

1. **Natural language recipe selection** - `tforge new <name> --ai "flutter app with firebase and GCP"`:
   - Builds a system prompt from the loaded template registry.
   - Expects JSON output: `{"templates":[...],"parameters":{...}}`.
   - Resolves selected templates against local registry and asks for execution confirmation before running.

### Pluggable Providers (Rig-Based)

Supports Anthropic, OpenAI, Gemini, and Ollama through `rig-core`. Configured via `tforge config llm` and stored in `~/.config/tforge/config.toml`.

All provider invocations go through `rig-core` as the single LLM invocation layer.

- No provider-specific raw HTTP request logic in tforge LLM modules.
- Provider switching is configuration-only (`provider`, `model`, `api_key_env`, optional `endpoint`).
- Endpoint overrides applied through rig client configuration (base URL override).
- API keys are read from the environment variable named by `api_key_env`.

### Implementation Status: Complete

1. `--ai` mode uses the `rig-core`-based query path via `llm::query_llm()`.
2. Anthropic, OpenAI, Gemini, and Ollama are all invoked via rig provider clients.
3. `src/llm/mod.rs` uses `rig::providers::{anthropic, openai, ollama}` — no raw HTTP calls.
4. Changing LLM provider requires only config changes.

## Cloud Provisioning

Cloud actions are templates — same system, same TOML manifests. No special cloud subsystem.

Idempotency via step `check` fields ensures cloud resources aren't duplicated. See `architecture.md` for details.

### App Engine Guided Configuration

The `gcp-appengine` template must guide users through deployment-profile setup:

1. Select deployment target (`project-root`, `flutter-app`, `axum-server`, or `custom-path`).
2. Select App Engine environment (`standard` or `flexible`).
3. Enter only settings relevant to the selected environment.
4. Generate/update `app.yaml` in the selected target directory.
5. Optionally deploy immediately (`gcloud app deploy`) when `deploy_now` is enabled.

Acceptance criteria:
- Standard-only settings are never prompted when `flexible` is selected.
- Flexible-only settings are never prompted when `standard` is selected.
- `custom-path` requires an explicit path prompt; non-custom targets do not.
- Template execution uses the resolved target directory for file generation and deploy commands.

## Config Management

Global config at `~/.config/tforge/config.toml`. Current config surface:
- Optional LLM settings (`provider`, `model`, `api_key_env`, `endpoint`)

Reset via `tforge config reset`.

## Template Registry Sources

The runtime registry merges templates from:
- Embedded manifests shipped in the binary (`rust-embed`)
- Local `templates/` directory when present (development override)
- Cached remote templates under `~/.config/tforge/templates/`
