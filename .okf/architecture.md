---
type: Architecture
title: Architecture
description: Shared runtime, client, execution, and persistence boundaries.
tags: [architecture]
---

# Architecture

## Components

```text
codex CLI dispatch
  -> TUI or noninteractive exec
  -> app-server client / embedded or connected app-server
  -> core thread/session orchestration
  -> model requests, tools, approvals, and executor infrastructure
```

- `cli/src/main.rs` dispatches commands and interactive startup.
- `tui/` owns terminal interaction and rendering. Its startup supports embedded
  app-server and connected targets.
- `exec/` owns noninteractive execution, request/notification handling, and
  human-readable or JSONL output.
- `app-server/` serves thread/turn operations; `app-server-protocol/` defines its
  client contracts. Client and transport crates support this boundary.
- `core/` contains thread management, session orchestration, context,
  compaction, model requests, tools, and policy integration.
- `exec-server/`, sandboxing crates, and `network-proxy/` handle execution and
  isolation concerns. Inspect the selected backend before changing policy.

These are navigation boundaries. Check per-crate manifests for precise
compile-time dependencies before moving code.

## Persistence

- `rollout/` and core rollout integration support session history.
- `state/` owns SQLite runtime, connection helpers, telemetry, and migrations.
- `thread-store/` defines `ThreadStore`, `LocalThreadStore`, and `LiveThread`.
  Local history uses rollout JSONL; queryable metadata uses SQLite when available.
  Raw history appends preserve canonical items. Explicit metadata patches go
  through `ThreadStore::update_thread_metadata`; active sessions use `LiveThread`.
  Keep metadata observation semantics above the storage boundary. See
  `codex-rs/thread-store/README.md` for its persistence contract.
- State changes can affect list, read, resume, fork, archive, and client views.
  Verify affected queries and notifications alongside writes.

## Security and compatibility invariants

- Preserve configured approval, filesystem, and application network policy
  across startup and reload paths. App-server documentation describes network
  policy installation shared with TUI and exec.
- Preserve protocol compatibility, experimental API opt-ins, and serialization
  behavior when changing public types; regenerate derived schemas and SDK types.
- SQLite pool/connection creation goes through `codex-state`'s shim, enforced by
  workspace Clippy rules.
- Sandbox integrity checks have a telemetry-only scope governed by their local
  `AGENTS.md`; findings must preserve execution and permission behavior.

## Sources

- `codex-rs/cli/src/main.rs`
- `codex-rs/tui/src/lib.rs`
- `codex-rs/exec/src/lib.rs`
- `codex-rs/core/src/lib.rs`
- `codex-rs/app-server/README.md`
