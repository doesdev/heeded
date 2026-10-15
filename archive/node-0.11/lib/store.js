import fs from 'node:fs'
import path from 'node:path'

export const createStore = (dir) => {
  const fileFor = (id) => path.join(dir, `${String(id).replace(/\W/g, '_')}.json`)

  const load = (id) => {
    try {
      return JSON.parse(fs.readFileSync(fileFor(id), 'utf8'))
    } catch (ex) {
      if (ex.code === 'ENOENT') return null
      throw ex
    }
  }

  const save = (id, state) => {
    fs.mkdirSync(dir, { recursive: true })
    fs.writeFileSync(fileFor(id), JSON.stringify(state))
  }

  const has = (id) => fs.existsSync(fileFor(id))

  return { load, save, has }
}
