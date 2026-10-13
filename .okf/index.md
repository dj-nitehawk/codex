---
okf_version: "0.1"
---

# Codex fork knowledge

Compact, evidence-backed operational knowledge for agents working in this fork.
Expanded shape: core workspace guidance plus focused generation, debugging, and
fork-sync playbooks. One root set covers the repository; nested agent contracts
retain their local scope.

## Choose a task path

Apply the reading gates in [AGENTS.md](../AGENTS.md), then use these entry points:

| Task                                         | Read                                                                        |
| -------------------------------------------- | --------------------------------------------------------------------------- |
| Build, install, or Code Mode troubleshooting | [Workflows](workflows.md)                                                   |
| TUI or fork feature                          | [Code map](code-map.md), [Operations](operations.md), [Testing](testing.md) |
| Config, authentication, or saved state       | [Architecture](architecture.md), [Operations](operations.md)                |
| Protocol or schema change                    | [Generated code](generated-code.md), [Testing](testing.md)                  |
| Dependency or toolchain update               | [Dependencies](dependencies.md)                                             |
| Runtime investigation                        | [Debugging](debugging.md)                                                   |
| Upstream sync or customization ownership     | [Fork sync](fork-sync.md)                                                   |
| Knowledge maintenance                        | [Maintenance](maintenance.md)                                               |

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
