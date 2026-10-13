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

See [Code map](code-map.md) for workspace layout and entry points, and
[Fork sync](fork-sync.md) for branch/release policy and customization ownership.

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
