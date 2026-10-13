# Slash commands

For an overview of Codex CLI slash commands, see [this documentation](https://developers.openai.com/codex/cli/slash-commands).

## Fork command: `/purge`

Permanently delete every locally saved session, including archived sessions and
subagents, then exit Codex. The confirmation defaults to keeping your sessions.
Press Esc to cancel, or select **Yes, purge all sessions and exit** to proceed.

Close other Codex instances using the same storage first, and launch with
`codex --no-daemon`. Remote and daemon connections reject `/purge` because their
writers are not owned by the TUI. The embedded server must finish shutdown before
cleanup starts. Shutdown failure cancels the purge; deletion failure exits with
an error reporting that cleanup may be incomplete.

Cleanup removes session transcripts, prompt history, the session-name index,
shell snapshots, session-derived memories, visualizations, session database rows,
and the runtime history, queue, goal, memory, and diagnostic databases. Configured
SQLite storage outside `CODEX_HOME` is included. Older runtime database versions
and automatic `db-backups` recovery copies are also removed. The configured
`codex-tui.log` is emptied; timestamped session recordings in that log directory
and the current opt-in custom recording are removed.

Credentials, configuration, skills, plugins, project files, and current database
project registrations and remote-control settings are preserved. Cleanup does
not follow session-directory symlinks outside the store. Exported copies, old
custom recording paths, provider-side records, and external backups are outside
its scope. This is local deletion, not guaranteed forensic erasure.

## Fork command: `/handoff <task>`

Generate a task-focused context note from the current conversation, open a fresh
session, and automatically submit the note with the task. For example:

```text
/handoff finish the parser and run its regression tests
```

Unlike `/fork`, the new session does not copy the conversation history. Unlike
`/compact`, the parent conversation is unchanged and remains resumable. The new
session keeps the current working directory, model, provider, reasoning effort,
service tier, and permissions instead of reloading new-session defaults.

Use this command while the session is idle and has no queued follow-up messages.
Note generation runs separately with tools disabled. Press Esc to cancel, with
no popup or overlay open. Switching sessions or starting a new parent turn also
cancels the handoff. A new composer draft prevents automatic session switching.

History and note size are bounded, so long conversations may omit older content
and excerpt large tool outputs. The model is instructed to retain only facts
relevant to the supplied task. The note is a summary, not a guarantee that every
detail was preserved.
