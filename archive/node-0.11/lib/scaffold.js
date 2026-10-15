import fs from 'node:fs'
import path from 'node:path'
import { MARKER, NOTES, TEMPLATES } from '../config.js'
import { git } from './handoff.js'
import { notesOf, within } from './notes.js'

const MANIFEST = 'package.json'
const EXCLUDE = 'info/exclude'

const templates = (kind) => {
  const dir = path.join(TEMPLATES, kind)

  return fs.readdirSync(dir, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile())
    .map((entry) => within(dir, path.join(entry.parentPath, entry.name)))
    .sort()
    .map((file) => [path.join(dir, file), file])
}

// never overwrites: a second run, or a repo with its own files, loses nothing
const create = (dest, put) => {
  fs.mkdirSync(path.dirname(dest), { recursive: true })

  try {
    put(dest)
  } catch (ex) {
    if (ex.code === 'EEXIST') return 'kept'
    throw ex
  }

  return 'created'
}

const stamp = (kind, dir) => {
  return templates(kind).map(([src, file]) => {
    const dest = path.join(dir, file)
    const copy = () => fs.copyFileSync(src, dest, fs.constants.COPYFILE_EXCL)

    return [dest, create(dest, copy)]
  })
}

const mark = (root, notes) => {
  const dest = path.join(root, MARKER)
  const text = `${JSON.stringify({ notes }, null, 2)}\n`

  return [dest, create(dest, () => fs.writeFileSync(dest, text, { flag: 'wx' }))]
}

const hasCheck = (root) => {
  try {
    const text = fs.readFileSync(path.join(root, MANIFEST), 'utf8')

    return Boolean(JSON.parse(text).scripts.check)
  } catch (ex) {
    return false
  }
}

// private by default: this clone's git stops seeing them, .gitignore is not touched
const hide = (root, patterns) => {
  const found = git(root, ['rev-parse', '--git-path', EXCLUDE])
  const wanted = patterns
    .filter((pattern) => !pattern.startsWith('/..') && !/^\/\w:/.test(pattern))
    .filter((pattern) => git(root, ['check-ignore', '-q', pattern.slice(1)]) === null)

  if (found === null || !wanted.length) return []

  const file = path.resolve(root, found.trim())
  fs.mkdirSync(path.dirname(file), { recursive: true })
  fs.appendFileSync(file, `${wanted.join('\n')}\n`)

  return [[file, 'updated']]
}

// an existing marker wins over the notes folder asked for, and only what
// this run created is hidden: what was already there is the repo's choice
export const scaffold = (root, opts) => {
  const { notes: asked, tracked } = opts || {}
  const marker = mark(root, asked || NOTES)
  const notes = notesOf(root)
  const fresh = [
    marker[1] === 'created' && `/${MARKER}`,
    !fs.existsSync(notes) && `/${within(root, notes)}/`,
    !fs.existsSync(path.join(root, MANIFEST)) && `/${MANIFEST}`
  ].filter(Boolean)
  const stamped = [marker, ...stamp('notes', notes), ...stamp('root', root)]
  const files = [...stamped, ...(tracked ? [] : hide(root, fresh))]
    .map(([file, did]) => [within(root, file), did])

  return { notes, files, check: hasCheck(root) }
}
