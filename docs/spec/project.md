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
