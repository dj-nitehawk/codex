# Agent instructions

This repository is a fork of OpenAI Codex, a local coding agent with a Rust CLI,
terminal UI, app-server, and SDKs.

## OKF knowledge set

This repository uses `.okf/` as compact operational memory for AI agents.

OKF v0.1: non-reserved Markdown files require YAML frontmatter with non-empty
`type`, `title`, and `description`. `index.md` is a directory listing; only the
bundle-root index may have frontmatter (`okf_version`).

Normative day-to-day OKF gates live here. `.okf/index.md` routes reading;
`.okf/maintenance.md` supplies the trigger inventory and conformance detail.

### Before work

Match reading depth to the change's blast radius:

- Local change: read `.okf/index.md`, conventions, gotchas, and relevant task files.
- Cross-cutting change, public contract, persistence, authentication, or new
  surface: read overview, architecture, code-map, and conventions first, followed
  by the applicable workflows, testing, operations, dependencies, and generation
  guides.
- Check for nearer `AGENTS.md` files. Existing scoped contracts apply in
  `codex-rs/tui/src/bottom_pane/` and
  `codex-rs/exec-server/src/sandbox_integrity/`.

Confirm exact behavior against source, tests, and manifests.

### During work

Preserve documented boundaries and conventions. When knowledge conflicts with
source, tests, generated artifacts, lockfiles, or manifests, verify current
behavior, correct OKF, and mention the correction in the final response.

### OKF content discipline

- Keep OKF as operational memory, not a change log or second documentation site.
- Add facts that guide future decisions: boundaries, contracts, commands,
  non-obvious failure modes, or owner-approved policies. Omit routine
  implementation details and exhaustive API/test inventories.
- Give each detailed fact one topic owner. Elsewhere, use a short summary or
  link only when needed for navigation.
- Update or replace existing guidance instead of appending another account of
  the same behavior. Remove superseded material in the same edit.
- Test guidance identifies relevant suites, commands, setup, and isolation
  requirements, not individual assertions.
- Before finishing an OKF edit, check for duplication, stale detail, and
  unnecessary reading cost. Preserve critical safety, compatibility,
  persistence, and owner-decision constraints.

### Before finishing

Evaluate OKF when a change hits the update triggers in `.okf/maintenance.md`.
Update it only when existing guidance becomes incorrect or incomplete, or the
change introduces durable operational knowledge that future agents would
otherwise need to rediscover. A trigger requires evaluation, not an automatic
addition. If existing guidance remains sufficient, report
`OKF unaffected (existing guidance remains accurate and sufficient)`.

If OKF is unaffected, explicitly state why. For a pure comment, typo, or
formatting edit, use `OKF unaffected (non-behavioral edit)`. Work is complete
when OKF is synced or explicitly assessed as unaffected.

## General

- Make focused changes and follow existing patterns. Ask the user when a
  consequential policy or implementation decision requires their choice.
- Use plain, direct language. Use commas, parentheses, or separate sentences
  instead of em or en dashes in prose.
- Use the smallest relevant check and regression tests from `.okf/testing.md`.
  Report commands run and any verification limits.
- Regenerate derived artifacts through `.okf/generated-code.md`; edit their
  authoritative inputs.
- Follow `codex-rs/clippy.toml`, `codex-rs/rustfmt.toml`, and
  `codex-rs/tui/styles.md` for their respective surfaces.
- Test shared behavior across affected clients and persistence paths. For TUI
  changes, exercise interaction and terminal resizing when practical.
- Keep credentials, user histories, and local operational data out of commits
  and knowledge files.

## Fork ownership

- Local `main` and `origin/main` mirror verified upstream `main` through
  fast-forward updates. `dev` holds customizations above the latest published
  stable upstream `rust-v<major>.<minor>.<patch>` release tag. Exclude all
  prereleases, including alpha, beta, and RC releases. Use `dev`'s
  recorded release baseline for rebases, reviews, and regrouping. See
  `.okf/fork-sync.md` for baseline selection, records, and verification.
- Follow `.okf/fork-sync.md`. A fork-sync request authorizes its standard
  replay of fork customizations onto the selected stable release, including
  non-descendant release histories, folding updates into their owning commits,
  lease-protected `dev` push, backup cleanup, and local release build. Obtain
  separate approval for unexpected remote or branch divergence, baseline rollback
  or repair, regrouping outside that flow, branch resets, or other force pushes.
- Keep one commit per customization area. Fold updates into an existing fork
  commit whenever that area already exists. Create a new commit only for a
  distinct feature/change with no existing owning area.
  Keep `.okf/**` and this file's OKF guidance in the `setup okf` area.
- Commit and publish only when requested. Preserve pre-existing changes.

## Project pointers

- Knowledge router: `.okf/index.md`
- Source build: `docs/install.md`, root `justfile`
- Product documentation: `README.md`, `docs/`
- Upstream contribution policy: `docs/contributing.md`
