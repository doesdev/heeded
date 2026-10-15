import fs from 'node:fs'
import path from 'node:path'
import { DIRECTIVE_MAX, MARKER, SETTINGS } from '../config.js'

export const DEFAULTS = {
  permission: { commit: 'ask', push: 'ask' },
  identity: { name: '', email: '', attribution: 'any' },
  checks: { changelog: false, quiet: [] },
  directives: {}
}

export const readJson = (file) => {
  try {
    return JSON.parse(fs.readFileSync(file, 'utf8')) || {}
  } catch (ex) {
    return {}
  }
}

const inside = (base, file) => {
  const rel = path.relative(base, file)

  return !rel.startsWith('..') && !path.isAbsolute(rel)
}

// a directive is text, or a file to read it from; anything else switches
// it off. A repo's marker may only point at files inside that repo.
const place = (value, base, confined) => {
  if (typeof value === 'string') return value.trim() ? { text: value.trim() } : null
  if (!value || typeof value.file !== 'string') return null

  const file = path.resolve(base, value.file)

  return confined && !inside(base, file) ? null : { file }
}

const readLayer = (file, confined) => {
  const raw = readJson(file)
  const placed = Object.entries(raw.directives || {})
    .map(([name, value]) => [name, place(value, path.dirname(file), confined)])

  return { ...raw, directives: Object.fromEntries(placed) }
}

const layer = (base, over) => {
  return Object.fromEntries(Object.keys(base).map((key) => {
    return [key, { ...base[key], ...(over[key] || {}) }]
  }))
}

export const settingsFiles = (root, home) => {
  return [path.join(home, SETTINGS), path.join(root, MARKER)]
}

// the user's defaults, then the repo's marker on top
export const settingsFor = (root, home) => {
  const [user, repo] = settingsFiles(root, home)

  return [readLayer(user, false), readLayer(repo, true)].reduce(layer, DEFAULTS)
}

// what a repo inherits: the defaults with the user's config on top
export const userSettings = (home) => {
  return [readLayer(path.join(home, SETTINGS), false)].reduce(layer, DEFAULTS)
}

const textOf = ({ text, file }) => {
  if (text) return text

  try {
    return fs.readFileSync(file, 'utf8').trim()
  } catch (ex) {
    return `(heeded could not read ${file})`
  }
}

const clip = (text) => {
  if (text.length <= DIRECTIVE_MAX) return text

  return `${text.slice(0, DIRECTIVE_MAX)}\n(cut at ${DIRECTIVE_MAX} characters)`
}

// [name, text] for each directive still switched on
export const directivesOf = (settings) => {
  return Object.entries(settings.directives)
    .filter(([, directive]) => directive)
    .map(([name, directive]) => [name, clip(textOf(directive))])
    .filter(([, text]) => text)
}
