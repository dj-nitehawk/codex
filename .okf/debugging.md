---
type: Playbook
title: Rust Debugging
description: Standalone DebugMCP setup, GDB DAP inspection, and cleanup for Rust.
tags: [build, test]
---

# Rust debugging

## Setup and build

Use the `debug-live` skill for the debugging lifecycle. Project-local
`.debugmcp.json` registers `/usr/bin/gdb` in stdio DAP mode
(`--quiet --nx --interpreter=dap`); GDB needs DAP support. Standalone DebugMCP
uses this file, not `.vscode/launch.json`. Preserve existing configuration;
adapter installation or persistent configuration changes require approval.

Build with full debug information for variable inspection, from `codex-rs/`:

```sh
CARGO_PROFILE_DEV_DEBUG=full cargo build --locked -p codex-cli -j 4
CARGO_PROFILE_TEST_DEBUG=full cargo test --locked -p codex-utils-absolute-path --lib --no-run
```

## Launch

- Start DebugMCP with `workingDirectory` at the repository root so project
  registration and `${workspaceFolder}` resolve correctly. Pass an absolute
  Rust source path as `fileFullPath`.
- `configurationName: "gdb"` launches `codex-rs/target/debug/codex` with
  `--help`. Adjust `launch.args` for the investigation. Interactive TUI launches
  need terminal support and are not verified by this configuration.
- `configurationName: "gdb-test"` launches the local, untracked
  `codex-rs/target/debug/debug-test` symlink. After building, point it at Cargo's
  printed test executable, from `codex-rs/`:
  `ln -sfn deps/<printed-test-binary> target/debug/debug-test`.
  Retarget when the crate or executable hash changes. To select one test, set
  `launch.args` to `["<fully-qualified-test-name>", "--exact", "--test-threads=1"]`;
  the default runs the selected executable's tests serially.

## Inspection and cleanup

- Set a breakpoint before launch. Then call `get_debug_status` with
  `waitForPauseSeconds`; a launch completion response can precede the pause.
- Use `scope: "all"` for arguments, which GDB places outside `local`. Read named
  variables and evaluate specific descendants of complex values.
- Evaluations use DAP `repl` context: GDB expects `print <expression>`, not a
  bare expression. Rust structures can expose their raw representation.
- Adapter validation establishes initialization, not application execution.
  If validation reports success but hangs, interrupt only that validation process.
- Stop only sessions/processes and remove only breakpoints created for the task.

## Sources

- `.debugmcp.json`
- `codex-rs/Cargo.toml`
- [GDB DAP contract](https://sourceware.org/gdb/download/onlinedocs/gdb.html/Debugger-Adapter-Protocol.html)
