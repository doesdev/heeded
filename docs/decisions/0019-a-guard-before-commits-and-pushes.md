# 0019: A guard runs before every commit and push

Date: 2026-10-09T06:57Z
Status: accepted
Proposed by: model (claude-opus-5-5), building what 0013 and 0015 said was owed
Ratified by: claude-opus-5-5

## Context

0013 listed enforcement its rules were owed: a secret scan
heeded ships, a commit and push gate, a changelog check. 0015
put permission and commit identity in config and said a hook
should read it. Andrew was away when this was built; the
mechanism below is his to overturn.

## Decision

A pre-tool hook (`lib/guard.js`) looks at each shell command
before it runs, in a stamped repo only.

- Settings are the defaults (`ask` for commit and push, no
  identity, attribution `any`), then `~/.heeded/config.json`,
  then the repo's `.heeded.json`, each layered on the last.
  Keys: `permission.commit`, `permission.push`,
  `identity.name`, `identity.email`, `identity.attribution`
  (`any`, `required` or `forbidden`).
- A commit is refused outright, with the reason given to the
  model, when: something in it looks like a secret (file and
  kind are named, never the value); the git author is not the
  configured identity; or the attribution policy is broken.
- A commit that touches no `CHANGELOG.md` but does touch
  something a user could see is refused once, with the rule.
  The same command run again goes through. The model decides
  whether the change is user-visible; the hook only makes
  sure it was asked.
- Then permission. Where the setting is `ask`, the harness
  asks the owner, not the model. `push` covers `git push`,
  `git tag`, package publishes and `gh` releases and PRs.
- An edit to either settings file asks the owner, since
  otherwise a model could grant itself permission.
- A line can be waived from the secret scan with the comment
  `heeded: not a secret`.

## Consequences

- The `permission` rule no longer defers to a repo's written
  instructions. Config is the only way to loosen it.
- In a headless session there is nobody to ask, so `ask`
  refuses. Seen in a real run: with `Bash(git:*)` allowed,
  the commit was still stopped.
- An interactive prompt has not been seen yet. Only the
  refusals were run for real.
- The scan sees the index, and the working tree when the
  command stages for itself. A commit made by a script, an
  alias or another tool is not seen at all.
- Editing settings through a shell command (`sed`, `echo`)
  is not caught. Only the edit tools are.
- One more process start before every shell command and
  edit, about 0.3s on Windows.
- The secret patterns will have false positives. The waiver
  comment is the only way past one.
- The Cursor mapping (`beforeShellExecution`) is written from
  docs and untested there.
