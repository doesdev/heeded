# 0032: `heeded update` installs the latest release

Date: 2026-10-16T06:55Z
Status: accepted
Proposed by: human (Andrew)
Ratified by: claude-opus-5-5

## Context

Updating took five steps from the README: find the release,
download the binary for the machine, check it against
`SHA256SUMS`, put it on the `PATH` under the right name, then
`claude plugin marketplace update heeded` and `claude plugin
update heeded@heeded`. Andrew proposed one command for it.

## Assessment

Ratified. Every step is mechanical and the order matters: a
binary newer than its plugin, or the reverse, is the stale
binary problem the handoff already carries. A command does the
steps the same way each time, which a README cannot.

The shape is mine:

- Downloads go through `curl`, a child process, as git does
  (0025). Windows 10 and later, macOS and Linux ship it. An HTTP
  and TLS stack in the binary would be most of its dependencies.
- SHA-256 is written here, about sixty lines checked against the
  standard test vectors, rather than a crate or a different tool
  per system (`certutil`, `shasum`, `sha256sum`).
- The checksum guards against a broken or partial download, not
  a hostile one: the sums come from the same release. Signing is
  what would change that, and the builds are not signed (0028).
- The running binary is replaced where it is. On Windows a
  running file cannot be overwritten but can be renamed, so the
  old one is moved aside to `heeded.old.exe` and removed on the
  next update. Elsewhere the new file is renamed over the old.
  Before the swap the new binary must answer `--version`; a
  binary that cannot run on this machine is never installed.
- It only moves forward. An older or equal release changes
  nothing. `--check` says what would happen and does nothing.
- The plugin steps run `claude` afterwards. When it is missing,
  or a step fails, the command prints what to run by hand and
  still reports the binary as updated. A marketplace added from
  a folder updates from that folder, so a checkout must be at
  the new version too; the command cannot know which kind it is
  and says so.
- Where it downloads from lives in `consts.rs`, with
  `HEEDED_RELEASES_API` and `HEEDED_RELEASES` to point it
  elsewhere, and `HEEDED_CLAUDE` for the `claude` it runs. The
  tests use them to serve a fake release from local files
  (`file://`) and never reach GitHub or the real Claude Code.

Turned down: updating automatically, at session start or in a
hook. A tool that changes itself while it is gating a session
is the wrong surprise.

## Decision

- `heeded update [--check]` finds the latest release, and when
  it is newer than this build downloads the binary for this
  machine and `SHA256SUMS`, checks the sum, checks the binary
  runs, swaps it in, then runs Claude Code's two plugin update
  commands.
- Machines with no release build (anything but Windows x64,
  Linux x64, macOS arm64 and x64) are told so.

## Consequences

- It cannot work while the GitHub repo is private: the release
  is not reachable without a token. It says it could not reach
  the release, with curl's reason.
- A build from source (`cargo install`) is replaced by the
  release build of a newer version.
- The download is in the binary's own folder, so the swap is a
  rename on the same disk.
