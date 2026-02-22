# tforge - Project Specification

**tforge** is a Rust CLI tool that scaffolds multi-stack projects (Flutter, Axum, etc.) with cloud infrastructure provisioning workflows (GCP/Firebase) in a single pipeline.

Status: **implemented (v0.1)**.

## Core Concepts

- **Template** - A TOML manifest (`template.toml`) describing one component (for example `flutter-app`, `axum-server`, `gcp-project`).
- **Step** - An atomic execution unit inside a template. Current runtime semantics are command execution and git clone execution, with optional condition/check gates.
- **Recipe** - The resolved template set plus parameter values persisted to `tforge.toml`.
- **Provider** - Template metadata (`bundled`, `git`, `command`). Bundled manifests are embedded in the binary; runtime step execution is driven by `[[steps]]`.

## V1 Implemented Scope

- CLI workflow: `new`, `list`, `search`, `add`, `update`, `resume`, `status`, `config`.
- Template system: manifest parsing, dependency expansion (`requires_templates`), topological ordering, conditional/idempotent step execution.
- Template parameter prompting: deterministic lexical ordering plus optional `when` conditions for prompt-time gating.
- Bundled manifest catalog: `flutter-app`, `axum-server`, `gcp-project`, `gcp-cloudsql`, `gcp-appengine`, `firebase-project`, `firebase-flutter`.
- `gcp-appengine` guided deployment profile: deploy target selection (`project-root`, `flutter-app`, `axum-server`, `custom-path`) and environment-specific settings for `standard`/`flexible`.
- Optional LLM-assisted recipe selection via `tforge new <name> --ai "..."` using `rig-core`.
- Inquire-based interactive prompts for non-LLM project setup.
- Persistent execution state via `tforge.toml` and `.tforge-state.json`.
