---
type: Reference
title: Gotchas
description: Non-obvious workspace, state, and scoped instruction constraints.
tags: [gotcha]
---

# Gotchas

- Root `just` commands normally change into `codex-rs/`. Plain Cargo commands
  from the repository root need `--manifest-path codex-rs/Cargo.toml` or a cd.
- Root formatter scripts and globs have different scopes. Explicitly include
  `.okf/*.md` when checking knowledge with Prettier.
- Building only `codex-cli` leaves out `codex-code-mode-host`. Code Mode can
  fail closed. For host installation, transport compatibility, and V8
  source-build limitations, see
  [Workflows](workflows.md#code-mode-companion).
- Native TUI and exec share app-server infrastructure. Trace shared contracts
  when changing session state, config, or runtime behavior.
- Named config profiles currently use separate `<name>.config.toml` files.
  Check source when examples from hosted/older documentation differ.
- An explicit `CODEX_HOME` directory must exist before launch. Use an isolated
  home for stateful manual testing when practical.
- Nested TUI startup can fail the socket-directory owner/mode check.
  The reserved `/tmp/codex-daemon-<uid>`
  directory must be a real directory owned by the effective user with mode
  `0700`; sandboxes must hide this socket directory. A sandbox-visible mode
  such as `0000` does not establish the host directory's permissions. Do not
  chmod or unmask it to bypass isolation. Verify outside the sandbox when
  authorized, or report the manual interaction check as blocked.
- App-server schema regeneration can update Python SDK outputs and needs uv.
- Cargo/Bazel and package scripts are evidence for commands. Successful builds,
  test execution, local tool availability, and fork deployment remain separate
  verification obligations.
- Many `/status` snapshots and the daemon-menu CLI confirmation expect
  `v0.0.0`, unlike the release-versioned fork. Review version-line diffs rather
  than accepting unrelated bulk snapshot updates for local UI work.
- Use the recorded release baseline, not `main`, for fork comparisons under
  [Fork sync](fork-sync.md). Post-tag client changes can require
  API fields the live backend rejects even when the workspace advertises the
  release version.

## Sources

- `justfile`
- `package.json`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/uds/src/daemon_directory.rs`
- `codex-rs/app-server-protocol/scripts/write_schema_fixtures.py`
- `codex-rs/tui/src/status/snapshots/`
- `codex-rs/tui/src/app/snapshots/codex_tui__app__daemon_menu__tests__daemon_cli_confirmation.snap`
- `codex-rs/Cargo.toml`
