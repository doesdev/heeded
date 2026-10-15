import fs from 'node:fs'
import { OUTSIDE } from './session.js'

const parse = (line) => {
  try {
    return JSON.parse(line)
  } catch (ex) {
    return null
  }
}

export const createJournal = (file) => {
  const lines = () => {
    try {
      return fs.readFileSync(file, 'utf8').split('\n').filter(Boolean)
    } catch (ex) {
      if (ex.code === 'ENOENT') return []
      throw ex
    }
  }

  const read = () => lines().map(parse).filter(Boolean)

  const append = (entry) => {
    fs.appendFileSync(file, `${JSON.stringify(entry)}\n`)
  }

  const trim = (max) => {
    const all = lines()
    if (all.length <= max) return

    fs.writeFileSync(file, `${all.slice(-max).join('\n')}\n`)
  }

  return { read, append, trim }
}

export const sinceHandoff = (entries) => {
  return entries.slice(entries.findLastIndex((entry) => entry.handoff) + 1)
}

// moves other sessions made after the handoff was last written
export const unhanded = (entries, session) => {
  return sinceHandoff(entries)
    .filter((entry) => entry.tool !== OUTSIDE && entry.session !== session)
}
