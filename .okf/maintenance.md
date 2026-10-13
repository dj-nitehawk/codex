---
type: Reference
title: Knowledge Maintenance
description: OKF conformance, update triggers, and conflict resolution.
tags: [maintain]
---

# Knowledge maintenance

## Authority and gates

`AGENTS.md` owns normative reading, content-discipline, and update/finish gates.
This file is the format reference and trigger inventory. Keep a single root
knowledge set; existing nested `AGENTS.md` contracts retain their scope.

## Conformance

- Each non-reserved Markdown file needs YAML frontmatter with non-empty `type`,
  `title`, and `description`.
- Allowed types: Reference, Architecture, Playbook, API Endpoint, Database,
  Service, Event, Security, Deployment, Generated, ADR.
- `index.md` is a router. Only the root index may have frontmatter, containing
  `okf_version: "0.1"` alone.
- `log.md` is reserved and optional; create it only when requested.
- Keep links resolving, claims sourced, and files compact. Omit empty topics.
- Keep at most one primary `resource` field; use short Sources lists for
  multi-source claims.
- Store concept files as Markdown and exclude secrets/user operational data.

## Update triggers

Evaluate OKF under `AGENTS.md`'s update gate after changes to:

- Architecture, dependency directions, ownership, or component boundaries.
- Public APIs, protocol fields, schemas, events, contracts, or compatibility.
- Storage, persistence, migration, or history/state assumptions.
- Dependencies, runtimes, package managers, or toolchain pins.
- Build, run, test, lint, format, generation, packaging, or deployment commands.
- Test strategy, fixture layout, or verification requirements.
- Security, authentication, approvals, or sandbox/network policy.
- Config, environment variable meanings, service endpoints, or operations.
- Code conventions, directory layout, or gotchas.
- Fork ownership, sync policy, or customization area inventory.

## Conflicts and validation

Verify against current source/tests/manifests, correct knowledge, and report
material corrections. Validate frontmatter and local links after knowledge
changes. Assess unaffected status explicitly under the canonical finish gate.

## Sources

- `AGENTS.md`
- `.okf/index.md`
