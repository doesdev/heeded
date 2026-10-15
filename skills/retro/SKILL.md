---
name: retro
description: Log how a finished task turned out as one line in the repo's rework log. Use when the user asks for a retro, or says a finished piece of work was kept, had to be redone, or was thrown away.
---

One line per finished task, so that rework can be counted
later. Log what happened, not what you hoped. If the owner
had to correct the work, it was reworked.

Run exactly this, from inside the repo, filling in the values:

```
heeded retro --task "<what the task was>" --outcome <outcome> --class <class> --note "<one sentence>"
```

`--outcome` is one of:

- `kept`: used as delivered.
- `reworked`: had to be redone or corrected, in part or whole.
- `discarded`: thrown away.

`--class` is why, and is `none` only when the outcome is
`kept`:

- `wrong-context`: the work was built on the wrong files,
  facts or state.
- `wrong-assumption`: something was guessed that should have
  been asked or checked.
- `missing-test`: it broke in a way a test would have caught.
- `style-mismatch`: it worked but did not fit how this repo
  does things.
- `spec-gap`: the request left out something that mattered.

The command refuses anything outside those sets. Pick the
closest; say what did not fit in `--note`. Ask the user
which outcome applies if you are not sure. Do not guess in
your own favour.
