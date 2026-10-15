import fs from 'node:fs'
import path from 'node:path'
import { HOME_DIR, MARKER, STATE_DIR } from '../config.js'
import { createStore } from '../lib/store.js'
import * as claude from './claude.js'
import * as cursor from './cursor.js'

// is there a marker in this folder or any above it
const marked = (dir) => {
  for (let at = path.resolve(dir); ; at = path.dirname(at)) {
    if (fs.existsSync(path.join(at, MARKER))) return true
    if (path.dirname(at) === at) return false
  }
}

// one entry for every harness: the adapter turns its input into a common
// event and the gate's answer back into what that harness expects
const input = JSON.parse(fs.readFileSync(0, 'utf8'))
const harness = input.cursor_version ? cursor : claude
const event = harness.parse(input, process.env)
const store = createStore(STATE_DIR)

const answer = async () => {
  if (!event.name) return {}

  // the way out for a repo that never opted in, before git is touched or
  // the rest is loaded; a session already under way still goes through,
  // so deleting the marker does not get one past the gate
  if (!marked(event.dir) && !store.has(event.id)) return {}

  if (event.name === 'pre') {
    const { createGuard } = await import('../lib/guard.js')
    return createGuard(HOME_DIR, store).check(event)
  }

  const { createGate } = await import('../lib/gate.js')
  return createGate(STATE_DIR, HOME_DIR).handle(event)
}

const { stdout, stderr, code } = harness.emit(event, await answer())

if (stdout) console.log(stdout)
if (stderr) console.error(stderr)
process.exitCode = code || 0
