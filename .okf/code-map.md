---
type: Reference
title: Code Map
description: Workspace layout and entry points for common change surfaces.
tags: [layout, monorepo]
---

# Code map

## Top-level layout

| Path                                 | Role                                                        |
| ------------------------------------ | ----------------------------------------------------------- |
| `codex-rs/`                          | Native Cargo workspace and per-crate Bazel targets          |
| `codex-cli/`                         | npm CLI launcher                                            |
| `sdk/`                               | TypeScript SDK, Python SDK, and Python runtime package      |
| `scripts/`                           | Formatting, installation, packaging, and repository helpers |
| `.github/`                           | CI and release automation                                   |
| `docs/`                              | Product/developer docs, often pointers to hosted references |
| `bazel/`, `MODULE.bazel`, `defs.bzl` | Bazel toolchains, dependencies, and wrappers                |
| `justfile`                           | Common commands, default working directory `codex-rs`       |

## Native task locations

| Surface                                  | Start here                                                                         |
| ---------------------------------------- | ---------------------------------------------------------------------------------- |
| Command parsing/dispatch                 | `codex-rs/cli/src/main.rs`                                                         |
| TUI startup and shared server connection | `codex-rs/tui/src/lib.rs`                                                          |
| Terminal app and chat view               | `codex-rs/tui/src/app.rs`, `codex-rs/tui/src/chatwidget.rs`                        |
| Composer, paste, footer                  | `codex-rs/tui/src/bottom_pane/`                                                    |
| Slash commands                           | `codex-rs/tui/src/slash_command.rs`                                                |
| Task-scoped session handoff              | `codex-rs/tui/src/app/handoff.rs`                                                  |
| Noninteractive execution                 | `codex-rs/exec/src/lib.rs`                                                         |
| Thread/session runtime                   | `codex-rs/core/src/thread_manager/`, `codex-rs/core/src/session/`                  |
| Model requests and tools                 | `codex-rs/core/src/model_request.rs`, `codex-rs/core/src/tools/`                   |
| Effective config and editing             | `codex-rs/core/src/config/`, `codex-rs/config/`                                    |
| Login                                    | `codex-rs/login/`                                                                  |
| MCP                                      | `codex-rs/core/src/mcp.rs`, `codex-rs/rmcp-client/`                                |
| App-server contracts                     | `codex-rs/app-server-protocol/src/`                                                |
| SQLite and migrations                    | `codex-rs/state/src/`, `codex-rs/state/migrations/` and sibling migration dirs     |
| Prompts and instruction discovery        | `codex-rs/core/templates/`, core prompt Markdown, `codex-rs/core/src/agents_md.rs` |
| Execution and sandbox                    | `codex-rs/exec-server/`, `codex-rs/sandboxing/`, platform sandbox crates           |

## Tests and generated files

Core and TUI have inline/module tests plus `tests/all.rs`, `tests/suite/`, and
support/fixture directories. App-server protocol exports and config/hook schemas
are generated. Use [Generated code](generated-code.md) before editing them.

## Sources

- `codex-rs/Cargo.toml`
- `codex-rs/core/src/lib.rs`
- `codex-rs/tui/src/lib.rs`
- `justfile`
- `pnpm-workspace.yaml`
