# 0001: This repo becomes a Claude Code plugin

Date: 2026-10-09T05:39Z
Status: accepted
Proposed by: model (Fable, in the claude.ai chat that wrote the plan)
Ratified by: claude-opus-5-5

## Context

The repo held two role-based orchestration playbooks from May
2026 that were never used. Nothing in them was enforced. The
practices they describe only hold when something installed
makes them the default.

## Decision

`heeded` is a Claude Code plugin: `.claude-plugin/plugin.json`
at the root, hooks in `hooks/`, shared logic in `lib/`. Both
playbooks (the human-driven and the agentic multi-chat
orchestration guides) move to `archive/` and are not read or
maintained.

## Consequences

- The repo now has code, tests and a version.
- The salvage list in the plan is the only part of the
  playbooks that carries forward, and only as enforced rules.
- During development the plugin is loaded with
  `claude --plugin-dir <this repo>`. Distribution (a
  marketplace entry) is undecided and not needed yet.
