---
name: handoff
description: Rewrite the repo's handoff.md in its fixed shape from what this session did. Use before stopping after changing a repo stamped for heeded (it has a .heeded.json), and whenever the stop gate refuses.
---

First get the time, where the handoff file is, and the moves
the hook journaled since it was last written. Run exactly
this, from inside the repo:

```
heeded moves
```

Then rewrite the file it names as `Handoff:` whole. Do not
append to it, and do not write a handoff anywhere else.
The shape is fixed:

```
# Handoff

Last session: <the "Now" value> (<agent, such as Claude Code>, <model>)

## Did

## Did not

## Open

## Next
```

- **Did**: what changed, and the proof it works (the command
  run and what it said). Account for every listed move. A
  move marked `outside` was the human's; say so if it matters.
- **Did not**: what was asked for or planned and is not done,
  including anything unverified. Nothing committed counts.
- **Open**: risks, loose ends and questions for the next
  session. Carry forward what is still true from the old
  file; drop what no longer is.
- **Next**: the one thing to start on.

Write for a reader with none of this conversation. Never take
the time from memory. Make this the last change of the turn:
any move after it makes the file stale again.
