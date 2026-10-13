---
type: Reference
title: Dependencies
description: Toolchain pins, workspace manifests, and dependency update constraints.
tags: [deps]
---

# Dependencies

## Runtime and tooling

| Surface                                     | Authority                                             |
| ------------------------------------------- | ----------------------------------------------------- |
| Rust 1.95.0, edition 2024                   | `codex-rs/rust-toolchain.toml`, `codex-rs/Cargo.toml` |
| Rust dependency resolution                  | `codex-rs/Cargo.lock`                                 |
| Node >=22 and pnpm 10.34.5 for root tooling | `package.json`                                        |
| JavaScript workspace/resolution             | `pnpm-workspace.yaml`, `pnpm-lock.yaml`               |
| Python SDK >=3.10                           | `sdk/python/pyproject.toml`                           |
| Bazel toolchain/modules                     | `.bazelversion`, `MODULE.bazel`, `MODULE.bazel.lock`  |

The root Node requirement differs from package-specific engines; use each
package manifest when evaluating published consumer compatibility.

## Native libraries

Workspace-managed dependencies include Tokio, clap, serde, reqwest, Ratatui,
SQLx/SQLite, insta, and wiremock. Inspect the manifest for exact versions and
feature flags. Terminal/WebSocket libraries have pinned Git patches under
`[patch.crates-io]`; preserve and evaluate these when upgrading dependencies.

## Update rules

- Change dependency declarations and commit resulting lockfile changes together.
- Cargo defines native crate metadata; update Bazel metadata/lock resolution
  when its imported dependency graph changes.
- pnpm enforces minimum release age, strict dependency builds, and trust policy
  through `pnpm-workspace.yaml`; preserve those controls.
- Python generation uses uv lockfiles and pinned generator dependencies.
- Review Clippy's SQLx connection restrictions when changing the SQLx pin.

## Sources

- `codex-rs/Cargo.toml`
- `package.json`
- `pnpm-workspace.yaml`
- `sdk/python/pyproject.toml`
- `codex-rs/docs/bazel.md`
