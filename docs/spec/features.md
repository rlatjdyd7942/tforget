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
tforge config reset          # reset config
```

## TUI Interactive Prompts

inquire-based interactive prompts for template selection and parameter input. This is the default mode — LLM is optional.

## LLM Integration

Optional. The tool works fully without it via TUI prompts.

### Two Modes

1. **Natural language input** — `tforge new --ai "flutter app with firebase and GCP"` → LLM maps to templates + params → user confirms before execution
2. **Smart recommendations** — during interactive prompts, LLM suggests relevant integrations

### Pluggable Providers (Rig-Based)

Supports Anthropic, OpenAI, Gemini, Ollama (local). Configured via `tforge config llm`. Stored in `~/.config/tforge/config.toml`. API keys in system keychain or encrypted.

All provider invocations MUST go through `rig-core` as the single LLM invocation layer.

- No provider-specific raw HTTP request logic is allowed in tforge application LLM modules.
- Provider switching must be configuration-only (`provider`, `model`, `api_key_env`, optional `endpoint`) without changing call-site logic.
- Endpoint overrides must be applied through rig client configuration (base URL override), not manual request URL construction.

### Acceptance Criteria

1. `--ai` mode and recommendation mode use the same `rig-core`-based query path.
2. Anthropic, OpenAI, Gemini, and Ollama are all invoked via rig provider clients.
3. `src/llm/` contains no direct provider HTTP client calls for runtime inference.
4. Changing LLM provider requires only config changes, not code changes.

## Cloud Provisioning

Cloud actions are templates — same system, same TOML manifests. No special cloud subsystem.

Idempotency via step `check` fields ensures cloud resources aren't duplicated. See `architecture.md` for details.

## Config Management

Global config at `~/.config/tforge/config.toml`. Manages:
- LLM provider settings and API keys
- Template cache location
- Default preferences

Reset via `tforge config reset`.
