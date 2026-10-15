# Changelog

## [Unreleased]

## [0.13.0] - 2026-10-15

### Added

- Ready-made binaries for Windows, Linux and macOS come with each
  release, so you can install heeded without a Rust toolchain.
- `heeded --version` says which build you have.

### Changed

- Where you would be asked before a commit or push, a subagent is now
  refused and told to report back, because a background subagent's
  request never reached you and went through unasked.

### Fixed

- Writing a file through the shell no longer asks for your OK just
  because the file's text mentions a git commit or push.

- Commits and pushes made in a git worktree of an opted-in repo are
  now checked like any other, even when the repo's marker is hidden.
  Before, a subagent working in a worktree skipped every check.
- Commits and pushes in the git checkout that holds an opted-in repo's
  notes are now checked by that repo's settings: secrets are caught,
  and a push asks first. The changelog reminder stays out of notes
  commits.
- heeded no longer briefly locks your repo while it checks for changes,
  so your own git commands cannot fail on a lock it holds.

## [0.12.0] - 2026-10-15

### Changed

- heeded is now one fast program instead of a set of Node scripts.
  Each tool call costs about 11 ms in a repo that has not opted in,
  where it took about 46, and about 70 ms after a change in one
  that has, where it took about 170. Node is no longer needed to
  run it.
- Install it with `cargo install --path .` in this repo; the plugin
  calls `heeded` from your `PATH`.
- Checking an ordinary shell command before it runs no longer
  touches git at all.

## [0.11.0] - 2026-10-15

### Changed

- heeded costs a repo that has not opted in almost nothing: it
  no longer runs git there or writes anything for its sessions.
- The placeholder `package.json` that `heeded init` writes is
  now private like the notes, so a public repo does not show a
  new file you did not ask for.

### Fixed

- A commit made after `cd` into another folder is judged by the
  repo in that folder, not the one the session started in.

## [0.10.0] - 2026-10-15

### Added

- `heeded help` explains every command, and `heeded help <command>`
  (or `--help`) explains one.
- `heeded config` now walks you through every setting and asks for
  each. `--global` edits your own config, `--local` edits the repo
  you are in, where you can also drop a value so yours applies.
  `--show` prints what is in effect without changing anything.
- You can name paths the changelog reminder should ignore, such as
  test folders, with `checks.quiet`.

### Fixed

- Looking at a release or listing tags (`gh release view`,
  `git tag -l`) no longer asks for your permission. Only creating,
  uploading, editing or deleting one does.
- A commit or push into another repo with `git -C` is judged by
  that repo's settings, and left alone if that repo is not set up
  for heeded.

### Changed

- Changing heeded's settings from the command line now asks for
  your permission, the same as editing the file does.

## [0.9.0] - 2026-10-10

### Changed

- The reminder to add a changelog line before a commit is now
  off unless you turn it on. Set `checks.changelog` to `true` in
  your config, or in a repo's `.heeded.json`, to get it back.

## [0.8.0] - 2026-10-10

### Changed

- heeded no longer comes with rules of its own. A session is
  told only what you put in `directives` in your config. If
  you relied on the six that shipped before, add them there.

## [0.7.0] - 2026-10-09

### Added

- `heeded config` writes your config file for you, with every
  setting at its default and your name and email taken from
  git, then shows what is in effect in the repo you are in.
- You can give sessions your own standing directions, such as
  a voice or a style, with `directives` in your config or in a
  repo's `.heeded.json`. Each is a line of text or a file, and
  a repo can replace or switch off one of yours by name.

## [0.6.0] - 2026-10-09

### Added

- You are asked before a session commits, pushes, tags or
  publishes in a stamped repo. Set `permission.commit` or
  `permission.push` to `allow` in `~/.heeded/config.json`, or
  in a repo's `.heeded.json`, to stop being asked.
- A commit is refused if it contains what looks like a key,
  token or password, or a credentials file.
- You can set the name and email commits must carry, and
  whether they may, must or must not credit a model, with
  `identity` in the same config.
- A session is reminded once to add a changelog line when a
  commit has none.
- `heeded retro` and `/heeded:retro` log whether a finished
  task was kept, reworked or discarded, and why.
- A subagent's work is now logged with the session that
  started it, including work in its own git worktree.

### Changed

- A session is no longer made to write the handoff while its
  subagents are still running. It is asked when the last one
  is back.

## [0.5.0] - 2026-10-09

### Changed

- The project is now called heeded. The command is `heeded`,
  a repo opts in with `.heeded.json`, the default notes folder
  is `.heeded`, and the skills are `/heeded:init` and
  `/heeded:handoff`. A repo set up under the old name needs
  its marker file renamed.

## [0.4.0] - 2026-10-09

### Added

- A session in a stamped repo is told your six standing rules
  when it starts: no em or en dashes, tests stay off live
  systems, no credentials in commits, ask before committing or
  pushing, keep a changelog, and write it for the reader.
- `heeded init --tracked`, for when you want a repo's notes
  committed with it.

### Changed

- `heeded init` now keeps the notes it creates out of git. They
  are hidden in that clone only, and your `.gitignore` is not
  touched.

## [0.3.0] - 2026-10-09

### Added

- A repo's notes can live in any folder. `.heeded.json` at the
  root names it, and `heeded init --notes <folder>` sets it.
- A session is told where the notes are when it starts.

### Changed

- A repo now opts in with `.heeded.json`. Having a
  `docs/handoff.md` no longer does anything.
- `heeded init` no longer writes `CLAUDE.md`, `docs/` or
  `.gitignore`. The default notes folder is `.heeded`.

## [0.2.0] - 2026-10-09

### Added

- `heeded init` sets a repo up, without overwriting anything.
- `heeded moves` lists what changed since the last handoff.
- `/heeded:init` and `/heeded:handoff` do the same from inside a
  session.

### Changed

- When a session is refused at stop, it is told to use the
  handoff skill.

## [0.1.0] - 2026-10-09

### Added

- A session cannot stop while it has changed the repo since
  the handoff was last written.
- Every change a session makes is logged to a journal.
