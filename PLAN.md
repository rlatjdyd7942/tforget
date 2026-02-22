# PLAN.md - tforge Execution Plan

`PLAN.md` tracks implementation work only.
Behavior and requirements live in `docs/spec/`.

## Spec References

- `docs/spec/project.md` — project scope and current product state
- `docs/spec/architecture.md` — pipeline, modules, data flow, types
- `docs/spec/templates.md` — template manifest contract and runtime semantics
- `docs/spec/features.md` — CLI behavior, LLM flow, cloud workflows

## Planning Rules

1. Spec-first: update `docs/spec/*` before adding or changing plan items when behavior changes.
2. Every plan item must cite at least one spec file.
3. Every completed item must record validation evidence (`cargo test`, targeted test, or manual command proof).
4. Keep this file focused on status and execution steps. Do not duplicate architecture/feature narratives.

## Status Legend

- `todo` — queued, not started
- `in_progress` — actively being worked
- `blocked` — waiting on decision/dependency
- `done` — implemented and validated

## Current Sprint

- Sprint goal: define and execute post-v0.1 stabilization work.
- Active items: none

## Backlog

| ID | Status | Work Item | Spec Links | Validation Target |
|----|--------|-----------|------------|-------------------|
| TF-PLAN-001 | todo | Decide final runtime behavior for `bundled` steps (keep placeholder vs real file overlay/copy semantics) | `docs/spec/templates.md`, `docs/spec/architecture.md` | Spec update + tests proving selected behavior |
| TF-PLAN-002 | todo | Implement the `bundled` step behavior chosen in TF-PLAN-001 | `docs/spec/templates.md`, `docs/spec/architecture.md` | `cargo test` + executor integration coverage |
| TF-PLAN-003 | todo | Add targeted regression tests for `resume`/`status` state transitions with mixed success/failure steps | `docs/spec/features.md`, `docs/spec/architecture.md` | `cargo test --test state_test` and related integration tests |

## Done

- v0.1 baseline implemented; reference: `docs/spec/project.md` (`Status: implemented (v0.1)` and V1 scope section).
