---
name: librarian
description: Find and summarize only the documents and code locations required for a task. Use before implementation when requirements, specs, or repository context must be gathered quickly.
---

# Librarian

## Overview

Run a focused discovery pass over specs and code, then return only task-relevant context.
Prioritize speed, precision, and traceable file references.

## Workflow

1. Parse the request.
- Identify domain (`backend`, `frontend`, `common`) and expected output.
- Extract constraints, affected areas, and unknowns.

2. Search relevant specs first.
- Start from `SPEC.md` and `PLAN.md`.
- Then inspect task-relevant files under `docs/specs/**`.
- For backend-heavy work, include `backend/SPEC.md` and `backend/specs/**` when relevant.

3. Find implementation references in code.
- Use targeted `rg` queries.
- Prefer high-signal paths first:
  - `backend/app/**`, `backend/migrations/**`, `backend/tests/**`
  - `frontend/src/**`

4. Return a compact, structured brief.
- Include only sources that directly affect the task.
- Provide file paths and line numbers for code references.

## Output standard

Use this exact section order:

1. Related specs
- `[doc title](path): relevant section summary`

2. Reference code
- `[file](path:line): why it matters`

3. Key takeaways
- Bullet list of implementation-critical facts only.

## Behavior rules

- Avoid broad dumps; select only actionable context.
- Mark uncertainty explicitly when evidence is incomplete.
- Prefer current repo evidence over assumptions.
- Keep summaries short and citation-heavy.
