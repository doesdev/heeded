// what a shell command is about to do, and what is wrong with a commit.
// Pattern matching: a guard against drift, not against a determined model.

import path from 'node:path'

// git, then any global options, then the subcommand
const WORD = String.raw`(?:[^\s"']|"[^"]*"|'[^']*')+`
const GIT = String.raw`\bgit(?:\s+(?:-[cC]\s+${WORD}|-{1,2}[\w-]+(?:=${WORD})?))*\s+`

const TAGS = new RegExp(String.raw`${GIT}tag\b([^|;&\n]*)`, 'g')
const TAG_READS = /^(-l|--list|-n\d*|-v|--verify|--points-at|--contains|--no-contains|--merged|--no-merged|--sort|--format|--column)(=|$)/
const TAG_WRITES = /^(-[adsfm]+|--delete|--annotate|--sign|--force|--message)(=|$)/

// git tag lists, verifies and creates; only making or deleting one leaves a mark
const tagsWrite = (command) => {
  return [...command.matchAll(TAGS)].some(([, args]) => {
    const words = args.trim().split(/\s+/).filter(Boolean)

    if (words.some((word) => TAG_READS.test(word))) return false

    return words.some((word) => TAG_WRITES.test(word) || !word.startsWith('-'))
  })
}

const matches = (pattern) => (command) => pattern.test(command)

const ACTS = [
  ['commit', matches(new RegExp(String.raw`${GIT}commit\b`))],
  ['push', matches(new RegExp(String.raw`${GIT}push\b`))],
  ['push', tagsWrite],
  ['push', matches(/\b(npm|pnpm|yarn|cargo)\s+publish\b/)],
  ['push', matches(/\bgh\s+(release\s+(create|upload|edit|delete)|pr\s+(create|merge))\b/)],
  ['settings', matches(/\bheeded(?:\.js)?["']?\s+config\b(?![^|;&\n]*--show\b)/)]
]

const TARGET = new RegExp(String.raw`\bgit\s+(?:[^|;&\n]*?\s)?-C\s+(${WORD})`)
const CD = new RegExp(String.raw`(?:^|[;&|(]\s*)cd\s+(${WORD})(?=\s*(?:&&|;))`, 'g')
const GIT_AT = /\bgit\s/

const unquote = (word) => word.replace(/^(["'])(.*)\1$/, '$2')

// a Git Bash path such as /c/Users on Windows
const native = (dir) => {
  if (process.platform !== 'win32') return dir

  return dir.replace(/^\/([a-zA-Z])(\/|$)/, '$1:/')
}

// the folder a git command acts on: its -C, else the last cd before it.
// One with a variable in it cannot be known, so it counts as here.
export const targetOf = (command, cwd) => {
  const flag = command.match(TARGET)
  const before = command.slice(0, Math.max(0, command.search(GIT_AT)))
  const moves = [...before.matchAll(CD)].map(([, dir]) => unquote(dir))
  const steps = flag ? [...moves, unquote(flag[1])] : moves

  if (steps.some((dir) => /[$%`]/.test(dir))) return cwd

  return steps.reduce((at, dir) => path.resolve(at, native(dir)), cwd)
}

const ADDS = new RegExp(
  String.raw`${GIT}add\b|${GIT}commit\b[^|;&\n]*?\s(-[a-zA-Z]*a[a-zA-Z]*|--all)\b`
)

const SECRETS = [
  ['a private key', /-----BEGIN [A-Z ]*PRIVATE KEY-----/],
  ['an AWS access key', /\b(AKIA|ASIA)[0-9A-Z]{16}\b/],
  ['a GitHub token', /\b(gh[pousr]_[A-Za-z0-9]{36,}|github_pat_[A-Za-z0-9_]{40,})\b/],
  ['an Anthropic or OpenAI key', /\bsk-[A-Za-z0-9_-]{24,}\b/],
  ['a Slack token', /\bxox[abprs]-[A-Za-z0-9-]{10,}\b/],
  ['a hard-coded secret',
    /\b(api[_-]?key|secret|token|passw(or)?d)\b["']?\s*[:=]\s*["'][^"'\s]{16,}["']/i]
]

const SECRET_FILES = /(^|\/)(\.env(\.[\w.-]+)?|id_(rsa|dsa|ecdsa|ed25519)|[^/]*\.(pem|pfx|p12|key))$/
const SAFE_FILES = /(^|\/)\.env\.(example|sample|template)$/
const WAIVER = 'heeded: not a secret'
const QUIET = /(^|\/)(test|tests|__tests__|docs|\.github)\/|(^|\/)test\.[cm]?js$|\.md$|(^|\/)\.[^/]+$/

export const CHANGELOG = 'CHANGELOG.md'

export const classify = (command) => {
  const acts = ACTS.filter(([, test]) => test(command)).map(([act]) => act)

  return [...new Set(acts)]
}

// does the command stage things itself, so that the index is not yet the commit
export const stages = (command) => ADDS.test(command)

export const secretIn = (line) => {
  if (line.includes(WAIVER)) return null
  const found = SECRETS.find(([, pattern]) => pattern.test(line))

  return found ? found[0] : null
}

export const secretFile = (file) => SECRET_FILES.test(file) && !SAFE_FILES.test(file)

// added lines of a unified diff, as [file, kind] for each that looks secret
export const scanDiff = (diff) => {
  const found = []
  let file = ''

  diff.split('\n').forEach((line) => {
    if (line.startsWith('+++ ')) file = line.slice(4).replace(/^b\//, '')
    if (!line.startsWith('+') || line.startsWith('+++')) return

    const kind = secretIn(line)
    if (kind) found.push([file, kind])
  })

  return found
}

export const attributionProblem = (command, policy) => {
  const has = /co-authored-by:/i.test(command)

  if (policy === 'forbidden' && has) return 'a Co-Authored-By line, which this repo forbids'
  if (policy === 'required' && !has) return 'no Co-Authored-By line, which this repo requires'

  return null
}

export const identityProblem = (command, actual, wanted) => {
  const overrides = /--author\b|\buser\.(name|email)=|\bGIT_(AUTHOR|COMMITTER)_/.test(command)
  const want = [wanted.name, wanted.email].filter(Boolean)

  if (!want.length) return null
  if (overrides && !want.every((part) => command.includes(part))) {
    return `an author override that is not ${want.join(' ')}`
  }

  const off = [['name', 'user.name'], ['email', 'user.email']]
    .filter(([key]) => wanted[key] && actual[key] !== wanted[key])
    .map(([key, setting]) => `git ${setting} is "${actual[key]}", not "${wanted[key]}"`)

  return !overrides && off.length ? off.join(' and ') : null
}

// files that could change what a user sees, with no changelog beside them
export const changelogOwed = (files, notes, quiet) => {
  if (files.includes(CHANGELOG)) return false

  const hushed = [...(quiet || []), ...(notes ? [notes] : [])]
  const under = (file, prefix) => {
    return file === prefix || file.startsWith(prefix.endsWith('/') ? prefix : `${prefix}/`)
  }

  return files.some((file) => !QUIET.test(file) && !hushed.some((prefix) => under(file, prefix)))
}
