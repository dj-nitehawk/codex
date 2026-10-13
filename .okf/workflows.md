---
type: Playbook
title: Workflows
description: Verified setup and native build, run, formatting, and lint commands.
tags: [build]
---

# Workflows

## Setup

- Install Rust via rustup. `codex-rs/rust-toolchain.toml` pins `1.95.0` with
  Clippy, rustfmt, and rust-src.
- Install helpers as needed: `cargo install --locked just`,
  `cargo install --locked dotslash`, `cargo install --locked cargo-nextest`.
- Root JavaScript tooling requires Node `>=22` and pnpm `10.34.5` per the root
  manifest. Use the committed pnpm lockfile for installs.
- Python generation/formatting has its own uv-managed projects and lockfiles.
- Root `just` recipes normally run in `codex-rs/`; `[no-cd]` recipes retain
  their explicitly selected working directory. Plain Cargo commands below
  run from `codex-rs/`.

## Build and run

```sh
# From repository root
just codex
just exec "describe this repository"

# From codex-rs
cargo check -p codex-cli
cargo build -p codex-cli
cargo run --bin codex -- "describe this repository"
cargo build -p codex-cli --release
```

Prefer checking the affected crate first. For fast local optimized builds, use
`release-local`:

```sh
# From codex-rs
cargo build --locked -p codex-cli -p codex-code-mode-host --profile release-local -j 4
./target/release-local/codex
```

For a user-requested fork sync, follow [Fork sync](fork-sync.md)'s required
completion flow. A Cargo build alone is not completion of that flow.

## Code Mode companion

Code Mode requires the companion `codex-code-mode-host` executable. For a local
Cargo build, Codex resolves it next to its own executable and starts it
automatically. Build both packages with the same profile, and keep the host
alongside Codex when installing local binaries.

An official distribution host can be reused when its platform, transport, and
protocol match the local CLI. Download the standalone host asset from
[Codex releases](https://github.com/openai/codex/releases), verify the release
asset digest, and install it as `target/release-local/codex-code-mode-host`.
The default process-owned provider uses the host's standard stdio transport.
`features.code_mode_host_grpc` opts into `--listen grpc+stdio://` and defaults
to false. Verify host support before enabling that transport. See
[Gotchas](gotchas.md) for source-build limitations.

Keep the released host aligned with the fork's upstream baseline release,
including prerelease identifiers. After each upstream version update, install
the corresponding release host and verify transport compatibility, protocol
negotiation, and Code Mode execution. Fork-only changes can retain the existing
host while its protocol remains compatible.

Restart Codex after building or replacing the host. Enabling
`features.code_mode_host` requires that executable.

## Local release profile

`release-local` inherits release optimization, disables LTO, sets eight codegen
units, and disables Rust debug information. It retains the inherited
`strip = false`, so symbol/name tables and native-dependency debug sections can
remain. The standard `release` profile retains its existing LTO and debug
settings. Outside the approved fork-sync flow, installation requires a request.

## Rust debugging with standalone DebugMCP

See [Debugging](debugging.md) for full-debug builds, the project-local GDB DAP
configurations, launch/inspection pitfalls, and cleanup.

## Fork build version

- Build customized releases from `dev`, whose baseline is the latest published
  upstream Rust release tag, including prereleases, under [Fork sync](fork-sync.md).
  `main` independently tracks upstream development. Keep `dev`'s
  `codex-rs/Cargo.toml` workspace version identical to its selected tag's version.
  Read the baseline from that playbook's frontmatter. A matching version string
  alone does not make development-tip source equivalent to the release.
- Model discovery sends Cargo's numeric major/minor/patch version to the
  backend. A `0.0.0` source build can receive a reduced model catalog.
- After a version change, run `cargo update --workspace --offline` from
  `codex-rs/`, review the lockfile for workspace-only changes, then rebuild
  with the locked build command above.
  If new upstream dependencies are absent from the local index, retry without
  `--offline` and review any dependency changes as well.
- During approved upstream syncs, update the full version to match the upstream
  release corresponding to the new baseline. Follow the fork-sync build/install
  sequence, then verify the installed `codex --version`, model discovery, and
  `/model` before publication when available. Report sandbox or backend limits
  explicitly; a returned model catalog alone does not prove a fresh backend fetch.

## Format and lint

From the repository root:

```sh
just fmt-check
just fmt
just clippy -p <crate>
just fix -p <crate>
```

`just fmt` covers Rust, Bazel/Starlark, Python SDK code, and Python scripts.
`just fix` applies Clippy fixes, so use it after reviewing its change scope.
For Markdown use the configured Prettier explicitly, e.g.
`pnpm exec prettier --check AGENTS.md '.okf/*.md'` after installing dependencies.
The root `format` script's globs omit `.okf/`.

## Bazel and generation

- `just bazel-codex` runs the native CLI through Bazel.
- `just bazel-test` runs Bazel tests; `just build-for-release` builds release
  binaries via the configured Bazel target.
- Cargo manifests and lockfile supply crate metadata to Bazel; preserve
  associated BUILD targets when changing packaging/data dependencies.
- Remote cache/execution is opt-in. Keep credentials in local configuration.
- See [Generated code](generated-code.md) for schema commands and
  [Testing](testing.md) for targeted validation.

## Sources

- `justfile`
- `docs/install.md`
- `package.json`
- `codex-rs/rust-toolchain.toml`
- `codex-rs/docs/bazel.md`
- `codex-rs/models-manager/src/lib.rs`
- Code Mode host: `codex-rs/code-mode-host/Cargo.toml`,
  `codex-rs/install-context/src/lib.rs`, `codex-rs/core/src/thread_manager.rs`,
  `codex-rs/features/src/lib.rs`
