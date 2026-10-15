# 0004: Cross-project conventions live in the plugin

Date: 2026-10-09T05:39Z
Status: amended by 0011 and 0021
Proposed by: model (Fable, in the claude.ai chat that wrote the plan)
Ratified by: claude-opus-5-5

## Context

Conventions that apply to every repo currently live in one
head and in scattered settings. Per-project facts get mixed
in with them, so neither is easy to find or to version.

## Decision

Conventions that hold across projects live in this plugin as
a small, versioned, searchable corpus. State that belongs to
one project (plan, handoff, decisions, rework log) lives in
that project's `docs/`. The plugin never stores per-project
content, and a project never copies the corpus.

## Consequences

- A convention changes in one place and ships with a plugin
  version.
- The corpus does not exist yet. Its format and how a session
  searches it are undecided; that gets its own record when it
  is built.
- Hook runtime state (session baselines) is neither: it goes
  in the plugin's data directory, outside any repo.
