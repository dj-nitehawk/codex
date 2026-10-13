---
type: Reference
title: Conventions
description: Style, scoped contracts, and fork change ownership.
tags: [conventions]
---

# Conventions

## Native style

- Rust workspace edition is 2024; formatter uses item-granularity imports.
  Match neighboring code and let the repository formatter enforce layout.
- Prefer narrowly scoped changes, existing helpers, and clear regression tests.
- Workspace Clippy permits `expect` and `unwrap` in tests and flags holding
  Tokio mutex/RwLock guards across await.
- SQLite connections and pools use the `codex-state` shim. Preserve the Clippy
  restriction when introducing persistence code.

## Terminal UI

Follow `codex-rs/tui/styles.md`: use shared accent, status, secondary-text, and
key-hint helpers. Prefer theme-aware/default text and semantic ANSI colors.
Workspace Clippy restricts several direct color methods; use the existing style
helpers for compatible rendering across terminal palettes.

Composer/paste state-machine changes must keep module docs synchronized under
`codex-rs/tui/src/bottom_pane/AGENTS.md`.
Sandbox integrity changes follow the scope and privacy contract in
`codex-rs/exec-server/src/sandbox_integrity/AGENTS.md`.

## Fork customizations

- Work on `dev`. [Fork sync](fork-sync.md) owns branch/release policy, the
  recorded baseline, customization areas, and sync authorization/safety gates.
- Keep one commit per customization area, folding updates into the existing
  owner when commits are authorized. Follow the playbook for history regrouping.
- OKF guidance belongs to the `setup okf` area. Dependency and lockfile changes
  belong to the feature or dependency area that produced them.
- Preserve pre-existing user work. Commit or publish when requested.

## Documentation

Write compact operational facts and link canonical sources. Keep secret values,
user transcripts, and machine-specific credentials outside repository docs.
Use direct prose with commas, parentheses, or separate sentences.

## Sources

- `codex-rs/Cargo.toml`
- `codex-rs/rustfmt.toml`
- `codex-rs/clippy.toml`
- `codex-rs/tui/styles.md`
- `AGENTS.md`
