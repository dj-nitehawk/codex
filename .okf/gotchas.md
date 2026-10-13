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
  report `host executable was not found` and fail closed until the companion
  executable is built alongside Codex. See [Workflows](workflows.md).
- Code Mode runtime enables `v8_enable_sandbox`. The pinned V8 `150.4.0`
  Linux x86_64 sandbox archive previously returned HTTP 404; recheck availability
  before choosing a workaround. Preserve the sandbox requirement.
  `V8_FROM_SOURCE=1` downloads Chromium toolchains but alone did not resolve
  missing `third_party/icu/common/icudtl.dat` and
  `third_party/rust/chromium_crates_io/vendor/` in the published crate.
  Source recovery must use V8's pinned dependency revisions. For compatible
  distribution hosts and transport selection, see
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
- SQLite connection construction has a centralized shim enforced by Clippy.
- Sandbox integrity checks are telemetry-only. Read the local scope contract
  before changing preparation/checker behavior.
- Paste/composer state-machine docs are part of the bottom-pane change contract.
- App-server schema regeneration can update Python SDK outputs and needs uv.
- Cargo/Bazel and package scripts are evidence for commands. Successful builds,
  test execution, local tool availability, and fork deployment remain separate
  verification obligations.
- Many `/status` snapshots and the daemon-menu CLI confirmation expect
  `v0.0.0`, unlike the release-versioned fork. Review version-line diffs rather
  than accepting unrelated bulk snapshot updates for local UI work.
- The sole configured Git remote is the user's fork (`origin`). Verify upstream
  `main` directly when refreshing the `main` mirror. Select the latest published
  Rust release tag, including prereleases, separately for `dev`. Its recorded
  release tag/SHA defines rebase, diff, and regrouping boundaries; `main` can be
  ahead, and branch divergence is expected. Post-tag client changes can require
  API fields the live backend rejects even when the workspace advertises the
  release version. See [Fork sync](fork-sync.md).

## Sources

- `justfile`
- `package.json`
- `codex-rs/core/src/config/mod.rs`
- `codex-rs/clippy.toml`
- `codex-rs/uds/src/daemon_directory.rs`
- `codex-rs/app-server-protocol/scripts/write_schema_fixtures.py`
- `codex-rs/tui/src/status/snapshots/`
- `codex-rs/tui/src/app/snapshots/codex_tui__app__daemon_menu__tests__daemon_cli_confirmation.snap`
- `codex-rs/Cargo.toml`
- Code Mode availability: `codex-rs/code-mode/src/remote_session.rs`,
  `codex-rs/core/src/tools/code_mode/mod.rs`
- V8 sandbox feature: `codex-rs/code-mode-runtime/Cargo.toml`; prebuilt assets:
  [rusty_v8 v150.4.0](https://github.com/denoland/rusty_v8/releases/tag/v150.4.0)
- Host transport selection: `codex-rs/core/src/thread_manager.rs`,
  `codex-rs/features/src/lib.rs`,
  `codex-rs/code-mode/src/remote_session/connection.rs`
