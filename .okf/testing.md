---
type: Playbook
title: Testing
description: Targeted regression commands and shared-surface verification guidance.
tags: [test]
---

# Testing

## Native tests

Rust tests include inline modules, separate `*_tests.rs`, integration suites,
and fixture/snapshot assertions. Core and TUI have `tests/all.rs` aggregators
and `tests/suite/` modules. Workspace dependencies include insta and wiremock.

From the repository root:

```sh
just test -p codex-tui
just test -p codex-core
just test -p codex-app-server-protocol
just test
```

`just test` uses nextest, `--no-fail-fast`, the `local` profile, and
`RUST_MIN_STACK=8388608`. Local app-server integration concurrency is bounded
by `codex-rs/.config/nextest.toml`. Prefer package-scoped runs. Routine runs use
default features as documented by the recipe.

For a single test with Cargo, from `codex-rs/`:

```sh
RUST_MIN_STACK=8388608 cargo test -p <crate> <test-name>
```

If the toolchain's `lld` rejects native dependency debug sections with
`ELFCOMPRESS_ZLIB` because it lacks zlib support, use an installed compatible
linker. To relink only a crate's library-test harness with `mold`, use
`cargo rustc --locked -p <crate> --lib --profile test -- -C link-arg=-fuse-ld=mold`,
then run the emitted test executable with the desired filter from the crate's
directory as `RUST_MIN_STACK=8388608 <test-executable> <test-name>`. The stack
setting must apply to execution, not just compilation. This avoids recompiling
all dependencies with new global `RUSTFLAGS`.

## Handoff regressions

From `codex-rs/`, run:

```sh
RUST_MIN_STACK=8388608 cargo test -p codex-tui --lib app::handoff::tests
RUST_MIN_STACK=8388608 cargo test -p codex-tui --lib slash_handoff
```

These cover bounded server history, structured note validation, cancellation,
stale results, command dispatch, and an embedded-server/model-fixture flow that
starts a fresh resumable peer without altering parent history. Fresh-session
regressions also apply because handoff shares attachment and initial-message
submission with `/new`.

## Other surfaces

Purge regressions, from `codex-rs/`:

```sh
RUST_MIN_STACK=8388608 cargo test -p codex-tui --lib purge
cargo test -p codex-state --lib purge
cargo test -p codex-app-server-client --lib shutdown
cargo test -p codex-app-server --lib in_process_shutdown
```

These cover confirmation/cancellation, narrow/wide popups, remote rejection,
session-file cleanup, settings preservation, symlink safety, missing/unavailable
stores, and honest shutdown errors. Use an isolated `CODEX_HOME` for interactive
purge tests, never a user's real session store.

- TypeScript SDK: from `sdk/typescript/`, `pnpm test` uses Jest;
  `pnpm run lint` and `pnpm run build` validate lint and build surfaces.
- Python SDK: from `sdk/python/`, `uv run --group test pytest` uses the project's
  pytest configuration. Some tests exercise real app-server integration;
  inspect the selected tests before assuming external dependencies.
- Repository GitHub helpers: `just test-github-scripts` runs unittest discovery.
- Bazel: `just bazel-test` for relevant workspace-wide validation.

## Regression expectations

- Test changed behavior plus failure, empty, and compatibility states.
- Shared runtime/config/protocol changes require the affected TUI, exec,
  app-server, and SDK readers to remain consistent.
- Persistence changes require checking the affected list/read/resume/fork and
  notification paths. Verify schema/migration behavior when storage changes.
- TUI changes require interaction checks plus narrow/wide terminal layouts when
  practical. Update focused rendering snapshots after reviewing their diffs.
- Check tests scoped to policy and selected platform when changing sandboxing.
- Record the commands actually run and limitations; documented commands alone
  establish a workflow, not a passing result.

## Sources

- `justfile`
- `codex-rs/.config/nextest.toml`
- `codex-rs/core/tests/`
- `sdk/typescript/package.json`
- `sdk/python/pyproject.toml`
