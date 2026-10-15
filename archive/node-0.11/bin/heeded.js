#!/usr/bin/env node
import fs from 'node:fs'
import path from 'node:path'
import readline from 'node:readline/promises'
import { parseArgs } from 'node:util'
import { HANDOFF, HOME_DIR, JOURNAL, MARKER, NOTES, SETTINGS } from '../config.js'
import { git, rootOf } from '../lib/handoff.js'
import { createJournal, sinceHandoff } from '../lib/journal.js'
import { notesOf } from '../lib/notes.js'
import { CLASSES, OUTCOMES, REWORK, checkRework } from '../lib/rework.js'
import { scaffold } from '../lib/scaffold.js'
import { list } from '../lib/session.js'
import { DEFAULTS, directivesOf, readJson, settingsFor, userSettings } from '../lib/settings.js'
import { walk } from '../lib/wizard.js'

const USER_FILE = path.join(HOME_DIR, SETTINGS)

const HELP = {
  init: `heeded init [dir] [--notes <folder>] [--tracked]

  Stamps a repo so heeded's hooks gate it. Writes ${MARKER} at the root,
  the notes folder it names (default ${NOTES}) with handoff.md, plan.md,
  decisions/ and rework.jsonl, and a package.json with a failing check
  script if the repo has none. Never overwrites.

  What it creates is hidden from this clone's git (.git/info/exclude)
  unless --tracked. --notes puts the notes somewhere else, such as a
  private checkout inside the repo.`,

  moves: `heeded moves [dir]

  Prints the time, where the handoff is, and every move the hooks
  logged since the handoff was last written.`,

  retro: `heeded retro [dir] --task <text> --outcome <outcome> --class <class>
             [--note <text>]

  Logs how a finished task turned out, one line in rework.jsonl.
    outcome  ${OUTCOMES.join(', ')}
    class    ${CLASSES.join(', ')}
  The class is none when, and only when, the outcome is kept.`,

  config: `heeded config [dir] [--global | --local | --show]

  Steps through every setting and asks for each, then writes the file.
  Enter keeps what is there.
    --global  your own config, ${USER_FILE}
    --local   this repo's ${MARKER}, over yours; inherit drops a value
              so yours applies
    --show    change nothing; write your config if it is missing, and
              print what is in effect here
  With none of them, it asks which file, or shows when nobody is at
  the keyboard.`,

  help: `heeded help [command]

  Prints this, or more about one command.`
}

const SUMMARY = `heeded: good habits, installed rather than remembered

Usage: heeded <command> [dir] [options]

  init     stamp a repo
  moves    what changed since the last handoff
  retro    log how a finished task turned out
  config   step through your settings, global or for this repo
  help     this, or more about one command (heeded help config)

[dir] defaults to the current folder; any folder in a repo means
that repo.`

const help = (root, opts, topic) => {
  if (topic && !HELP[topic]) throw new Error(`No command "${topic}".\n\n${SUMMARY}`)

  console.log(topic ? HELP[topic] : SUMMARY)
}

const init = (root, opts) => {
  const { files, check } = scaffold(root, opts)

  console.log(root)
  files.forEach(([file, did]) => console.log(`  ${did.padEnd(8)}${file}`))
  if (!check) console.log('package.json has no "check" script. Add one.')
}

const stamped = (root) => {
  const notes = notesOf(root)
  if (!notes) throw new Error(`${root} has no ${MARKER}. Stamp it with: heeded init`)

  return notes
}

const moves = (root) => {
  const notes = stamped(root)
  const left = sinceHandoff(createJournal(path.join(notes, JOURNAL)).read())

  console.log(`Now: ${new Date().toISOString().slice(0, 16)}Z`)
  console.log(`Handoff: ${path.join(notes, HANDOFF)}`)
  if (!left.length) return console.log('No moves since it was last written.')

  console.log(`Moves since it was last written:\n${list(left)}`)
}

const retro = (root, { task, outcome, class: failureClass, note }) => {
  const notes = stamped(root)
  const entry = { ts: new Date().toISOString(), task, outcome, failureClass, note: note || '' }
  const problems = checkRework(entry)

  if (problems.length) {
    throw new Error(`Not logged, bad ${problems.join(', ')}.\n\n${HELP.retro}`)
  }

  createJournal(path.join(notes, REWORK)).append(entry)
  console.log(`Logged to ${path.join(notes, REWORK)}`)
}

// config

const seeded = (root) => {
  const global = (key) => (git(root, ['config', '--global', key]) || '').trim()
  const identity = { ...DEFAULTS.identity, name: global('user.name'), email: global('user.email') }

  return { ...DEFAULTS, identity }
}

const writeJson = (file, data) => {
  fs.mkdirSync(path.dirname(file), { recursive: true })
  fs.writeFileSync(file, `${JSON.stringify(data, null, 2)}\n`)
}

const inEffect = (root) => {
  const settings = settingsFor(root, HOME_DIR)
  const directives = directivesOf(settings).map(([name, said]) => {
    return `${name} (${said.length} characters)`
  })

  console.log(`In effect in ${root}:`)
  console.log(JSON.stringify({ ...settings, directives }, null, 2))
}

const show = (root) => {
  const missing = !fs.existsSync(USER_FILE)
  if (missing) writeJson(USER_FILE, seeded(root))

  console.log(`${missing ? 'created' : 'kept'} ${USER_FILE}`)
  inEffect(root)
}

// a person gets a prompt; piped answers are read up front, one per line
const createAsker = async () => {
  if (process.stdin.isTTY) {
    const rl = readline.createInterface({ input: process.stdin, output: process.stdout })
    const ask = async (question) => {
      try {
        return await rl.question(question)
      } catch (ex) {
        return ''
      }
    }

    return { ask, close: () => rl.close() }
  }

  const chunks = []
  for await (const chunk of process.stdin) chunks.push(chunk)
  const lines = Buffer.concat(chunks).toString('utf8').split(/\r?\n/)
  const ask = async (question) => {
    const answer = lines.length ? lines.shift() : ''
    console.log(`${question}${answer}`)

    return answer
  }

  return { ask, close: () => {} }
}

const SCOPES = {
  global: (root) => ({
    file: USER_FILE,
    start: fs.existsSync(USER_FILE) ? readJson(USER_FILE) : seeded(root),
    inherited: DEFAULTS,
    local: false
  }),
  local: (root) => {
    stamped(root)
    const file = path.join(root, MARKER)

    return { file, start: readJson(file), inherited: userSettings(HOME_DIR), local: true }
  }
}

const config = async (root, opts) => {
  if (opts.show || (!opts.global && !opts.local && !process.stdin.isTTY)) return show(root)

  const { ask, close } = await createAsker()
  const say = (text) => console.log(text)

  try {
    let scope = opts.local ? 'local' : 'global'
    if (!opts.local && !opts.global && notesOf(root)) {
      const answer = (await ask(`Edit [g]lobal (${USER_FILE}) or [l]ocal (${MARKER} here)? `)).trim()
      scope = answer.toLowerCase().startsWith('l') ? 'local' : 'global'
    }

    const { file, start, inherited, local } = SCOPES[scope](root)
    say(`Editing ${file}. Enter keeps what is there.`)
    const next = await walk(start, { ask, say, local, inherited })

    writeJson(file, next)
    say(`wrote ${file}`)
    inEffect(root)
  } finally {
    close()
  }
}

const COMMANDS = { init, moves, retro, config, help }

const run = async () => {
  const { values, positionals } = parseArgs({
    allowPositionals: true,
    options: {
      notes: { type: 'string' },
      tracked: { type: 'boolean' },
      task: { type: 'string' },
      outcome: { type: 'string' },
      class: { type: 'string' },
      note: { type: 'string' },
      global: { type: 'boolean' },
      local: { type: 'boolean' },
      show: { type: 'boolean' },
      help: { type: 'boolean', short: 'h' }
    }
  })
  const [name, arg] = positionals

  if (!name || values.help) return help(null, values, name)
  if (!COMMANDS[name]) throw new Error(`No command "${name}".\n\n${SUMMARY}`)
  if (name === 'help') return help(null, values, arg)
  if (values.global && values.local) throw new Error('Pick one of --global and --local.')

  await COMMANDS[name](rootOf(path.resolve(arg || '.')), values)
}

try {
  await run()
} catch (ex) {
  console.error(ex.message)
  process.exitCode = 1
}
