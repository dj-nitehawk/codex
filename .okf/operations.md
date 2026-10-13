---
type: Playbook
title: Operations
description: Local configuration, credentials, persistence, and diagnostics.
tags: [ops, security]
---

# Operations

## Run model

The native CLI serves interactive TUI, noninteractive exec, and app-server
commands. TUI can embed a server or connect to another target. Treat remote
endpoints, transport/auth settings, and daemon lifecycle as explicit config;
inspect command parsing and startup code for the selected mode.

## Configuration and credentials

- Effective configuration is assembled in `codex-rs/core/src/config/` with
  supporting types/loaders under `codex-rs/config/`.
- Codex home defaults to `~/.codex`, overridable by `CODEX_HOME`.
  An explicit home override must point to an existing directory.
- User config is `config.toml`; current named profiles use `<name>.config.toml`
  with `--profile <name>`. The legacy top-level `profile` selection is rejected.
- Fork parent-folder trust: exact cwd/project/repository decisions retain their
  precedence, then the nearest configured ancestor of the canonical cwd supplies
  `trust_level`. This covers separate/nested repositories. Explicit project
  `untrusted` overrides inherited trust. Canonical ancestry prevents inheriting
  through a symlink to an outside target; use the real parent path. If
  canonicalization fails, original-path ancestry is used. Runtime config and
  both project-layer loaders share the ancestor matcher in
  `codex-rs/config/src/project_trust.rs`. This does not expand sandbox roots.
  See [`docs/config.md`](../docs/config.md).
- Authentication supports ChatGPT sign-in and API-key setup. Credential storage
  and MCP OAuth policy are configurable; inspect login/config code for the
  selected backend before changing credential behavior.
- Admin managed requirements constrain runtime config. For example,
  `allow_managed_hooks_only` belongs in `requirements.toml`.
- For isolated manual checks, create a temporary Codex home before setting
  `CODEX_HOME`. Keep authentication and histories separate from ordinary use.

## Quota status display

- The `weekly-limit` status-line and terminal-title items show used weekly quota
  as `week: 17% (reset: 6d16h)`. Reset countdowns use days/hours, hours/minutes
  below one day, and minutes below one hour, truncating smaller units.
- Reset timestamps are retained from the account usage snapshot and compared
  with the current time when the status surface refreshes. Missing reset times
  omit the suffix; elapsed reset times show `0m`.
- The five-hour item and non-weekly secondary-window fallbacks retain their
  remaining-percentage display. Unavailable quota data remains omitted.

## New-session prompts

Fork-only TUI shorthand `~ <prompt>` and `/new [name] ~ <prompt>` carry an initial
message through the existing fresh-session startup/attachment lifecycle for both
checkout choices. Cancelling the picker cannot seed a later session. Core,
protocol, config, and storage schemas are unchanged. See the
[user guide](../docs/new-session-prompt.md) for syntax and parsing rules.

The fork honors `features.worktrees` (default `true`) as a checkout-picker
preference, unlike upstream 0.164.0, which ignores it. Setting `false` makes
`/new`, `/fork`, and `~ <prompt>` use the current directory without prompting.
Managed requirements can pin the preference. Worktree tools, `/worktree`, and
repository-aware discovery remain available independently of this flag.

## Task handoff

`/handoff <task>` is a TUI-only fork command. It summarizes canonical server
history into an isolated, tool-disabled temporary structured request, then starts
a fresh peer thread seeded with the note and task. The parent history is not
mutated or forked; the seed records its source session ID. The destination uses
current session settings, not reloaded new-session defaults. Normal persistence
and ephemeral settings still apply.

Handoff requires an idle writable session without queued follow-up messages.
Esc (outside popups/overlays), widget replacement, disconnect, or a new parent
turn cancels it. Request IDs and parent/model/cwd/turn-count checks reject stale
results. New composer input prevents automatic switching. Temporary requests
use existing cancellation, timeout, and unsubscribe handling.

History scans and excerpts are bounded, retaining the newest available items in
chronological order. Private reasoning and image payloads are omitted. Tasks and
validated notes are size-limited. Older servers fall back from item pagination to
`thread/read(includeTurns=true)` when pagination is unsupported.

Implementation: `codex-rs/tui/src/app/handoff.rs`. User guide:
[`docs/slash_commands.md`](../docs/slash_commands.md).

## Persistence and logs

`/purge` is a TUI-only fork command that confirms deletion of every saved local
session and exits without a resume hint. Close other Codex instances first and
use `codex --no-daemon`; remote and daemon connections reject it. Embedded
shutdown must succeed before cleanup. Incomplete shutdown is now reported by
the in-process server/client rather than silently succeeding. Cleanup failures
exit with an explicit possibly-incomplete-purge error.

Cleanup preserves the primary SQLite database's non-session settings and
projects while clearing thread rows, attachments, tools, feedback, spawn edges,
receipts, and rollout-migration state. It closes runtime pools and removes
secondary runtime databases/sidecars, old database versions, `db-backups`,
session/archived rollout directories, prompt history, name index, snapshots,
session memories, and visualization stores. Configured SQLite and log locations
are honored. The TUI plaintext log is emptied and timestamped recordings and
the current custom recording are removed. Credentials/config/skills/plugins and
project files remain intact. External exports/backups and previous custom
recordings are excluded; deletion does not promise forensic erasure.

Implementation: `codex-rs/tui/src/session_purge.rs` and
`codex-rs/state/src/runtime/purge.rs`. User guide:
[`docs/slash_commands.md`](../docs/slash_commands.md).

State uses the SQLite runtime and migrations in `codex-rs/state/`, alongside
rollout/thread storage. Preserve migration and connection helper semantics.
Keep user histories, databases, and credential files out of commits.

`RUST_LOG` controls tracing. TUI diagnostics use bounded local stores by default;
set `log_dir` to enable plaintext logs, for example:

```sh
codex -c log_dir=./.codex-log
```

The TUI plaintext filename is `codex-tui.log`. `codex exec` defaults to an
error-level log filter. `just log` tails logs from the state database.
Redact sensitive content before sharing diagnostics.

## Distribution

npm packaging lives in `codex-cli/`; repository scripts and GitHub workflows
handle native release/package assembly. `just assemble-codex-package` invokes
local assembly tooling. The approved fork-sync build, installation, and
publication policy lives in [Fork sync](fork-sync.md#required-completion).
Ordinary development commands are in [Workflows](workflows.md).

## Sources

- `codex-rs/core/src/config/mod.rs`
- `codex-rs/cli/src/main.rs`
- `docs/install.md`
- `docs/config.md`
- `justfile`
- `codex-rs/tui/src/status/rate_limits.rs`
- `codex-rs/tui/src/chatwidget/status_controls.rs`
