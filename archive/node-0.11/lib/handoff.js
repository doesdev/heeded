import { execFileSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import fs from 'node:fs'
import path from 'node:path'
import { HANDOFF, JOURNAL } from '../config.js'
import { notesOf, within } from './notes.js'

export const SECTIONS = ['Did', 'Did not', 'Open', 'Next']

const sha = (data) => createHash('sha256').update(data).digest('hex')

const hashFile = (file) => {
  try {
    return sha(fs.readFileSync(file))
  } catch (ex) {
    if (ex.code === 'ENOENT' || ex.code === 'EISDIR') return null
    throw ex
  }
}

export const git = (cwd, args) => {
  try {
    return execFileSync('git', args, {
      cwd,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore']
    })
  } catch (ex) {
    return null
  }
}

// porcelain -z puts a rename's original path in its own NUL field
const changedPaths = (status) => {
  const paths = []
  const fields = status.split('\0').filter(Boolean)

  for (let i = 0; i < fields.length; i++) {
    const [xy, file] = [fields[i].slice(0, 2), fields[i].slice(3)]
    paths.push([xy, file])
    if (/[RC]/.test(xy)) i++
  }

  return paths
}

const hashTree = (root, ours) => {
  const status = git(root, ['status', '--porcelain', '-z', '-uall'])
  if (status === null) return null

  const head = git(root, ['rev-parse', '--verify', '-q', 'HEAD']) || ''
  const entries = changedPaths(status)
    .filter(([, file]) => !ours.includes(file) && !file.endsWith('/'))
    .map(([xy, file]) => [xy, file, hashFile(path.join(root, file))])

  return sha(JSON.stringify([head.trim(), entries]))
}

export const rootOf = (dir) => {
  const top = git(dir, ['rev-parse', '--show-toplevel'])

  return top ? path.normalize(top.trim()) : dir
}

const commonOf = (dir) => {
  const found = git(dir, ['rev-parse', '--git-common-dir'])

  return found && path.resolve(dir, found.trim())
}

// another worktree of the same repo, when that is where the event happened
const sideOf = (root, cwd) => {
  const side = cwd ? rootOf(cwd) : root
  if (side === root) return undefined

  const common = commonOf(root)
  if (!common || commonOf(side) !== common) return undefined

  return { root: side, tree: hashTree(side, []) }
}

export const snapshot = (dir, cwd) => {
  const root = rootOf(dir)
  const notes = notesOf(root)
  const file = notes && path.join(notes, HANDOFF)
  const journal = notes && path.join(notes, JOURNAL)
  const ours = notes ? [file, journal].map((own) => within(root, own)) : []
  const handoff = notes ? hashFile(file) : null
  const tree = hashTree(root, ours)

  return { root, notes, file, journal, handoff, tree, side: sideOf(root, cwd) }
}

export const checkHandoff = (text) => {
  const heads = [...text.matchAll(/^## (.+)$/gm)].map(([, head]) => head)
  const checks = [
    ['title', /^# Handoff\r?\n/.test(text)],
    ['last-session', /^Last session: \S/m.test(text)],
    ['sections', heads.join('|') === SECTIONS.join('|')]
  ]

  return checks.filter(([, ok]) => !ok).map(([name]) => name)
}
