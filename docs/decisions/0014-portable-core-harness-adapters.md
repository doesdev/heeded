# 0014: A portable core, with one thin adapter per harness

Date: 2026-10-09T06:32Z
Status: accepted
Proposed by: human (Andrew)
Ratified by: claude-opus-5-5

## Context

0001 made this a Claude Code plugin. Andrew, 2026-10-09: at
home he uses Claude Code directly, at work he uses Claude
through Cursor, including its "Auto" model mode, and this has
to stay usable there.

A research pass over Cursor's published docs and forum on
2026-10-09 (not run against a real Cursor) found:

- Cursor has hooks for all four things this plugin does:
  `sessionStart`, `beforeSubmitPrompt`, `postToolUse` (or
  `afterFileEdit` and `afterShellExecution`), and `stop`.
- The shapes differ. Session start must print JSON
  (`additional_context`), not text. Stop is refused by
  printing `followup_message`, not by exit 2 and stderr. The
  session key is `conversation_id`. Tools are `Shell` and
  `Write`, and a shell call carries the command but no
  description.
- Two of the four are reported broken on Windows: injected
  session-start context is dropped by a race, and the stop
  follow-up was reported lost. In `cursor-agent -p` the
  prompt and stop hooks do not fire at all. Cloud agents get
  no session-start hook.
- Cursor imports Claude Code settings, skills and installed
  plugins by default, with its own payload shapes, so this
  plugin may already half-run there.
- Cursor has its own plugin format (`.cursor-plugin/`,
  `hooks/hooks.json`, `skills/`), which can sit beside
  `.claude-plugin/` in one repo.
- Nothing documented makes hooks depend on the model, so
  "Auto" should not matter. What it reports as the model is
  unknown.

## Assessment

Ratified, and it is a constraint on design, not a feature to
add later. Where he works is his to say. What follows from it
is mine:

- Anything that exists only as text a hook injects is not
  portable, because on one harness that text may not arrive.
  0011 moved the notes pointer from a stamped file into the
  session-start hook, and 0013 put the rules there. Both are
  right for Claude Code and both need a second route on
  Cursor. I would not undo 0011: a file the harness reads by
  itself (`AGENTS.md`, a Cursor rule) is a tracked, public
  file in the repo, which 0012 just made the exception.
- The parts that are already portable are the ones that are
  not hooks: the marker, the notes folder, the journal format
  and the `heeded` CLI. The gate's logic (`lib/session.js`) is
  pure. Only `hooks/hook.js` knows a harness.
- I am not building the Cursor adapter from documentation.
  0007: where a claim can be tested, the test decides, and
  two of the four hooks are reported broken on his platform.
  It gets built against a real Cursor on the work machine.

## Decision

- The core is harness-neutral: marker, notes, journal, CLI,
  and the logic in `lib/`. Nothing in `lib/` may read a
  harness's input shape or environment.
- Each harness gets one adapter that turns its hook input
  into a common event (harness, event, session id, dir,
  tool, tool input) and turns the core's answer (context to
  add, a refusal and its reason) into that harness's output.
- A capability a harness lacks degrades to the CLI: the
  model, or the person, runs `heeded moves` and writes the
  handoff unprompted, and the gate is simply absent.
- New features are designed against the weaker harness first.

## Consequences

- Not built. Today's `hooks/hook.js` is the Claude Code
  adapter and the core in one file, and has to be split.
- The journal's `what` for a shell call is the tool's
  description, which Cursor does not have. Logging nothing,
  or a truncated command, needs deciding; commands carry
  secrets (0008).
- Session state lives in `CLAUDE_PLUGIN_DATA`; Cursor
  documents no equivalent, so it falls back to a folder in
  the user's home.
- If Cursor imports the Claude plugin and also loads a native
  one, each event fires twice. One of them has to yield.
- The pointer and the rules need a route on Cursor that is
  not session-start context. Undecided until the Windows bugs
  are checked on the real machine.
- Features that lean on Claude Code alone (plugin skills,
  `${CLAUDE_PLUGIN_ROOT}`) need a Cursor twin or a CLI path.
