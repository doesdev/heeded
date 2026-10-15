# 0009: `heeded init` ships as a global CLI and in the plugin

Date: 2026-10-09T05:51Z
Status: accepted
Proposed by: human (Andrew)
Ratified by: claude-opus-5-5

## Context

Session 1 left open whether the scaffold command lives in the
plugin or in a separate small CLI. Andrew wants both: a
global CLI on his machine as well as the plugin.

## Assessment

Ratified. Stamping a repo happens before any session exists
in it, so a shell command is the natural entry point, and a
session should be able to do the same thing without leaving.
Whether he wants a CLI on his machine is also his call as
owner, not a question about guiding models.

One condition added: a single implementation. Two code paths
that stamp repos would drift, and the scaffold is the
interface every other layer depends on.

## Decision

This package gains a `bin` named `heeded`; `npm link` (or a
global install) puts it on the PATH. The plugin exposes the
same command to a session as a skill that runs that bin.
Scaffold logic lives in `lib/` and is called from one place.

## Consequences

- The plugin and the CLI are versioned together.
- The CLI needs Node, already required by 0002.
- Not built yet. Session 2.
