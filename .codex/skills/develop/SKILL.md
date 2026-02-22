---
name: develop
description: PLAN-driven implementation workflow for kurly-crawler that discovers required docs and code context before making changes. Use when the user asks to execute tasks from PLAN.md, continue planned development, or implement scoped work items with validation.
---

# Develop

## Overview

Execute plan-backed development work with explicit context discovery and validation.
Mirror the intent of `.claude/commands/develop.md`.

## Workflow

1. Start from `PLAN.md`.
- Read the relevant plan section.
- Identify concrete tasks, dependencies, and acceptance signals.

2. Run a librarian-equivalent discovery pass.
- Gather the docs and code needed for current plan items.
- Prioritize `SPEC.md`, `docs/specs/**`, root `CLAUDE.md`, and area-specific guidance (`backend/CLAUDE.md`, `frontend/CLAUDE.md`).
- Read only the files needed to execute current tasks safely.

3. Implement plan items.
- Follow plan order unless dependency constraints require resequencing.
- Keep edits scoped to the targeted task.
- Update code, tests, and docs together when behavior changes.

4. Validate the implementation.
- Run targeted checks for touched areas.
- If checks fail, diagnose and fix or report exact blockers.

5. Reflect progress in planning artifacts.
- Mark completed tasks.
- Update `PLAN.md` when scope, ordering, or follow-up tasks change.

6. Apply deep reasoning for ambiguous work.
- Treat `ultrathink` in the source command as a requirement to reason carefully through edge cases before editing.

## Output Standard

Return a concise implementation report: changed files, validation results, and remaining plan tasks.
