> **ARCHIVED:** This document has been superseded by `docs/spec/`. It is kept as a historical record and is no longer maintained. See `docs/spec/README.md` for the current specification.

# tforge — Project Builder CLI

## Overview

**tforge** is a Rust CLI tool that scaffolds multi-stack projects with cloud infrastructure provisioning. It combines project templates (Flutter, Axum, Next.js, etc.) with cloud setup (GCP, Firebase, AWS, etc.) into a single command.

Target: open-source developer community.

## Core Concepts

- **Template** — a TOML manifest + optional bundled files. Describes one component (e.g., "flutter-app", "axum-server", "gcp-project").
- **Step** — an atomic action: copy files, run a command, clone a repo, execute cloud CLI.
- **Recipe** — a combination of templates + configuration. What the user is actually building.
- **Provider** — a template source: `bundled`, `git`, or `command`.

## Architecture: Monolithic Pipeline

Single Rust binary. Everything is a step in a pipeline.

```
tforge new my-project
  → TUI prompts (or LLM-assisted via --ai flag)
  → Resolves to a Recipe (list of Templates + config)
  → Engine executes each Template's Steps in dependency order
  → Each Step: check prerequisites → execute → report status
  → Output: project directory + provisioned cloud resources
```

## Template System

### Three providers

1. **Bundled** — static files embedded in the binary
2. **Git** — template repos cloned at runtime
3. **Command** — delegates to existing CLIs (flutter create, npx create-react-app, etc.)

### Dependency handling for tools

Check + guide: CLI checks if required tools exist, prints install instructions if missing. Does not auto-install.

### Template manifest format

```toml
[template]
name = "flutter-app"
description = "Flutter mobile application"
category = "mobile"
provider = "command"

[dependencies]
required_tools = ["flutter"]
requires_templates = []  # optional: templates that must be selected too

[parameters]
org = { type = "string", prompt = "Organization name", default = "com.example" }
platforms = { type = "multi-select", prompt = "Target platforms", options = ["ios", "android", "web"], default = ["ios", "android"] }

[[steps]]
type = "command"
command = "flutter create --org {{org}} --platforms {{platforms|join(',')}} {{project_name}}"

[[steps]]
type = "bundled"
action = "overlay"
source = "files/"

[[steps]]
type = "command"
command = "flutter pub get"
working_dir = "{{project_name}}"
```

### Composability

Templates compose via `requires_templates` and conditional steps.

Example: `firebase-flutter` requires `flutter-app` and only appears when Flutter is selected.

```toml
[template]
name = "firebase-flutter"
description = "Firebase setup for Flutter"
category = "integration"
provider = "command"

[dependencies]
required_tools = ["firebase", "flutterfire"]
requires_templates = ["flutter-app"]

[parameters.services]
type = "multi-select"
prompt = "Which Firebase services?"
options = ["crashlytics", "auth", "firestore", "cloud-messaging", "analytics"]
default = ["crashlytics", "analytics"]

[[steps]]
type = "command"
command = "firebase projects:create {{firebase_project_id}}"

[[steps]]
type = "command"
command = "flutterfire configure --project={{firebase_project_id}}"
working_dir = "{{project_name}}"

[[steps]]
type = "command"
condition = "services contains 'crashlytics'"
command = "flutter pub add firebase_crashlytics"
working_dir = "{{project_name}}"
```

### Parameter sharing

Parameters set in one template (e.g., `gcp_project_id` in `gcp-project`) are automatically available to downstream templates that depend on it.

## Cloud Provisioning

Cloud actions are templates too — same system, same TOML manifests.

### Idempotency

Steps can have an optional `check` field. If the check command succeeds (exit 0), the step is skipped:

```toml
[[steps]]
type = "command"
check = "gcloud projects describe {{gcp_project_id}} --format='value(projectId)'"
command = "gcloud projects create {{gcp_project_id}}"
```

### Error recovery

- Step state persisted to `.tforge-state.json`
- `tforge resume` retries from last failed step
- No automatic rollback (too dangerous for cloud resources)
- `tforge status` shows what was created

## LLM Integration

Optional. Tool works fully without it via interactive TUI prompts.

### Two modes

1. **Natural language input** — `tforge new --ai "flutter app with firebase and GCP"` → LLM maps to templates + params → user confirms
2. **Smart recommendations** — during interactive prompts, LLM suggests relevant integrations

### Pluggable providers

Supports Anthropic, OpenAI, Gemini, Ollama (local). Configured via:

```
$ tforge config llm
```

Stored in `~/.config/tforge/config.toml`. API keys in system keychain or encrypted.

## Template Registry

- **Bundled** — ship with binary (flutter-app, axum-server, gcp-project, firebase-flutter)
- **Community** — central `registry.toml` on GitHub pointing to git repos
- **Custom** — `tforge add <git-url>` to add any template

Templates cached at `~/.config/tforge/templates/`.

## Output Structure

```
my-app/
├── tforge.toml          ← recipe manifest (reproducible)
├── app/                 ← Flutter project
├── server/              ← Axum project
└── deploy/              ← deployment configs
```

`tforge.toml` records all selections and parameters, enabling `tforge status` and `tforge rerun`.

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

## V1 Scope

Ship with:
- CLI engine (pipeline, template resolution, dependency ordering)
- Template system (all 3 providers, composability, conditions)
- Bundled templates: flutter-app, axum-server, gcp-project, gcp-cloudsql, gcp-appengine, firebase-project, firebase-flutter
- LLM integration (pluggable, optional)
- TUI interactive prompts
- Resume/state tracking
