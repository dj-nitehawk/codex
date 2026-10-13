---
type: Reference
title: Project Overview
description: Purpose and supported surfaces of the Codex fork.
tags: [overview]
---

# Project overview

## Purpose and scope

Codex is OpenAI's local coding agent. This fork is hosted at
`https://github.com/dj-nitehawk/codex.git` and retains upstream CLI, terminal UI,
app-server, sandbox, and SDK infrastructure.

Consumers include interactive terminal users, automation using `codex exec`,
app-server clients, and TypeScript/Python SDK users. Capabilities include coding
turns, tool execution, approvals, MCP integrations, and thread start/resume/fork.

## Repository model

- Native implementation: the Cargo workspace in `codex-rs/`.
- npm CLI entry point: `codex-cli/`, packaging/launching the native CLI.
- Client libraries: `sdk/typescript/`, `sdk/python/`, and `sdk/python-runtime/`.
- Build systems: Cargo and Bazel; pnpm supports JavaScript packages and tooling.
- License: Apache-2.0.
- Fork policy: local `main` and `origin/main` mirror upstream `main`; `dev`
  holds customizations above the latest published Rust release tag, including
  prereleases.
  See [Fork sync](fork-sync.md) for the current area inventory.

## Boundaries

This set describes verified Codex behavior and the user-approved fork policy.
Feature porting decisions require separate discovery and user approval.
Upstream accepts issue reports and analysis; external code PRs are closed under
`docs/contributing.md`.

## Sources

- `README.md`
- `codex-rs/Cargo.toml`
- `codex-cli/package.json`
- `docs/contributing.md`
- `sdk/`
