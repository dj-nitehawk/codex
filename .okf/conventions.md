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

- Work on `dev`; keep local `main` and `origin/main` as fast-forward mirrors
  of upstream `main`. Sync `dev` only through the latest published Rust release
  tag, including prereleases, under [Fork sync](fork-sync.md). Use the recorded
  release baseline for customization boundaries and regrouping.
- A fork-sync request authorizes the standard completion flow in
  [Fork sync](fork-sync.md), including routine history regrouping. Ask about
  unexpected divergence, branch resets, and history rewrites outside that flow.
- Keep one commit per customization area. Fold each update into its existing
  owning commit; create a new commit only for a distinct feature/change with no
  existing owner. Preserve exact tip-tree content when regrouping.
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
