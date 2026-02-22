---
name: spec
description: Specification authoring workflow for kurly-crawler that discovers related docs and code, updates the right spec files, and synchronizes PLAN.md. Use when requirements change and the user asks to define or revise how the system should behave before implementation.
---

# Spec

## Overview

Convert requirement changes into updated specification documents and an aligned implementation plan.
Mirror the intent of `.claude/commands/spec.md`.

## Workflow

1. Parse the requirement scope.
- Identify affected domains (common, backend admin/crawling, frontend pages, or backend-only).
- Extract explicit constraints and expected outcomes.

2. Run context discovery with a librarian-equivalent pass.
- Find related docs and code before writing specs.
- For Figma links, use `figma-code-reader` when available.
- If unavailable, continue with repository context and state the limitation.

3. Update the correct spec files.
- Use the minimum file set needed by scope.
- Primary targets from the source command:
  - `SPEC.md`
  - `docs/specs/core/`
  - `docs/specs/admin/`
  - `docs/specs/frontend/pages/`
  - `backend/SPEC.md` and `backend/specs/` for backend-only requirements
- Prefer canonical files that exist in this repo, including:
  - `docs/specs/common/`
  - `docs/specs/backend/admin/`
  - `docs/specs/backend/crawling/`

4. Write specifications as target state.
- Describe intended behavior, interfaces, constraints, and acceptance criteria.
- Do not record codebase problems, defect narratives, or temporary implementation commentary in spec docs.

5. Synchronize `PLAN.md`.
- Add or update implementation tasks based on the revised specs.
- Link each plan item to the spec files that define it.

6. Verify consistency.
- Check that `PLAN.md` and all touched specs describe the same target behavior.
- Call out unresolved decisions separately from finalized specs.

## Output Standard

Return concrete spec edits and plan updates with explicit file paths for every changed scope.
