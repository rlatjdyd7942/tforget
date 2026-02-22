---
name: discuss
description: Interactive discussion workflow for kurly-crawler features and design decisions grounded in repository evidence. Use when asked to discuss requirements, architecture, UI behavior, tradeoffs, or Figma-linked implementation ideas before writing specs or code.
---

# Discuss

## Overview

Run a document-grounded discussion loop before committing to specification or implementation changes.
Mirror the intent of `.claude/commands/discuss.md`.

## Workflow

1. Parse the request.
- Capture the feature or design topic, constraints, assumptions, and explicit links.
- Record what must be decided in this discussion round.

2. Discover context with a librarian-equivalent pass.
- Search for related docs and code with targeted `rg` queries.
- Prioritize `SPEC.md`, `PLAN.md`, `CLAUDE.md`, `docs/specs/**`, and relevant `backend/` or `frontend/` files.
- Read only the files needed for the current topic.

3. Handle Figma links.
- Use `figma-code-reader` when available to extract implementation-relevant details.
- If unavailable, state the limitation and continue with repository evidence.

4. Run the discussion.
- Present options and tradeoffs clearly.
- Tie claims to file paths and current behavior.
- Mark unknowns and risks explicitly.

5. Close with actionable output.
- Summarize decisions and rationale.
- List open questions.
- Recommend next step: `spec` for spec updates or `develop` for implementation.

## Output Standard

Return concise, evidence-backed discussion guidance with direct file references for every significant recommendation.
