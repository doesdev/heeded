import fs from 'node:fs'
import path from 'node:path'
import { MARKER, NOTES } from '../config.js'

const parse = (text) => {
  try {
    return JSON.parse(text).notes
  } catch (ex) {
    return null
  }
}

// the marker opts a repo in and says where its notes live; null means no marker
export const notesOf = (root) => {
  const file = path.join(root, MARKER)
  let text

  try {
    text = fs.readFileSync(file, 'utf8')
  } catch (ex) {
    if (ex.code === 'ENOENT') return null
    throw ex
  }

  const notes = parse(text)
  if (typeof notes === 'string' && notes) return path.resolve(root, notes)

  const ex = new Error(`${file} must be JSON like { "notes": "${NOTES}" }`)
  ex.code = 'EMARKER'
  throw ex
}

export const within = (root, file) => {
  return path.relative(root, file).split(path.sep).join('/')
}
