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
- `PLAN.md` is execution tracking only (tasks, status, validation) and should link to spec files instead of duplicating behavior descriptions.
