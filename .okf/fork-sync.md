---
type: Playbook
title: Fork Sync
description: Upstream-main tracking, release-tag-only dev baselines, and safe synchronization.
tags: [ops, maintain]
baseline_tag: rust-v0.163.0
baseline_commit: 83086188ae0614bd882818ff1aa23f3194962012
---

# Fork sync

## Branch and remote model

User-approved policy:

- Local `main` and `origin/main` mirror upstream `main` through verified
  fast-forward updates. Keep fork customizations on `dev`.
- `dev` holds a linear customization series above the selected upstream Rust
  release tag. Its release ceiling is independent of the development-tip mirror.
- Select the most recently published non-draft `rust-v<semver>` release from
  `openai/codex`, including prereleases. A stable-only baseline requires an
  explicit user request. Verify GitHub release metadata and the peeled tag SHA.
- Rebase `dev` only onto that release tag. Divergence between `main` and `dev`
  is expected; use the recorded release baseline for history and diff boundaries.
- `origin` is `https://github.com/dj-nitehawk/codex.git`. Only `origin` is
  configured; fetch upstream `main` and the selected tag directly from
  `https://github.com/openai/codex.git`.
- This file's `baseline_tag` and `baseline_commit` frontmatter record `dev`'s
  current release base. Verify both against Git before replaying customizations,
  and update both after each release sync. Preserve them during policy-only edits.
- A request to sync the fork authorizes fast-forward refresh/publication of
  `main`, the standard release-tag `dev` rebase, folding updates into their
  owning fork commits, lease-protected `dev` push, backup deletion, and
  release-local build/install below. Complete these without a separate prompt
  for each step. This standing approval is scoped to routine fork syncs.
- Obtain separate approval for unexpected divergence, regrouping outside this flow,
  resets, or force pushes outside this flow. Retain backups until completion;
  use `--force-with-lease`, never an unprotected `--force`.
- Normal editing/setup permission does not authorize Git history or publication.

## Preflight

Commands use Bash-compatible syntax; run from the repository root.

```sh
set -euo pipefail
git status -sb
git remote -v
git fetch origin
EXPECTED_DEV=$(git rev-parse origin/dev)
EXPECTED_MAIN=$(git rev-parse origin/main)
BASELINE_TAG=$(sed -n 's/^baseline_tag: //p' .okf/fork-sync.md)
BASELINE_SHA=$(sed -n 's/^baseline_commit: //p' .okf/fork-sync.md)
BASELINE=$(git rev-parse "$BASELINE_TAG^{commit}")
test "$BASELINE" = "$BASELINE_SHA"
git merge-base --is-ancestor "$BASELINE" dev
UPSTREAM_URL=https://github.com/openai/codex.git
ADVERTISED_MAIN=$(git ls-remote "$UPSTREAM_URL" refs/heads/main | cut -f1)
git fetch --no-tags "$UPSTREAM_URL" refs/heads/main
UPSTREAM_MAIN=$(git rev-parse 'FETCH_HEAD^{commit}')
test "$UPSTREAM_MAIN" = "$ADVERTISED_MAIN"
git merge-base --is-ancestor main "$UPSTREAM_MAIN"
git merge-base --is-ancestor "$EXPECTED_MAIN" "$UPSTREAM_MAIN"
RELEASE_TAG=$(gh api graphql --paginate --slurp \
  -f query='query($endCursor: String) { repository(owner: "openai", name: "codex") { releases(first: 100, after: $endCursor, orderBy: {field: CREATED_AT, direction: DESC}) { nodes { tagName isDraft publishedAt } pageInfo { hasNextPage endCursor } } } }' | \
  jq -r '[.[].data.repository.releases.nodes[] | select(.isDraft == false and .publishedAt != null) | select(.tagName | test("^rust-v[0-9]+[.][0-9]+[.][0-9]+(-[0-9A-Za-z.-]+)?$"))] | max_by(.publishedAt) | .tagName')
test -n "$RELEASE_TAG"
test "$RELEASE_TAG" != null
git ls-remote https://github.com/openai/codex.git \
  "refs/tags/$RELEASE_TAG" "refs/tags/$RELEASE_TAG^{}"
git -c fetch.pruneTags=false fetch --no-tags https://github.com/openai/codex.git \
  "refs/tags/$RELEASE_TAG:refs/tags/$RELEASE_TAG"
TARGET=$(git rev-parse "$RELEASE_TAG^{commit}")
printf 'Selected release: %s (%s)\n' "$RELEASE_TAG" "$TARGET"
git log --oneline "$BASELINE..$TARGET"
git log --oneline "$TARGET..$BASELINE"
git log --oneline "$BASELINE..dev"
git log --oneline origin/dev..dev
git log --oneline dev..origin/dev
git merge-base --is-ancestor "$BASELINE" "$TARGET"
```

Start with a clean working tree. Confirm the fetched tag's peeled SHA matches
OpenAI's advertised ref and that its release is published. Require the recorded
baseline to be an ancestor of `dev`; use its explicit tag/SHA to identify the
customization series. Review every commit above it for the owning fork area;
unexpected post-release upstream commits require explicit repair approval.
A merge-base with development `main` can conceal accidental post-release commits
and must never substitute for that record.
Stop for unexpected remote `dev` changes, `main` divergence from upstream,
a missing/mismatched baseline record, or an older/non-descendant release target.
Baseline rollback or repair requires explicit user approval and a `dev` recovery
branch. Review local-only `dev` work and preserve it before any approved rewrite.
For an explicitly approved non-descendant release, retain the original recorded
`BASELINE` as the customization boundary and the verified release as `TARGET`.
After creating the recovery branch, omit only the `BASELINE`-to-`TARGET` ancestry
gate for that approved pair and run `git rebase --onto "$TARGET" "$BASELINE" dev`.
Keep every other preflight, mirror fast-forward, lease, and verification check.
This approval is specific to that transition, not permission for future rollbacks.
Confirm `/home/SOFTWARE/codex/build-release-local.sh` is executable and the
active filesystem/sandbox policy permits installation into
`/home/SOFTWARE/codex/`. If blocked, report the permission requirement before
rewriting history. Unix write bits alone do not establish sandbox permission.
A user-approved partial sync can proceed through local rebase, OKF updates,
and workspace verification. Keep installation, publication, and backup cleanup
pending, preserve the observed remote lease, and report the incomplete steps.

## Approved update

After preflight for the user-requested sync:

```sh
: "${BASELINE:?Run preflight first}" "${TARGET:?Select a release first}"
: "${UPSTREAM_MAIN:?Verify upstream main first}"
STAMP=$(date +%Y%m%d-%H%M%S)
BACKUP="dev-backup-$STAMP"
git branch "$BACKUP" dev
# Require fast-forward ancestry for routine release-to-release syncs.
git merge-base --is-ancestor "$BASELINE" "$TARGET"
git switch main
git merge --ff-only "$UPSTREAM_MAIN"
git switch dev
git rebase --onto "$TARGET" "$BASELINE" dev
test "$(git rev-parse main)" = "$UPSTREAM_MAIN"
git merge-base --is-ancestor "$TARGET" dev
```

If an ancestry check or the `main` fast-forward fails, stop and inspect. The
explicitly approved non-descendant transition above is the sole exception to
the `BASELINE`-to-`TARGET` ancestry gate.
Resolve `dev` conflicts using the selected release's structure while preserving
customization intent. `git rebase --abort` returns to the pre-rebase `dev` branch;
`main` remains an independent upstream mirror. Update this file's baseline
frontmatter to `RELEASE_TAG` and `TARGET` after the rebase, and fold that update
into `setup okf`. Keep all recovery branches until completion succeeds.

No always-keep prompt or README paths are designated for this fork yet. Ask
before establishing such ownership.

## Verification and organization

- Confirm `main` matches the verified upstream-main snapshot, the recorded
  release tag/SHA matches `TARGET`, and `TARGET` is an ancestor of `dev`. Review
  the full customization change set against the backup.
- Run focused checks/tests for changed and conflicted crates. Expand to CLI and
  affected client surfaces where contracts/runtime changes warrant it.
- Keep exactly one fork commit per customization area. Fold fixes, version
  bumps, tests, docs, and sync adaptations into the existing owning commit.
  Create a new commit only for a distinct feature/change that belongs to no
  existing area, then add it to the inventory. Never retain a separate sync,
  follow-up, or fixup commit for an area that already exists.
- Regroup automatically during each fork sync, before building or publishing.
  Temporary `fixup!` commits plus `git rebase -i --autosquash "$TARGET"` are one
  approach; explicitly move older follow-up commits beside their owning commit
  and mark them `fixup` when their subjects are not autosquash targets.
  Fold workspace version and lockfile updates into the upstream build-version
  commit. Fold all `.okf/**` and canonical OKF guidance into `setup okf`.
- After intentional edits are committed temporarily, create a backup of that
  pre-regroup tip, name its branch in `REGROUP_BACKUP`, and capture
  `git rev-parse dev^{tree}`. Compare with the
  post-regroup tree ID, require exact equality, and stop on mismatch. Preserve
  both the pre-sync and pre-regroup recovery tips until completion succeeds.
  Review `git range-diff "$BASELINE..$BACKUP" "$TARGET..dev"` and
  `git log --oneline "$TARGET..dev"` to verify ownership
  and that no duplicate-area or fixup commits remain. A pure regroup with
  identical tree content does not require rebuilding or rerunning code tests.
- Keep OKF and related canonical instructions in the `setup okf` area.
- Refresh the area inventory when adding, retiring, or renaming customizations.
- Commit only sync-related changes under the standard sync authorization.
  Preserve unrelated user work. Routine regrouping is part of that authorization.

## Required completion

A sync is complete only when the verified upstream-main mirror and release-based
`dev` source are published, temporary backups are removed, and the matching
release-local CLI is installed at
`/home/SOFTWARE/codex/codex`. Do not leave version/lockfile or OKF sync updates
uncommitted. Fold them into their existing customization commits, not new
follow-up commits.

After focused verification and regrouping the sync updates, perform all three
steps below. Build first and delete the backup last so a failed build or push
retains the recovery branch.

```sh
(
set -euo pipefail
: "${EXPECTED_DEV:?Run preflight first}" "${BACKUP:?Create the sync backup first}"
: "${UPSTREAM_MAIN:?Verify upstream main first}"

# Build, strip the install copy, and atomically install the release-local CLI.
CARGO_BUILD_JOBS=4 /home/SOFTWARE/codex/build-release-local.sh \
  --source /home/SOURCE-CONTROL/codex/codex-rs \
  --output /home/SOFTWARE/codex
/home/SOFTWARE/codex/codex --version
/home/SOFTWARE/codex/codex --help >/dev/null

# Publish the upstream mirror with an ordinary fast-forward push.
git push origin main:main
REMOTE_MAIN=$(git ls-remote origin refs/heads/main | cut -f1)
test "$REMOTE_MAIN" = "$UPSTREAM_MAIN"

# Publish only if the remote dev tip still equals the preflight observation.
git push --force-with-lease="refs/heads/dev:$EXPECTED_DEV" origin dev:dev
REMOTE_DEV=$(git ls-remote origin refs/heads/dev | cut -f1)
test "$REMOTE_DEV" = "$(git rev-parse dev)"

# Delete only this sync's backup, after build and publication verification.
git branch -D "$BACKUP"
if [ -n "${REGROUP_BACKUP:-}" ]; then
  git branch -D "$REGROUP_BACKUP"
fi
git status -sb
)
```

Run each step only after the preceding step succeeds. Check the installed
version against the workspace manifest. If the lease fails, stop and inspect
the remote changes; do not refresh the lease just to overwrite them. On build,
install, smoke-test, or push failure, retain the backup and report the blocker.
An alternate output directory is not completion of the requested installation.
Permission restrictions take precedence over this workflow.

The script builds only `codex-cli`, not the Code Mode host. When Code Mode is
enabled, also align and verify the companion host using
[Workflows](workflows.md) before reporting the installation ready. Verify live
model discovery and `/model` after a version update when available, and report
any verification limits.

## Current customization areas

1. `git ignore serena`: `.gitignore` excludes `.serena/`.
2. `setup okf`: root knowledge set and canonical `AGENTS.md`.
3. Local release profile: `codex-rs/Cargo.toml` adds `release-local` with LTO
   disabled, eight codegen units, and Rust debug information disabled.
4. Upstream build version: match the recorded baseline's release version during
   approved syncs, including upstream prerelease identifiers. Refresh workspace lockfile
   versions and verify model discovery after changing it. See
   [Workflows](workflows.md).
5. Weekly quota status display: show consumed quota and the reset countdown as
   `week: 17% (reset: 6d16h)` in the status line and terminal title. Preserve the
   five-hour remaining-percentage display and non-weekly fallback behavior.
   See [Operations](operations.md).
6. Task-scoped handoff: `/handoff <task>` generates a bounded context note and
   seeds a fresh peer session without copying or modifying parent history.
   Keep the feature implementation, tests, and user guide in their own TUI area;
   its operational knowledge stays in `setup okf`. See [Operations](operations.md).
7. Local session purge: `/purge` confirms deletion of all saved local sessions
   and associated state, then exits. Keep the TUI, storage cleanup, shutdown
   safety checks, tests, and user guide in their own area. Operational knowledge
   stays in `setup okf`. See [Operations](operations.md).
8. Standalone Rust debugging: project-local `.debugmcp.json` provides GDB DAP
   CLI and test configurations. Operational knowledge stays in `setup okf`.
   See [Debugging](debugging.md).

Read `baseline_tag` from this file, resolve its commit as `BASELINE`, and check
`git log --oneline "$BASELINE..dev"` and `git diff "$BASELINE..dev"` for actual
committed state. Use that release boundary throughout customization reviews.
This inventory records customization intent; Git establishes committed state.

## Sources

- `AGENTS.md`
- `.gitignore`
- Git remote/branch configuration, release tags, and recorded-baseline-to-`dev` history
- `/home/SOFTWARE/codex/build-release-local.sh`
- User-approved fork policy and standard sync completion flow
