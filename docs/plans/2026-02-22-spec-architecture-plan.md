# Spec Files Architecture Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reorganize project documentation from a monolithic design doc into 4 structured spec files in `docs/spec/`.

**Architecture:** Content from `docs/plans/tforge-design.md` and existing `docs/spec/project.md` is split across topic-grouped spec files. Spec files become the source of truth; design doc is archived.

**Tech Stack:** Markdown files only. No code changes.

---

### Task 1: Create `docs/spec/README.md`

**Files:**
- Create: `docs/spec/README.md`

**Step 1: Write the file**

```markdown
# tforge Specification

## File Map

| File | Contents |
|------|----------|
| `project.md` | Project overview, core concepts, V1 scope |
| `architecture.md` | Pipeline, modules, data flow, error recovery |
| `templates.md` | Manifest format, providers, composability, registry |
| `features.md` | CLI commands, LLM integration, cloud provisioning, TUI |

## Maintenance Rules

- **Spec files are the source of truth** for project documentation.
- Update the relevant spec file whenever you change architecture, modules, or features.
- Each file is self-contained — readable without cross-referencing other spec files.
- When adding a new major topic area, create a new spec file and add it to this table.

## Archived

- `docs/plans/tforge-design.md` — original design document. Superseded by these spec files. Kept as historical record, not maintained.
```

**Step 2: Commit**

```bash
git add docs/spec/README.md
git commit -m "docs: add spec README with file map and maintenance rules"
```

---

### Task 2: Rewrite `docs/spec/project.md`

**Files:**
- Modify: `docs/spec/project.md`

**Step 1: Replace file contents**

Trim to overview + glossary + V1 scope only. Remove architecture/modules/data flow (those move to `architecture.md`).

```markdown
# tforge — Project Specification

**tforge** is a Rust CLI tool that scaffolds multi-stack projects (Flutter, Axum, Next.js, etc.) with cloud infrastructure provisioning (GCP, Firebase, AWS) in a single command. Target: open-source developer community.

Status: **pre-implementation**.

## Core Concepts

- **Template** — a TOML manifest (`template.toml`) + optional bundled files. Describes one component (e.g., "flutter-app", "axum-server", "gcp-project").
- **Step** — an atomic action: run a command, copy files, clone a repo, execute a cloud CLI.
- **Recipe** — a resolved combination of templates + user configuration values. What the user is actually building.
- **Provider** — a template source: `bundled` (embedded in binary), `git` (cloned at runtime), `command` (delegates to existing CLIs like `flutter create`).

## V1 Scope

Ship with:
- CLI engine (pipeline, template resolution, dependency ordering)
- Template system (all 3 providers, composability, conditions)
- Bundled templates: flutter-app, axum-server, gcp-project, gcp-cloudsql, gcp-appengine, firebase-project, firebase-flutter
- LLM integration (pluggable, optional)
- TUI interactive prompts
- Resume/state tracking
```

**Step 2: Commit**

```bash
git add docs/spec/project.md
git commit -m "docs: trim project.md to overview, glossary, and V1 scope"
```

---

### Task 3: Create `docs/spec/architecture.md`

**Files:**
- Create: `docs/spec/architecture.md`

**Step 1: Write the file**

Source content from existing `project.md` (modules, data flow) and `tforge-design.md` (pipeline, error recovery, idempotency).

```markdown
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
- `llm/` — pluggable LLM providers (Anthropic, OpenAI, Gemini, Ollama) for `--ai` mode
- `embedded.rs` — rust-embed for bundled templates

## Data Flow (`tforge new`)

1. Load registry (bundled + cached remote templates)
2. Interactive prompts OR LLM parses natural language → `RecipeSelection` (templates + vars)
3. Check required tool dependencies → print install hints if missing
4. `resolver::resolve_order()` — topological sort
5. `engine::run()` — for each template in order, for each step: render variables → check condition → check idempotency → execute
6. Track state in `.tforge-state.json`

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
```

**Step 2: Commit**

```bash
git add docs/spec/architecture.md
git commit -m "docs: add architecture spec (pipeline, modules, data flow, error recovery)"
```

---

### Task 4: Create `docs/spec/templates.md`

**Files:**
- Create: `docs/spec/templates.md`

**Step 1: Write the file**

Source content from `tforge-design.md` (manifest examples, providers, composability, parameter sharing, registry, output structure).

```markdown
# Template System

## Manifest Format

Templates live in `templates/<name>/template.toml`.

```toml
[template]
name = "flutter-app"
description = "Flutter mobile application"
category = "mobile"
provider = "command"

[dependencies]
required_tools = ["flutter"]
requires_templates = []

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

Key sections:
- `[template]` — name, description, category, provider
- `[dependencies]` — required_tools, requires_templates
- `[parameters]` — user-configurable values (string, multi-select, with defaults)
- `[[steps]]` — ordered actions with optional `condition` and `check` fields

## Providers

1. **Bundled** — static files embedded in the binary via rust-embed
2. **Git** — template repos cloned at runtime
3. **Command** — delegates to existing CLIs (flutter create, npx create-react-app, etc.)

### Dependency Handling

CLI checks if required tools exist, prints install instructions if missing. Does not auto-install.

## Composability

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

## Parameter Sharing

Parameters set in one template (e.g., `gcp_project_id` in `gcp-project`) are automatically available to downstream templates that depend on it.

## Registry

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
```

**Step 2: Commit**

```bash
git add docs/spec/templates.md
git commit -m "docs: add templates spec (manifest, providers, composability, registry)"
```

---

### Task 5: Create `docs/spec/features.md`

**Files:**
- Create: `docs/spec/features.md`

**Step 1: Write the file**

Source content from `tforge-design.md` (CLI commands, LLM integration, cloud provisioning).

```markdown
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

### Pluggable Providers

Supports Anthropic, OpenAI, Gemini, Ollama (local). Configured via `tforge config llm`. Stored in `~/.config/tforge/config.toml`. API keys in system keychain or encrypted.

## Cloud Provisioning

Cloud actions are templates — same system, same TOML manifests. No special cloud subsystem.

Idempotency via step `check` fields ensures cloud resources aren't duplicated. See `architecture.md` for details.

## Config Management

Global config at `~/.config/tforge/config.toml`. Manages:
- LLM provider settings and API keys
- Template cache location
- Default preferences

Reset via `tforge config reset`.
```

**Step 2: Commit**

```bash
git add docs/spec/features.md
git commit -m "docs: add features spec (CLI, TUI, LLM, cloud, config)"
```

---

### Task 6: Update `CLAUDE.md`

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Update references**

Replace current content with:

```markdown
# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

**tforge** — a Rust CLI tool that scaffolds multi-stack projects. Status: **pre-implementation**.

See `docs/spec/README.md` for the full specification index.

## Spec Sync Rule

**Always update `docs/spec/` files after making changes to the project.** When editing architecture, modules, data flow, or any design aspect, reflect those changes in the corresponding spec file. Spec files are the source of truth for project documentation.

## Build & Test Commands

```bash
cargo build                              # build
cargo test                               # run all tests
cargo test --test <test_name>            # run single test file (e.g. --test types_test)
cargo test <test_fn_name>                # run single test function
cargo run -- new <project-name>          # run CLI
cargo run -- list                        # list templates
```

## Key References

- `docs/spec/` — project specification (see README.md for index)
- `docs/plans/tforge-implementation-plan.md` — 22 tasks across 10 phases (TDD approach)
```

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md to reference spec index"
```

---

### Task 7: Archive design doc

**Files:**
- Modify: `docs/plans/tforge-design.md:1-2`

**Step 1: Add archive notice**

Prepend to the top of the file:

```markdown
> **ARCHIVED:** This document has been superseded by `docs/spec/`. It is kept as a historical record and is no longer maintained. See `docs/spec/README.md` for the current specification.

```

**Step 2: Commit**

```bash
git add docs/plans/tforge-design.md
git commit -m "docs: archive tforge-design.md (superseded by docs/spec/)"
```
