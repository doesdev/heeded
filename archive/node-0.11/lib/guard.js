import { createHash } from 'node:crypto'
import fs from 'node:fs'
import path from 'node:path'
import { MARKER } from '../config.js'
import {
  CHANGELOG,
  attributionProblem,
  changelogOwed,
  classify,
  identityProblem,
  scanDiff,
  secretFile,
  secretIn,
  stages,
  targetOf
} from './commit.js'
import { git, rootOf } from './handoff.js'
import { notesOf, within } from './notes.js'
import { settingsFiles, settingsFor } from './settings.js'

const EDITS = { Edit: true, Write: true }
const SMALL = 256 * 1024

const lines = (text) => (text || '').split('\n').filter(Boolean)

const config = (where, key) => (git(where, ['config', key]) || '').trim()

// what the commit will hold: the index, plus the working tree when the
// command stages for itself
const incoming = (where, wide) => {
  const staged = lines(git(where, ['diff', '--cached', '--name-only']))
  if (!wide) return { files: staged, diffs: [git(where, ['diff', '--cached', '-U0'])], fresh: [] }

  const changed = lines(git(where, ['diff', '--name-only']))
  const fresh = lines(git(where, ['ls-files', '--others', '--exclude-standard']))
  const diffs = [git(where, ['diff', '--cached', '-U0']), git(where, ['diff', '-U0'])]

  return { files: [...new Set([...staged, ...changed, ...fresh])], diffs, fresh }
}

const scanFile = (where, file) => {
  try {
    const full = path.join(where, file)
    if (fs.statSync(full).size > SMALL) return []

    return lines(fs.readFileSync(full, 'utf8')).map(secretIn).filter(Boolean)
      .map((kind) => [file, kind])
  } catch (ex) {
    return []
  }
}

const secretsIn = (where, { files, diffs, fresh }) => {
  const named = files.filter(secretFile).map((file) => [file, 'a credentials file'])
  const inDiffs = diffs.flatMap((diff) => scanDiff(diff || ''))
  const inFresh = fresh.flatMap((file) => scanFile(where, file))
  const all = [...named, ...inDiffs, ...inFresh].map(([file, kind]) => `${file}: ${kind}`)

  return [...new Set(all)]
}

// refuses a commit that breaks a rule, and asks the owner before a commit
// or a push the settings do not allow. Only in a stamped repo.
export const createGuard = (home, store) => {
  const nagged = (id, files) => {
    const key = createHash('sha256').update(files.join('\n')).digest('hex')
    const state = store.load(`${id}-guard`) || {}
    if (state[key]) return true

    store.save(`${id}-guard`, { ...state, [key]: true })

    return false
  }

  const commit = (id, root, where, command, settings) => {
    const coming = incoming(where, stages(command))
    const actual = { name: config(where, 'user.name'), email: config(where, 'user.email') }
    const secrets = secretsIn(where, coming)
    const problems = [
      identityProblem(command, actual, settings.identity),
      attributionProblem(command, settings.identity.attribution),
      secrets.length ? `what looks like a secret:\n${secrets.join('\n')}` : null
    ].filter(Boolean)

    if (problems.length) return `heeded: this commit has ${problems.join('; and ')}`

    if (!settings.checks.changelog) return null

    const notes = notesOf(root)
    const inside = notes ? within(root, notes) : ''
    const notesAt = inside.startsWith('..') ? '' : inside
    const owed = changelogOwed(coming.files, notesAt, settings.checks.quiet)
    if (!owed || nagged(id, coming.files)) return null

    return `heeded: this commit does not touch ${CHANGELOG}. If it changes ` +
      'anything a user can see, do or be affected by, add a line under ' +
      '"## [Unreleased]" first. If it does not, run the same command again.'
  }

  const check = ({ id, dir, cwd, tool, input }) => {
    const root = rootOf(dir)
    if (!fs.existsSync(path.join(root, MARKER))) return {}

    if (EDITS[tool]) {
      const file = path.resolve((input || {}).file_path || '')
      const ours = settingsFiles(root, home).map((own) => path.resolve(own))

      return ours.includes(file)
        ? { ask: `heeded: ${file} sets what needs the owner's OK. Changing it needs it too.` }
        : {}
    }

    const command = (input || {}).command || ''
    const acts = classify(command)
    if (!acts.length) return {}

    // a git -C into another repo is that repo's business, if it is stamped
    const where = targetOf(command, cwd || root)
    const repo = rootOf(where)
    const gated = repo === root || fs.existsSync(path.join(repo, MARKER))
    const own = gated ? acts : acts.filter((act) => act === 'settings')
    if (!own.length) return {}

    const settings = settingsFor(repo, home)
    const refusal = own.includes('commit')
      ? commit(id, repo, where, command, settings)
      : null
    if (refusal) return { deny: refusal }

    const asks = own.filter((act) => act === 'settings' || settings.permission[act] !== 'allow')
    const what = {
      commit: 'a commit',
      push: 'something that leaves the machine',
      settings: 'changing heeded settings'
    }

    return asks.length
      ? { ask: `heeded: ${asks.map((act) => what[act]).join(' and ')} needs the owner's OK here.` }
      : {}
  }

  return { check }
}
