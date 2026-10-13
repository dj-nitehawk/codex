# Slash commands

For an overview of Codex CLI slash commands, see [this documentation](https://developers.openai.com/codex/cli/slash-commands).

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
