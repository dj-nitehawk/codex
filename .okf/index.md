---
okf_version: "0.1"
---

# Codex fork knowledge

Compact, evidence-backed operational knowledge for agents working in this fork.
Expanded shape: core workspace guidance plus focused generation, debugging, and
fork-sync playbooks. One root set covers the repository; nested agent contracts
retain their local scope.

## Core reading order

- [Project overview](project-overview.md): purpose, scope, and consumers
- [Architecture](architecture.md): components, shared boundaries, and persistence
- [Code map](code-map.md): entry points and task locations
- [Conventions](conventions.md): style, scoped instructions, and fork ownership

## Workflow and validation

- [Workflows](workflows.md): setup, build, run, format, and lint
- [Testing](testing.md): targeted tests and regression surfaces
- [Generated code](generated-code.md): authoritative inputs and regeneration
- [Fork sync](fork-sync.md): approved upstream integration and customization areas

## Task-specific

- [Debugging](debugging.md): standalone Rust DebugMCP setup and inspection
- [Dependencies](dependencies.md): toolchains, package management, and patches
- [Operations](operations.md): config, authentication, state, and diagnostics
- [Gotchas](gotchas.md): traps and verification limits
- [Maintenance](maintenance.md): format and update-trigger inventory

## Authority and maintenance

Source, tests, generated artifacts, lockfiles, and manifests establish current
behavior. Verify conflicts, correct knowledge, and report the correction.
Normative reading and finish gates live in [AGENTS.md](../AGENTS.md).
