---
name: figma-code-reader
description: Extract implementation-ready React/Tailwind guidance from Figma nodes using MCP tools, and validate design tokens with get_variable_defs. Use when a request includes a Figma URL/node-id or asks for code conversion from Figma design.
---

# Figma Code Reader

## Overview

Analyze a Figma node and return project-convention code guidance, not raw Figma output.
Use Figma MCP tools in a strict order so color/token data is reliable.

## Figma project

- File key: `IvfaZtQJlN9jIvGSaKeVPv`

## Required workflow

1. Read structure with `get_design_context`.
- Use `clientLanguages: "typescript,html,css"` and `clientFrameworks: "react"`.
- Capture layout, hierarchy, spacing, and component boundaries.
- Do not trust or reuse fallback hex values from generated snippets.

2. Validate tokens with `get_variable_defs`.
- Treat `get_variable_defs` as source of truth for colors, typography, and sizes.
- Cross-check all suspicious values from step 1.

3. Verify visuals with `get_screenshot`.
- Confirm layout relationships, sizing, and color contrast.
- Flag mismatches explicitly.

4. Convert to project conventions.
- Map design tokens with `docs/specs/frontend/design-tokens.md`.
- Confirm actual CSS variable usage from `frontend/src/index.css`.
- Prefer shadcn/ui + Tailwind classes and TypeScript React components.

## Conversion rules

- Prefer shadcn/ui components where possible.
- Use `@/` path alias for imports.
- Use `lucide-react` for icons.
- Avoid inline styles unless unavoidable.
- If a value is uncertain, re-check with `get_variable_defs`.
- Parse Figma URL node IDs as `?node-id=1-2` -> `1:2`.

## Output standard

Return these sections:

1. Design analysis
- Node structure summary and notable UI regions.
- Screenshot-based verification notes.

2. Extracted tokens
- Color mapping: Figma variable -> project token/CSS variable.
- Typography, spacing, and sizing summary.

3. Implementation guidance
- React + Tailwind code shaped for this repo.
- Notes on reusable components and required TypeScript types.

## References

- `docs/specs/frontend/design-tokens.md`
- `docs/specs/frontend/setup.md`
- `frontend/src/index.css`
- `docs/specs/frontend/pages/kurly-products.md`
- `frontend/CLAUDE.md`
- `docs/specs/common/ui-principles.md`
