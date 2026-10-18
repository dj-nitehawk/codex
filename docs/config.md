# Configuration

For basic configuration instructions, see [this documentation](https://developers.openai.com/codex/config-basic).

For advanced configuration instructions, see [this documentation](https://developers.openai.com/codex/config-advanced).

For a full configuration reference, see [this documentation](https://developers.openai.com/codex/config-reference).

## Parent-folder trust (fork feature)

Trust a folder to cover projects below it, including separate or nested Git
repositories:

```toml
[projects."/path/to/projects"]
trust_level = "trusted"
```

Existing exact working-directory, project-root, and repository-root decisions
keep their precedence. If none matches, the nearest configured ancestor supplies
the trust level. Set a project's `trust_level = "untrusted"` to override an
inherited trust decision. This applies to both runtime configuration and loading
project-local config, hooks, and exec policies. It does not make the parent folder
a sandbox writable root; existing sandbox and approval settings still apply.

Inheritance follows the canonical working-directory path, so a symlink inside a
trusted folder does not automatically trust an outside target. Configure the
real parent path when the folder itself is a symlink. If canonicalization fails,
the original path is used. Path components, not string prefixes, determine
ancestry. Windows paths are matched case-insensitively; POSIX paths retain case.

Only trust parent folders whose current and future projects you trust.

## Lifecycle hooks

Admins can set top-level `allow_managed_hooks_only = true` in
`requirements.toml` to ignore user, project, and session hook configs while
still allowing managed hooks from requirements and managed config layers. This
setting is only supported in `requirements.toml`; putting it in `config.toml`
does not enable managed-hooks-only mode.
