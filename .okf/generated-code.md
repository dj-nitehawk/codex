---
type: Generated
title: Generated Code
description: Authoritative inputs and regeneration commands for schemas and derived SDK types.
tags: [build, api]
---

# Generated code

## Schemas and client types

Run commands from the repository root:

| Derived output                         | Authoritative inputs                            | Regenerate                                                         |
| -------------------------------------- | ----------------------------------------------- | ------------------------------------------------------------------ |
| `codex-rs/core/config.schema.json`     | Rust config types and schema crate              | `just write-config-schema`                                         |
| `codex-rs/app-server-protocol/schema/` | Protocol Rust definitions and schema generators | `just write-app-server-schema`                                     |
| Python SDK generated types             | App-server JSON schema and SDK generator        | Standard `just write-app-server-schema` also regenerates SDK types |
| `codex-rs/hooks/schema/generated/`     | Hook types and fixture writer                   | `just write-hooks-schema`                                          |
| `MODULE.bazel.lock`                    | Bazel modules and dependency inputs             | `just bazel-lock-update`                                           |

The app-server helper runs a specific ignored Rust fixture-writing test.
For normal checked-in schema exports, it then invokes the Python SDK generator
with uv's frozen test dependency group. Scratch `--schema-root` exports and
`--experimental` exports skip the checked-in Python SDK update.

## Change discipline

- Edit authoritative types/templates/scripts, regenerate, and review output.
- Preserve public field names, compatibility, experimental opt-ins, and
  serialization tests where applicable.
- Include derived outputs with their source changes. Run protocol/config tests
  for the surface changed.
- For another file marked generated, inspect its header and producing script
  before editing. The table covers known task entry points, not every generated
  artifact in the monorepo.
- Prompt Markdown under core is an input surface. Determine its loader and
  tests before changing prompts.

## Sources

- `justfile`
- `codex-rs/app-server-protocol/scripts/write_schema_fixtures.py`
- `sdk/python/scripts/update_sdk_artifacts.py`
- `codex-rs/core/config.schema.json`
- `codex-rs/hooks/schema/generated/`
