---
name: init
description: Stamp the current repo for heeded (a .heeded.json marker, a notes folder with handoff, plan, decisions and rework log, a check script). Use when the user asks to set up, stamp or opt in a repo to heeded.
---

Run exactly this, from inside the repo to stamp:

```
heeded init
```

`heeded` is on the `PATH`; if the command is not found, tell
the user it needs installing (`cargo install --path .` in the
heeded repo) rather than looking for another way. To stamp a
repo other than the current directory, add its path as one
last argument.

By default the notes go in a `.heeded` folder at the root and
are private: the marker and the folder are hidden from git in
this clone, so they are not committed, not versioned and not
in a fresh clone. Three flags change that, and all are the
user's call, not yours:

- `--notes <folder>` puts the notes somewhere else. Offer it
  when the repo already keeps status, plans or decisions
  somewhere, or has a private checkout for them. Never choose
  `docs/` yourself.
- `--tracked` leaves the notes visible to git so they are
  committed with the repo. In a public repo that publishes
  them. Use it only when the user asks.
- `--private <folder>` keeps the handoff and journal in a
  folder of their own that git never sees, so the plan and
  decisions can be tracked without them. Offer it with
  `--tracked` when the repo is public or may become so.

It never overwrites. Each file is reported as `created`,
`kept` or `updated`. Tell the user which were which. It
writes the marker, the notes folder, the private folder if
asked for and, only if there is none, a `package.json`. The only other file it can touch is
`.git/info/exclude`. Not `CLAUDE.md`, not `.gitignore`.

Then:

- If it says `package.json` has no `check` script, ask what
  proves this repo works, and what a check must never do
  (touch hardware, hit a live service). Wire that up as
  `check`. Do not invent a check that passes without testing
  anything. If the repo already has a command that does this
  (`just check`, `cargo test`), offer to name it as
  `checks.oracle` in `.heeded.json` instead.
- A `created` `plan.md` is a blank template. Offer to write
  it with the user; do not fill it with guesses.
- The repo is now gated: a session that changes it cannot
  stop until the handoff (in the private folder, if there is
  one) is rewritten.
