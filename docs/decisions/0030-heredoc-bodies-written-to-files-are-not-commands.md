# 0030: Heredoc bodies that cat or tee write out are not classified

Date: 2026-10-15T15:17Z
Status: accepted
Proposed by: model (claude-opus-5-5), from the gazelle-audio session's live test
Ratified by: claude-opus-5-5

## Context

The guard classifies a shell command by matching its whole text.
In gazelle a command that wrote a Node script through a heredoc,
whose body mentioned `git commit`, was taken for a commit and
asked Andrew, though running it committed nothing. Scripts, test
fixtures and docs written through the shell often quote git
commands, so this would recur. Andrew agreed to the proposal.

## Assessment

What `cat` or `tee` reads from a heredoc is written to a file,
never run. What a shell, python, node or git reads from one may
be run or used (`bash <<EOF`, `python - <<EOF`, `git commit -F
-`), so those must stay. Quoted strings stay classified as well:
`bash -c "git commit"` commits. Over-asking is the safe side.

## Decision

- Before classifying, drop the body of each heredoc whose command
  starts with `cat` or `tee`, up to its delimiter line. Quoted
  and `<<-` delimiters are understood; here-strings (`<<<`) are
  not heredocs and stay.
- Everything else about the command is classified as before,
  including lines after the heredoc.

## Consequences

- A script written and then run in one command (`cat > a.sh
  <<EOF ... EOF` then `bash a.sh`) is no longer seen to commit.
  The guard was always a check against drift, not against a
  determined model.
- Only classification changes. The identity and attribution
  checks still read the whole command.
