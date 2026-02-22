# Spec Files Architecture Design

Date: 2026-02-22

## Goal

Replace the monolithic `docs/plans/tforge-design.md` and the single `docs/spec/project.md` with a structured set of spec files in `docs/spec/`. Spec files serve as both human-readable documentation and Claude Code context. They are the source of truth for project documentation.

## Structure

```
docs/spec/
├── README.md            ← index: file map, maintenance rules
├── project.md           ← overview, core concepts, V1 scope
├── architecture.md      ← pipeline, modules, data flow, error recovery
├── templates.md         ← manifest format, providers, composability, registry
└── features.md          ← CLI commands, LLM integration, cloud provisioning, TUI
```

## File Contents

### README.md
- File map with one-line descriptions
- Maintenance rules: update spec files after any project change, each file is self-contained
- Note that `docs/plans/tforge-design.md` is archived (original design record, not maintained)

### project.md
- One-paragraph project description
- Core concepts glossary (Template, Step, Recipe, Provider)
- V1 scope (what ships: CLI engine, template system, bundled templates list, LLM integration, TUI, resume/state)

### architecture.md
- Pipeline overview (user input → recipe resolution → dependency sort → step execution → output)
- Module responsibilities (cli.rs, types.rs, registry.rs, renderer.rs, resolver.rs, condition.rs, executor.rs, engine.rs, state.rs, prompts.rs, config.rs, llm/, embedded.rs)
- Data flow for `tforge new` (6-step sequence)
- Error recovery & state tracking (.tforge-state.json, tforge resume, no auto-rollback)
- Idempotency (step `check` field)

### templates.md
- Manifest format (template.toml structure with full TOML examples from design doc)
- Three providers (bundled, git, command)
- Composability (requires_templates, conditional steps)
- Parameter sharing across templates
- Template registry (bundled, community, custom via tforge add)
- Output structure (tforge.toml, project directory layout)

### features.md
- CLI commands (tforge new, list, search, add, update, config, resume, status)
- TUI interactive prompts (inquire-based)
- LLM integration (natural language mode, smart recommendations, pluggable providers, config)
- Cloud provisioning (cloud actions as templates, idempotency)
- Config management (~/.config/tforge/config.toml)

## CLAUDE.md Changes

- Reference `docs/spec/README.md` as the entry point for project specs
- Keep build & test commands (operational, not spec)
- Update Key References section
- Spec Sync Rule stays

## Migration

- Content sourced from: existing `docs/spec/project.md` + `docs/plans/tforge-design.md`
- `docs/plans/tforge-design.md` archived (kept in place, noted as superseded in spec README)
- No content is lost; it's reorganized across the 4 spec files

## Decisions

- **Granularity:** 4 topic-grouped files (not per-topic or monolith)
- **Relationship to design doc:** Spec replaces design doc as source of truth; design doc frozen as historical record
- **Purpose:** Both human docs and Claude Code context equally
- **Self-contained files:** Each file readable without needing to cross-reference others
