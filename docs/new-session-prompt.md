# Start a new chat with a prompt

This fork's terminal UI accepts an initial text prompt after a standalone `~`:

```text
~ Explain this repository
/new Login fix ~ Fix the login bug
```

Type `~ <prompt>` and press Enter to start an unnamed fresh chat and submit
the prompt. No `/new` prefix is required. `~` must be at the start of the draft
and followed by whitespace. `/new ~ <prompt>` remains compatible.

`/new` still opens an unnamed empty chat. `/new My chat` still opens an empty
chat named `My chat`. With a separator, the text before `~` is an optional name,
and the text after it is automatically submitted as the new chat's first user
message. This is a fresh conversation, not a copy or summary of the old one.

The first `~` surrounded by whitespace (or at an argument boundary) is the
separator. `~/project`, `a~b`, and `~word` are not separators. An empty prompt
after a separator is rejected. Leading and trailing prompt whitespace is trimmed;
internal newlines, further tildes, shell characters, and slash-command-looking
text are literal prompt content. No shell quoting is required.

When the Git worktree picker is available, either checkout choice carries the
prompt to the destination. Cancelling the picker does not submit it or retain it
for the next new chat. Existing permission and startup checks still apply.
The prompt uses normal message persistence and is available when resuming a
saved chat. Ephemeral chats retain their existing persistence behavior.

This extension supports text prompts, not transferring image attachments or
structured mentions from the old composer. `/fork`, `/clear`, and `/handoff`
are unchanged.
