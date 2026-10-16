# 0031: Private notes apart from the notes

Date: 2026-10-16T06:22Z
Status: accepted
Proposed by: human (Andrew)
Ratified by: claude-opus-5-5

## Context

Before making this repo public, a review of its history found
the tracked notes holding things that should not be published:
the handoff and plan described an employer's machine and its
security setup. Andrew asked for that to be handled, and
proposed `privateNotes`, a folder separate from `notes`, with
gitignore handling.

0012 made all notes private by default and `--tracked` all of
them public. A repo had to choose one for everything. This
repo chose tracked, so every handoff, written by a model on
every turn with no review, was published.

## Assessment

Ratified. The two halves of the notes differ in kind. The plan,
decisions and rework log change rarely, are worth their history
and are read by people. The handoff and journal change on every
turn, are for the next session on the same machine (0012), and
are where the sensitive detail collects, because they record
what happened. One switch for both forced the wrong answer on
one half.

What goes private is mine: the handoff and the journal, which
are the files the gate reads and writes. The rework log stays
with the notes, because it is the record the monthly review
reads and wants history.

The gitignore handling is mine too. A new private folder gets a
`.gitignore` holding `*`, which ignores the folder and itself.
It works in every clone, needs no line in the repo's
`.gitignore` (0011 promised not to edit it) and none in
`.git/info/exclude`, which is per clone. The marker is tracked
and already names the folder, so nothing is hidden by keeping
its name out of `.gitignore`. A folder that already exists gets
no `*`: it may be a checkout of its own, which `*` would break.

The ignore file can be removed or forced past (`git add -f`).
So the guard also refuses a commit that carries a file from the
private folder. That is the part that holds when nobody is
looking.

Moving the handoff out of this repo's history does not remove
what is already there. Two commits, both after v0.13.0, hold the
employer detail; rewriting them is Andrew's call.

## Decision

- The marker may set `"privateNotes": "<folder>"`, relative to
  the root or absolute. The handoff and journal live there; the
  plan, decisions and rework log stay in `notes`. Without it,
  everything is in `notes`, as before. A `privateNotes` that is
  not a folder name is an error.
- `heeded init --private <folder>` writes it and stamps the
  handoff there, with a `.gitignore` of `*` when the folder is
  new and inside the repo.
- The guard refuses a commit carrying anything under the private
  folder, and treats a separate checkout holding it as it treats
  one holding the notes (0027).
- The session-start note names both folders and says the private
  one never goes into git.
- This repo sets `privateNotes` to `.heeded`. Its handoff and
  journal moved there; the work machine detail moved from the
  plan to `.heeded/work-machine.md`.

## Consequences

- Amends 0012: a repo can now track part of its notes.
- This repo's handoff is no longer versioned. Losing the working
  copy loses it, as 0012 already accepted for default repos.
- A fresh clone has no private folder, so no handoff, and is not
  gated until `heeded init` makes one (0011's rule).
- A session must be on a binary that knows `privateNotes`, or it
  looks for the handoff in the notes and finds none.
