# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

**tforge** — a Rust CLI tool that scaffolds multi-stack projects. Status: **implemented (v0.1)**.

See `docs/spec/README.md` for the full specification index.

## Spec Sync Rule

**Always update `docs/spec/` files after making changes to the project.** When editing architecture, modules, data flow, or any design aspect, reflect those changes in the corresponding spec file. Spec files are the source of truth for project documentation.

## Build & Test Commands

```bash
cargo build                              # build
cargo test                               # run all tests
cargo test --test <test_name>            # run single test file (e.g. --test types_test)
cargo test <test_fn_name>                # run single test function
cargo run -- new <project-name>          # run CLI
cargo run -- list                        # list templates
```

## Key References

- `docs/spec/` — project specification (see README.md for index)
- `PLAN.md` — implementation plan and task status
