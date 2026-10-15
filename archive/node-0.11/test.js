import { execFileSync, spawnSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import test from 'mvt'
import * as config from './config.js'
import * as claude from './hooks/claude.js'
import * as cursor from './hooks/cursor.js'
import {
  attributionProblem,
  changelogOwed,
  classify,
  identityProblem,
  scanDiff,
  secretFile,
  secretIn,
  stages,
  targetOf
} from './lib/commit.js'
import { checkRecord } from './lib/decisions.js'
import { SECTIONS, checkHandoff, snapshot } from './lib/handoff.js'
import { createJournal, sinceHandoff, unhanded } from './lib/journal.js'
import { notesOf } from './lib/notes.js'
import { CLASSES, OUTCOMES, checkRework } from './lib/rework.js'
import { scaffold } from './lib/scaffold.js'
import { DEFAULTS, directivesOf, settingsFor } from './lib/settings.js'
import { STEPS, get, pick, set, walk } from './lib/wizard.js'
import { createGate } from './lib/gate.js'
import { OUTSIDE, decide, live, open, toMove, track } from './lib/session.js'
import { createStore } from './lib/store.js'

const HERE = path.dirname(fileURLToPath(import.meta.url))
const HOOK = path.join(HERE, 'hooks', 'hook.js')
const BIN = path.join(HERE, 'bin', 'heeded.js')
// HEEDED_BIN runs the same tests against a built binary instead of the scripts
const EXE = process.env.HEEDED_BIN
const cli = (args) => EXE ? [EXE, args] : ['node', [BIN, ...args]]
const hookCommand = () => EXE ? [EXE, ['hook']] : ['node', [HOOK]]
const HELP_TOPICS = { init: 1, moves: 1, retro: 1, config: 1, help: 1 }
const { MARKER, MAX_BLOCKS, NOTES, TEMPLATES } = config
const HANDOFF = `${NOTES}/${config.HANDOFF}`
const JOURNAL = `${NOTES}/${config.JOURNAL}`
const DASHES = new RegExp(String.fromCharCode(91, 0x2013, 0x2014, 93))
const MARKED = `{ "notes": "${NOTES}" }\n`
const stampedIn = (notes) => [
  MARKER,
  `${notes}/.gitignore`,
  `${notes}/decisions/README.md`,
  `${notes}/handoff.md`,
  `${notes}/plan.md`,
  `${notes}/rework.jsonl`,
  'package.json'
]
const tmp = fs.realpathSync.native(
  fs.mkdtempSync(path.join(os.tmpdir(), 'heeded-test-'))
)
const stateDir = path.join(tmp, 'state')
let seq = 0

const git = (cwd, ...args) => execFileSync('git', args, { cwd, stdio: 'ignore' })

const write = (repo, file, text) => {
  fs.mkdirSync(path.dirname(path.join(repo, file)), { recursive: true })
  fs.writeFileSync(path.join(repo, file), text)
}

const createRepo = (stamped) => {
  const repo = path.join(tmp, `repo-${++seq}`)
  fs.mkdirSync(repo)
  git(repo, 'init', '-q')
  git(repo, 'config', 'user.email', 'test@example.com')
  git(repo, 'config', 'user.name', 'test')
  write(repo, 'README.md', 'hi\n')
  if (stamped) write(repo, MARKER, MARKED)
  if (stamped) write(repo, HANDOFF, '# Handoff\n')
  git(repo, 'add', '-A')
  git(repo, 'commit', '-q', '-m', 'Initial commit')

  return repo
}

const read = (...parts) => fs.readFileSync(path.join(...parts), 'utf8')

const runBin = (cwd, ...args) => {
  return spawnSync(...cli(args), { cwd, encoding: 'utf8' })
}

const runBinAt = (home, cwd, ...args) => {
  const env = { ...process.env, HEEDED_HOME: home }

  return spawnSync(...cli(args), { cwd, encoding: 'utf8', env })
}

const runHook = (repo, id, event, extra) => {
  const env = {
    ...process.env,
    CLAUDE_PLUGIN_DATA: stateDir,
    CLAUDE_PROJECT_DIR: repo,
    HEEDED_HOME: path.join(tmp, 'home')
  }
  const input = { session_id: id, cwd: repo, hook_event_name: event, ...extra }

  return spawnSync(...hookCommand(), {
    input: JSON.stringify(input),
    encoding: 'utf8',
    env
  })
}

// plays Claude Code: every tool call is followed by the PostToolUse hook
const createSession = (repo, id) => {
  const edit = (file, text) => {
    write(repo, file, text)
    const toolInput = { file_path: path.join(repo, file) }

    return runHook(repo, id, 'PostToolUse', {
      tool_name: 'Edit',
      tool_input: toolInput
    })
  }

  const bash = (description, run) => {
    run()
    const toolInput = { command: 'SECRET=hunter2 do-it', description }

    return runHook(repo, id, 'PostToolUse', {
      tool_name: 'Bash',
      tool_input: toolInput
    })
  }

  return {
    repo,
    id,
    edit,
    bash,
    hook: (event, extra) => runHook(repo, id, event, extra),
    start: () => runHook(repo, id, 'SessionStart'),
    prompt: () => runHook(repo, id, 'UserPromptSubmit'),
    stop: () => runHook(repo, id, 'Stop'),
    journal: () => createJournal(path.join(repo, JOURNAL)).read()
  }
}

const startSession = (stamped) => {
  const repo = createRepo(stamped)
  const session = createSession(repo, `session-${seq}`)
  session.start()

  return session
}

test.after(() => fs.rmSync(tmp, { recursive: true, force: true }))

// pure logic

test('track turns snapshot changes into journal entries', (assert) => {
  const edit = { tool: 'Edit', what: 'a.js' }
  const human = { tool: OUTSIDE, what: '' }
  const at = (handoff, tree) => ({ handoff, tree })
  const cases = [
    ['nothing changed', at('a', 'x'), at('a', 'x'), edit, null, 0],
    ['work is owed a handoff', at('a', 'x'), at('a', 'y'), edit, false, 1],
    ['a handoff write settles up', at('a', 'x'), at('b', 'x'), edit, true, 0],
    ['both at once settles up', at('a', 'x'), at('b', 'y'), edit, true, 0],
    ['losing the handoff is work', at('a', 'x'), at(null, 'x'), edit, false, 1],
    ['human work is not owed', at('a', 'x'), at('a', 'y'), human, false, 0],
    ['unstamped repos stay quiet', at(null, 'x'), at(null, 'y'), edit, null, 0],
    ['a new handoff stamps the repo', at(null, 'x'), at('a', 'x'), edit, true, 0]
  ]

  cases.forEach(([name, last, now, move, handoff, owed]) => {
    const { state, entry } = track(open(last), now, move)

    assert.is(entry ? entry.handoff : null, handoff, name)
    assert.is(state.pending.length, owed, name)
    assert.deepEqual(state.last, now, name)
  })
})

test('track clears what was owed once the handoff is written', (assert) => {
  const edit = { tool: 'Edit', what: 'a.js' }
  const steps = [
    [{ handoff: 'a', tree: 'y' }, 1],
    [{ handoff: 'a', tree: 'z' }, 2],
    [{ handoff: 'b', tree: 'z' }, 0],
    [{ handoff: 'b', tree: 'w' }, 1]
  ]
  let state = open({ handoff: 'a', tree: 'x' })

  steps.forEach(([now, owed]) => {
    state = track(state, now, edit).state
    assert.is(state.pending.length, owed)
  })
})

test('decide covers every state', (assert) => {
  const owed = [{ tool: 'Edit', what: 'a.js', handoff: false }]
  const cases = [
    [{ stamped: false, pending: [] }, { handoff: null }, false, 'unstamped'],
    [{ stamped: true, pending: [] }, { handoff: null }, true, 'handoff-missing'],
    [{ stamped: true, pending: owed }, { handoff: 'a' }, true, 'handoff-stale'],
    [{ stamped: true, pending: [] }, { handoff: 'a' }, false, 'current'],
    [{ stamped: true, pending: [] }, { notes: null }, true, 'marker-missing'],
    [{ stamped: false, pending: [] }, { notes: null }, false, 'unstamped']
  ]

  cases.forEach(([state, now, block, why]) => {
    assert.deepEqual(decide(state, now), { block, why })
  })
})

test('toMove names files, and never keeps a command', (assert) => {
  const root = path.join(tmp, 'root')
  const file = path.join(root, 'lib', 'a.js')
  const cases = [
    ['Edit', { file_path: file }, 'lib/a.js'],
    ['Write', { file_path: file }, 'lib/a.js'],
    ['NotebookEdit', { notebook_path: file }, 'lib/a.js'],
    ['Bash', { command: 'TOKEN=abc curl', description: 'Fetch it' }, 'Fetch it'],
    ['Bash', { command: 'TOKEN=abc curl' }, ''],
    ['PowerShell', { command: 'x', description: 'Run it' }, 'Run it'],
    ['SomethingNew', { anything: 1 }, ''],
    ['Edit', undefined, '']
  ]

  cases.forEach(([tool, input, what]) => {
    assert.deepEqual(toMove(tool, input, root), { tool, what })
  })
})

test('unhanded returns other sessions\' moves since the last handoff', (assert) => {
  const move = (session, tool, handoff) => ({ session, tool, handoff })
  const entries = [
    move('a', 'Edit', false),
    move('a', 'Edit', true),
    move('a', 'Bash', false),
    move('a', OUTSIDE, false),
    move('b', 'Edit', false)
  ]

  assert.deepEqual(unhanded(entries, 'b'), [entries[2]])
  assert.deepEqual(unhanded(entries, 'c'), [entries[2], entries[4]])
  assert.deepEqual(unhanded(entries.slice(0, 2), 'c'), [])
  assert.deepEqual(unhanded([], 'c'), [])
  assert.deepEqual(sinceHandoff(entries), entries.slice(2))
  assert.deepEqual(sinceHandoff(entries.slice(0, 1)), [entries[0]])
})

test('checkHandoff holds the handoff to its fixed shape', (assert) => {
  const body = SECTIONS.map((name) => `## ${name}\n\n- x\n`)
  const doc = (head, parts) => [...head, '', ...parts].join('\n')
  const head = ['# Handoff', '', 'Last session: 2026-10-09T06:04Z']
  const cases = [
    [doc(head, body), []],
    [doc(head, body).replaceAll('\n', '\r\n'), []],
    [doc(['# Notes', '', head[2]], body), ['title']],
    [doc(['intro', ...head], body), ['title']],
    [doc(['# Handoff'], body), ['last-session']],
    [doc(head, body.slice(1)), ['sections']],
    [doc(head, body.toReversed()), ['sections']],
    [doc(head, [...body, '## Misc\n']), ['sections']],
    ['', ['title', 'last-session', 'sections']]
  ]

  cases.forEach(([text, problems]) => {
    assert.deepEqual(checkHandoff(text), problems)
  })
})

test('the handoff shape is the same everywhere it is written down', (assert) => {
  const skill = read(HERE, 'skills', 'handoff', 'SKILL.md')
  const shape = SECTIONS.map((name) => `## ${name}\n`).join('\n')

  assert.deepEqual(checkHandoff(read(TEMPLATES, 'notes', config.HANDOFF)), [])
  assert.deepEqual(checkHandoff(read(notesOf(HERE), config.HANDOFF)), [])
  assert.truthy(skill.replaceAll('\r\n', '\n').includes(shape))
})

test('the plugin and the package are one version, and ship both skills', (assert) => {
  const { version, bin } = JSON.parse(read(HERE, 'package.json'))
  const plugin = JSON.parse(read(HERE, '.claude-plugin', 'plugin.json'))

  assert.is(plugin.version, version)
  assert.truthy(fs.existsSync(path.join(HERE, bin.heeded)))

  ;['init', 'handoff', 'retro'].forEach((name) => {
    const skill = read(HERE, 'skills', name, 'SKILL.md')

    assert.truthy(skill.startsWith(`---\nname: ${name}\ndescription: `), name)
    // eslint-disable-next-line no-template-curly-in-string
    assert.truthy(skill.includes('"${CLAUDE_PLUGIN_ROOT}/bin/heeded.js"'), name)
  })
})

test('checkRecord enforces who proposed and who ratified', (assert) => {
  const record = (lines, body) => `# 0000: x\n\n${lines.join('\n')}\n\n${body}`
  const head = ['Date: 2026-10-09T05:39Z', 'Status: accepted']
  const model = 'Proposed by: model (claude-opus-5-5)'
  const human = 'Proposed by: human (Andrew)'
  const ratified = 'Ratified by: claude-opus-5-5'
  const cases = [
    [[...head, model, ratified], '## Decision', []],
    [[...head, human, ratified], '## Assessment', []],
    [[...head, human, ratified], '## Decision', ['assessment']],
    [[...head, human, 'Ratified by: human'], '## Assessment', ['ratified-by']],
    [[...head, human], '## Assessment', ['ratified-by']],
    [[...head, ratified], '', ['proposed-by']],
    [['Date: yesterday', 'Status: vibes', model, ratified], '', ['date', 'status']],
    [[head[0], 'Status: amended by 0008', model, ratified], '', []]
  ]

  cases.forEach(([lines, body, problems]) => {
    assert.deepEqual(checkRecord(record(lines, body)), problems)
  })
})

test('every decision record in this repo is ratified', (assert) => {
  const dir = path.join(notesOf(HERE), 'decisions')
  const files = fs.readdirSync(dir)

  assert.truthy(files.length)
  files.forEach((file) => {
    const text = fs.readFileSync(path.join(dir, file), 'utf8')

    assert.truthy(/^\d{4}-[a-z0-9-]+\.md$/.test(file), file)
    assert.deepEqual(checkRecord(text), [], file)
  })
})

// against real files and git

test('snapshot sees every kind of work, and ignores our own files', (assert) => {
  const repo = createRepo(true)
  const trees = [snapshot(repo).tree]
  const steps = [
    () => write(repo, 'new.txt', 'one\n'),
    () => write(repo, 'new.txt', 'two\n'),
    () => write(repo, 'README.md', 'changed\n'),
    () => git(repo, 'add', '-A'),
    () => git(repo, 'commit', '-q', '-m', 'Work'),
    () => git(repo, 'mv', 'new.txt', 'renamed.txt'),
    () => fs.rmSync(path.join(repo, 'README.md'))
  ]

  steps.forEach((step) => {
    step()
    trees.push(snapshot(repo).tree)
  })
  assert.is(new Set(trees).size, trees.length)

  const before = snapshot(repo)
  write(repo, HANDOFF, '# Handoff\n\nDid things.\n')
  write(repo, JOURNAL, '{}\n')
  const after = snapshot(repo)
  assert.is(after.tree, before.tree)
  assert.not(after.handoff, before.handoff)
  assert.is(after.root, repo)
})

test('snapshot outside a git repo has no tree', (assert) => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'heeded-nogit-'))
  write(dir, MARKER, MARKED)
  write(dir, HANDOFF, '# Handoff\n')
  const snap = snapshot(dir)
  fs.rmSync(dir, { recursive: true, force: true })

  assert.is(snap.tree, null)
  assert.truthy(snap.handoff)
})

test('store round-trips and returns null for unknown ids', (assert) => {
  const store = createStore(path.join(tmp, 'store'))

  assert.is(store.load('nope'), null)
  store.save('../a b', { blocks: 2 })
  assert.deepEqual(store.load('../a b'), { blocks: 2 })
  assert.deepEqual(fs.readdirSync(path.join(tmp, 'store')), ['___a_b.json'])
})

test('journal appends, skips junk lines, and trims to the newest', (assert) => {
  const file = path.join(tmp, 'journal.jsonl')
  const journal = createJournal(file)

  assert.deepEqual(journal.read(), [])
  journal.trim(2)
  assert.is(fs.existsSync(file), false)

  fs.writeFileSync(file, '{"n":1}\nnot json\n')
  journal.append({ n: 2 })
  journal.append({ n: 3 })
  assert.deepEqual(journal.read(), [{ n: 1 }, { n: 2 }, { n: 3 }])

  journal.trim(2)
  assert.deepEqual(journal.read(), [{ n: 2 }, { n: 3 }])
})

test('notesOf reads the marker, and refuses a broken one', (assert) => {
  const dir = path.join(tmp, `marker-${++seq}`)
  const elsewhere = path.join(tmp, 'elsewhere')
  fs.mkdirSync(dir)
  assert.is(notesOf(dir), null)

  const good = [
    [MARKED, path.join(dir, NOTES)],
    ['{ "notes": "private/checkout/notes" }', path.join(dir, 'private', 'checkout', 'notes')],
    ['{ "notes": "." }', dir],
    [JSON.stringify({ notes: elsewhere }), elsewhere]
  ]
  const bad = ['', 'not json', 'null', '{}', '{ "notes": "" }', '{ "notes": 3 }']

  good.forEach(([text, notes]) => {
    write(dir, MARKER, text)
    assert.is(notesOf(dir), notes)
  })
  bad.forEach((text) => {
    write(dir, MARKER, text)
    try {
      assert.fail(`${notesOf(dir)} from a broken marker: ${text}`)
    } catch (ex) {
      assert.is(ex.code, 'EMARKER', text)
    }
  })
})

test('scaffold stamps an empty directory, and a second run changes nothing', (assert) => {
  const dir = path.join(tmp, `plain-${++seq}`)
  fs.mkdirSync(dir)

  const first = scaffold(dir)
  assert.deepEqual(first.files, stampedIn(NOTES).map((file) => [file, 'created']))
  assert.is(first.check, true)
  assert.is(first.notes, path.join(dir, NOTES))
  assert.deepEqual(JSON.parse(read(dir, MARKER)), { notes: NOTES })
  assert.is(read(dir, NOTES, '.gitignore'), `${config.JOURNAL}\n`)
  assert.is(read(dir, HANDOFF), read(TEMPLATES, 'notes', config.HANDOFF))
  assert.deepEqual(fs.readdirSync(dir).sort(), [NOTES, MARKER, 'package.json'].sort())

  write(dir, `${NOTES}/plan.md`, '# Plan\n\nMine now.\n')
  const second = scaffold(dir, { notes: 'somewhere/else' })
  assert.truthy(second.files.every(([, did]) => did === 'kept'))
  assert.is(second.notes, first.notes)
  assert.is(read(dir, NOTES, 'plan.md'), '# Plan\n\nMine now.\n')
  assert.is(fs.existsSync(path.join(dir, 'somewhere')), false)
})

test('scaffold puts the notes where it is told, and keeps what is there', (assert) => {
  const dir = path.join(tmp, `owned-${++seq}`)
  const manifest = '{ "name": "theirs", "scripts": { "test": "x" } }\n'
  const theirs = ['.gitignore', 'CLAUDE.md', 'docs/manual.md', 'package.json']
  write(dir, '.gitignore', '# curated\nnode_modules\n')
  write(dir, 'CLAUDE.md', 'theirs\n')
  write(dir, 'docs/manual.md', 'a product\n')
  write(dir, 'package.json', manifest)
  write(dir, 'private/notes/plan.md', 'theirs\n')
  const before = theirs.map((file) => read(dir, file))

  const { files, check, notes } = scaffold(dir, { notes: 'private/notes' })
  const done = Object.fromEntries(files)

  assert.is(notes, path.join(dir, 'private', 'notes'))
  assert.deepEqual(Object.keys(done), stampedIn('private/notes'))
  assert.is(done['private/notes/handoff.md'], 'created')
  assert.is(done['private/notes/plan.md'], 'kept')
  assert.is(done['package.json'], 'kept')
  assert.is(check, false)
  assert.is(read(dir, 'private/notes/plan.md'), 'theirs\n')
  assert.deepEqual(theirs.map((file) => read(dir, file)), before)
  assert.deepEqual(fs.readdirSync(path.join(dir, 'docs')), ['manual.md'])
})

test('the stamped check fails until someone wires it up', (assert) => {
  const { scripts } = JSON.parse(read(TEMPLATES, 'root', 'package.json'))
  const ran = spawnSync(scripts.check, { shell: true, encoding: 'utf8' })

  assert.is(ran.status, 1)
  assert.truthy(ran.stderr.includes('not wired up'))
})

// the CLI

test('heeded init stamps the repo root from anywhere inside it', (assert) => {
  const repo = createRepo(false)
  write(repo, 'deep/down/file.txt', 'x\n')

  const ran = runBin(path.join(repo, 'deep', 'down'), 'init')
  assert.is(ran.status, 0)
  assert.truthy(ran.stdout.startsWith(`${repo}\n`))
  assert.truthy(ran.stdout.includes(`created ${HANDOFF}`))
  assert.is(ran.stdout.includes('no "check" script'), false)
  stampedIn(NOTES).forEach((file) => {
    assert.truthy(fs.existsSync(path.join(repo, file)), file)
  })
  assert.is(fs.existsSync(path.join(repo, 'deep', 'down', NOTES)), false)

  const again = runBin(tmp, 'init', repo)
  assert.is(again.status, 0)
  assert.is(again.stdout.includes('created'), false)
})

test('heeded init --notes puts the notes there, and the journal stays ignored', (assert) => {
  const repo = createRepo(false)
  const ran = runBin(repo, 'init', '--notes', 'agent/notes')

  assert.is(ran.status, 0)
  assert.truthy(ran.stdout.includes('created agent/notes/handoff.md'))
  assert.deepEqual(JSON.parse(read(repo, MARKER)), { notes: 'agent/notes' })
  assert.is(fs.existsSync(path.join(repo, NOTES)), false)

  const session = createSession(repo, `notes-${seq}`)
  session.start()
  session.edit('work.txt', 'work\n')
  const status = execFileSync('git', ['status', '--porcelain', '-uall'], {
    cwd: repo,
    encoding: 'utf8'
  })

  assert.truthy(fs.existsSync(path.join(repo, 'agent', 'notes', config.JOURNAL)))
  assert.is(status.includes(config.JOURNAL), false)
  assert.truthy(session.stop().stderr.includes(path.join('agent', 'notes')))
})

test('heeded init keeps what it creates out of git unless told --tracked', (assert) => {
  const status = (repo) => execFileSync('git', ['status', '--porcelain', '-uall'], {
    cwd: repo,
    encoding: 'utf8'
  }).split('\n').filter(Boolean).map((line) => line.slice(3)).sort()
  const cases = [
    [[], [], true],
    [['--notes', 'agent/notes'], [], true],
    [['--tracked'], stampedIn(NOTES).filter((file) => file !== JOURNAL).sort(), false]
  ]

  cases.forEach(([args, seen, hidden]) => {
    const repo = createRepo(false)
    const ran = runBin(repo, 'init', ...args)

    assert.is(ran.status, 0)
    assert.deepEqual(status(repo), seen)
    assert.is(ran.stdout.includes('updated .git/info/exclude'), hidden)
    assert.is(fs.existsSync(path.join(repo, '.gitignore')), false)
    assert.is(runBin(repo, 'init', ...args).stdout.includes('updated'), false)
  })
})

test('heeded init hides nothing the repo already had or already ignores', (assert) => {
  const repo = createRepo(false)
  write(repo, '.gitignore', 'private\n')
  write(repo, MARKER, '{ "notes": "private/notes" }\n')
  write(repo, 'package.json', '{ "scripts": { "check": "x" } }\n')
  const kept = runBin(repo, 'init')

  assert.is(kept.stdout.includes('exclude'), false)

  const tracked = createRepo(true)
  write(tracked, 'package.json', '{ "scripts": { "check": "x" } }\n')
  const again = runBin(tracked, 'init')

  assert.is(again.status, 0)
  assert.is(again.stdout.includes('exclude'), false)
})

test('nothing written here has an em dash or an en dash in it', (assert) => {
  const listed = execFileSync(
    'git',
    ['ls-files', '-z', '--cached', '--others', '--exclude-standard'],
    { cwd: HERE, encoding: 'utf8' }
  )
  const files = listed.split('\0').filter(Boolean)
    .filter((file) => !/^archive\/|^package-lock\.json$/.test(file))
    .filter((file) => fs.existsSync(path.join(HERE, file)))

  assert.truthy(files.includes('README.md'))
  files.forEach((file) => {
    assert.is(DASHES.test(read(HERE, file)), false, file)
  })
})

test('heeded ships no rules of its own', (assert) => {
  const told = createSession(createRepo(true), `rules-${seq}`).start().stdout

  assert.is(fs.existsSync(path.join(HERE, 'corpus')), false)
  assert.is(told.includes('directives'), false)
  assert.is(told.trim().split('\n').length, 1)
  assert.is(createSession(createRepo(false), `none-${seq}`).start().stdout, '')
})

test('heeded init says so when the repo has no check', (assert) => {
  const repo = createRepo(false)
  write(repo, 'package.json', '{ "name": "theirs" }\n')

  assert.truthy(runBin(repo, 'init').stdout.includes('no "check" script'))
})

test('heeded fails with a reason, not a stack', (assert) => {
  const repo = createRepo(false)
  const broken = createRepo(false)
  write(broken, MARKER, 'not json')
  const cases = [
    [tmp, ['nope'], 'No command "nope".'],
    [tmp, ['help', 'nope'], 'No command "nope".'],
    [tmp, ['config', '--global', '--local'], 'Pick one of'],
    [repo, ['moves'], `${repo} has no ${MARKER}`],
    [repo, ['init', '--nope'], 'Unknown option'],
    [broken, ['moves'], `${path.join(broken, MARKER)} must be JSON`],
    [broken, ['init'], `${path.join(broken, MARKER)} must be JSON`]
  ]

  cases.forEach(([cwd, args, said]) => {
    const ran = runBin(cwd, ...args)

    assert.is(ran.status, 1, said)
    assert.truthy(ran.stderr.startsWith(said), said)
    assert.is(ran.stderr.includes('    at '), false, said)
  })
})

test('a repo stamped by heeded init is gated from then on', (assert) => {
  const session = startSession(false)
  session.edit('before.txt', 'before\n')
  assert.is(session.stop().status, 0)

  session.bash('Stamp the repo', () => runBin(session.repo, 'init'))
  assert.is(session.stop().status, 0)

  session.prompt()
  session.edit('after.txt', 'after\n')
  assert.is(session.stop().status, 2)
})

test('heeded moves lists what is owed, with the time and the file', (assert) => {
  const session = startSession(true)
  const file = path.join(session.repo, HANDOFF)
  const quiet = runBin(session.repo, 'moves').stdout.split('\n')
  assert.truthy(/^Now: \d{4}-\d\d-\d\dT\d\d:\d\dZ$/.test(quiet[0]))
  assert.is(quiet[1], `Handoff: ${file}`)
  assert.is(quiet[2], 'No moves since it was last written.')

  session.edit('one.txt', 'one\n')
  session.edit(HANDOFF, '# Handoff\n\nAdded one.\n')
  session.edit('two.txt', 'two\n')
  session.bash('Commit it', () => {
    git(session.repo, 'add', '-A')
    git(session.repo, 'commit', '-q', '-m', 'Work')
  })

  const owed = runBin(session.repo, 'moves')
  assert.is(owed.status, 0)
  assert.truthy(owed.stdout.endsWith('written:\n- Edit two.txt\n- Bash Commit it\n'))
  assert.is(owed.stdout.includes('one.txt'), false)
})

// the hook as Claude Code runs it

test('a session is told where the notes are when it starts', (assert) => {
  const repo = createRepo(true)
  const started = createSession(repo, `told-${seq}`).start()

  assert.is(started.status, 0)
  assert.truthy(started.stdout.includes(path.join(repo, NOTES)))
  assert.truthy(started.stdout.includes('handoff.md'))
  assert.is(started.stdout.includes('earlier sessions'), false)
  assert.is(createSession(createRepo(false), `quiet-${seq}`).start().stdout, '')
})

test('a docs folder with a handoff in it opts nothing in', (assert) => {
  const session = startSession(false)
  session.edit('docs/handoff.md', '# Handoff\n')
  session.edit('docs/manual.md', 'a product\n')

  assert.is(session.stop().status, 0)
  assert.is(fs.existsSync(path.join(session.repo, 'docs', config.JOURNAL)), false)
})

test('a marker with no handoff behind it leaves the repo alone', (assert) => {
  const repo = createRepo(false)
  write(repo, MARKER, '{ "notes": "private" }\n')
  const session = createSession(repo, `absent-${seq}`)

  assert.is(session.start().stdout, '')
  session.edit('work.txt', 'work\n')
  assert.is(session.stop().status, 0)
  assert.is(fs.existsSync(path.join(repo, 'private')), false)
})

test('notes in a private checkout the repo ignores still gate it', (assert) => {
  const repo = createRepo(false)
  const inner = path.join(repo, 'private')
  const handoff = 'private/notes/handoff.md'
  write(repo, '.gitignore', 'private\n')
  write(repo, MARKER, '{ "notes": "private/notes" }\n')
  fs.mkdirSync(inner)
  git(inner, 'init', '-q')
  write(repo, handoff, '# Handoff\n')
  git(repo, 'add', '-A')
  git(repo, 'commit', '-q', '-m', 'Opt in')

  const session = createSession(repo, `private-${seq}`)
  session.start()
  session.edit('private/notes/plan.md', '# Plan\n')
  assert.is(session.stop().status, 0)

  session.edit('work.txt', 'work\n')
  const refused = session.stop()
  assert.is(refused.status, 2)
  assert.truthy(refused.stderr.includes(path.join(inner, 'notes', 'handoff.md')))

  session.edit(handoff, '# Handoff\n\nAdded work.\n')
  assert.is(session.stop().status, 0)
  assert.deepEqual(
    createJournal(path.join(inner, 'notes', config.JOURNAL)).read()
      .map(({ what, handoff }) => [what, handoff]),
    [['work.txt', false], [handoff, true]]
  )
})

test('stop refuses when the marker was deleted', (assert) => {
  const session = startSession(true)
  session.bash('Opt out quietly', () => fs.rmSync(path.join(session.repo, MARKER)))

  const refused = session.stop()
  assert.is(refused.status, 2)
  assert.truthy(refused.stderr.startsWith(`${MARKER} is gone`))

  for (let i = 1; i < MAX_BLOCKS; i++) assert.is(session.stop().status, 2)
  assert.is(session.stop().status, 0)
})

test('stop passes when the session did no work', (assert) => {
  const session = startSession(true)

  assert.is(session.stop().status, 0)
  assert.deepEqual(session.journal(), [])
})

test('an unstamped repo is left alone, journal and all', (assert) => {
  const session = startSession(false)
  session.edit('work.txt', 'work\n')

  assert.is(session.stop().status, 0)
  assert.is(fs.existsSync(path.join(session.repo, JOURNAL)), false)
})

test('stop refuses after work until the handoff is written', (assert) => {
  const session = startSession(true)
  session.edit('lib/work.js', 'work\n')

  const refused = session.stop()
  assert.is(refused.status, 2)
  assert.truthy(refused.stderr.includes(path.join(session.repo, HANDOFF)))
  assert.truthy(refused.stderr.includes('- Edit lib/work.js'))
  assert.truthy(refused.stderr.includes('heeded:handoff skill'))

  session.edit(HANDOFF, '# Handoff\n\nAdded work.\n')
  assert.is(session.stop().status, 0)
})

test('work after the handoff makes it stale again', (assert) => {
  const session = startSession(true)
  session.edit('one.txt', 'one\n')
  session.edit(HANDOFF, '# Handoff\n\nAdded one.\n')
  session.edit('two.txt', 'two\n')

  const refused = session.stop()
  assert.is(refused.status, 2)
  assert.truthy(refused.stderr.includes('- Edit two.txt'))
  assert.is(refused.stderr.includes('one.txt'), false)

  session.edit(HANDOFF, '# Handoff\n\nAdded one and two.\n')
  assert.is(session.stop().status, 0)

  session.prompt()
  session.edit('three.txt', 'three\n')
  assert.is(session.stop().status, 2)
})

test('the journal records each move, by description for commands', (assert) => {
  const session = startSession(true)
  session.edit('one.txt', 'one\n')
  session.bash('Look around', () => {})
  session.bash('Commit it', () => {
    git(session.repo, 'add', '-A')
    git(session.repo, 'commit', '-q', '-m', 'Work')
  })
  session.edit(HANDOFF, '# Handoff\n\nCommitted one.\n')

  const entries = session.journal()
  const raw = fs.readFileSync(path.join(session.repo, JOURNAL), 'utf8')

  assert.deepEqual(
    entries.map(({ tool, what, handoff }) => [tool, what, handoff]),
    [
      ['Edit', 'one.txt', false],
      ['Bash', 'Commit it', false],
      ['Edit', HANDOFF, true]
    ]
  )
  assert.truthy(entries.every((entry) => entry.session === session.id))
  assert.truthy(entries.every((entry) => !isNaN(Date.parse(entry.ts))))
  assert.is(raw.includes('hunter2'), false)
  assert.is(session.stop().status, 0)
})

test('changes nobody journaled are caught at stop', (assert) => {
  const session = startSession(true)
  write(session.repo, 'sneaky.txt', 'made by a tool with no hook\n')

  const refused = session.stop()
  assert.is(refused.status, 2)
  assert.truthy(refused.stderr.includes('- untracked'))
})

test('the human\'s edits between turns are logged but not owed', (assert) => {
  const session = startSession(true)
  assert.is(session.stop().status, 0)

  write(session.repo, 'human.txt', 'typed by hand\n')
  session.prompt()
  session.bash('Run the tests', () => {})

  assert.is(session.stop().status, 0)
  assert.deepEqual(session.journal().map(({ tool }) => tool), [OUTSIDE])
})

test('stop refuses when the handoff was deleted', (assert) => {
  const session = startSession(true)
  session.bash('Tidy up', () => fs.rmSync(path.join(session.repo, HANDOFF)))

  const refused = session.stop()
  assert.is(refused.status, 2)
  assert.truthy(refused.stderr.includes('is gone'))
})

test('stop gives up loudly after MAX_BLOCKS refusals', (assert) => {
  const session = startSession(true)
  session.edit('work.txt', 'work\n')

  for (let i = 0; i < MAX_BLOCKS; i++) assert.is(session.stop().status, 2)

  const gaveUp = session.stop()
  const { systemMessage } = JSON.parse(gaveUp.stdout)
  assert.is(gaveUp.status, 0)
  assert.truthy(systemMessage.includes(path.join(session.repo, HANDOFF)))
})

test('the next session is told what the last one left unhanded', (assert) => {
  const first = startSession(true)
  first.edit('work.txt', 'work\n')
  for (let i = 0; i <= MAX_BLOCKS; i++) first.stop()

  const second = createSession(first.repo, 'second')
  const started = second.start()
  assert.is(started.status, 0)
  assert.truthy(started.stdout.includes('- Edit work.txt'))
  assert.truthy(started.stdout.includes(path.join(first.repo, HANDOFF)))

  second.edit(HANDOFF, '# Handoff\n\nCaught up.\n')
  assert.is(second.stop().status, 0)
  assert.is(createSession(first.repo, 'third').start().stdout.includes('earlier'), false)
})

test('a hook that arrives mid-session starts from now', (assert) => {
  const repo = createRepo(true)
  const session = createSession(repo, `late-${seq}`)
  write(repo, 'before.txt', 'before\n')

  assert.is(session.stop().status, 0)
  session.edit('after.txt', 'after\n')
  assert.is(session.stop().status, 2)
})

test('resuming a session keeps what it owes', (assert) => {
  const session = startSession(true)
  session.edit('work.txt', 'work\n')

  assert.is(session.start().status, 0)
  assert.is(session.stop().status, 2)
})

// harness adapters

test('the claude adapter turns its hook input into a common event', (assert) => {
  const input = { session_id: 's', cwd: '/in', tool_name: 'Edit', tool_input: { a: 1 } }
  const cases = [
    ['SessionStart', {}, 'start', '/in'],
    ['UserPromptSubmit', {}, 'prompt', '/in'],
    ['PostToolUse', { CLAUDE_PROJECT_DIR: '/env' }, 'tool', '/env'],
    ['Stop', {}, 'stop', '/in'],
    ['SubagentStart', {}, 'agent-start', '/in'],
    ['SubagentStop', {}, 'agent-stop', '/in'],
    ['PreCompact', {}, undefined, '/in']
  ]

  cases.forEach(([event, env, name, dir]) => {
    assert.deepEqual(claude.parse({ ...input, hook_event_name: event }, env), {
      name,
      id: 's',
      dir,
      cwd: '/in',
      agent: undefined,
      tool: 'Edit',
      input: { a: 1 }
    })
  })
  assert.deepEqual(claude.emit({}, { refusal: 'no' }), { stderr: 'no', code: 2 })
  assert.deepEqual(claude.emit({}, { context: 'hi' }), { stdout: 'hi' })
  assert.deepEqual(claude.emit({}, {}), { stdout: undefined })
  assert.deepEqual(
    JSON.parse(claude.emit({}, { notice: 'gave up' }).stdout),
    { systemMessage: 'gave up' }
  )
})

test('the cursor adapter does the same from cursor\'s shapes', (assert) => {
  const base = { cursor_version: '3', conversation_id: 'c', workspace_roots: ['/ws'] }
  const edit = { file_path: '/ws/a.js' }
  const cases = [
    [{ hook_event_name: 'sessionStart', session_id: 's' }, {}, 'start', undefined, undefined],
    [{ hook_event_name: 'beforeSubmitPrompt' }, {}, 'prompt', undefined, undefined],
    [{ hook_event_name: 'UserPromptSubmit' }, {}, 'prompt', undefined, undefined],
    [{ hook_event_name: 'postToolUse', tool_name: 'Shell', tool_input: { command: 'x' } },
      {}, 'tool', 'Bash', { command: 'x' }],
    [{ hook_event_name: 'postToolUse', tool_name: 'Write', tool_input: edit },
      {}, 'tool', 'Write', edit],
    [{ hook_event_name: 'afterFileEdit', file_path: '/ws/a.js' }, {}, 'tool', 'Write', edit],
    [{ hook_event_name: 'afterShellExecution', command: 'SECRET=1 x' }, {}, 'tool', 'Bash', {}],
    [{ hook_event_name: 'stop', status: 'completed' }, {}, 'stop', undefined, undefined],
    [{ hook_event_name: 'preCompact' }, {}, undefined, undefined, undefined]
  ]

  cases.forEach(([input, env, name, tool, toolInput]) => {
    const event = cursor.parse({ ...base, ...input }, env)

    assert.deepEqual(event, { name, id: 'c', dir: '/ws', tool, input: toolInput })
  })
  assert.is(cursor.parse(base, { CURSOR_PROJECT_DIR: '/env' }).dir, '/env')
  assert.is(cursor.parse({ session_id: 's' }, {}).id, 's')

  const out = (name, outcome) => JSON.parse(cursor.emit({ name }, outcome).stdout)
  assert.deepEqual(out('start', { context: 'hi' }), { additional_context: 'hi' })
  assert.deepEqual(out('start', {}), {})
  assert.deepEqual(out('prompt', {}), { continue: true })
  assert.deepEqual(out('tool', {}), {})
  assert.deepEqual(out('stop', { refusal: 'no' }), { followup_message: 'no' })
  assert.deepEqual(out('stop', { notice: 'gave up' }), { user_message: 'gave up' })
  assert.is(cursor.emit({ name: 'stop' }, { refusal: 'no' }).code, undefined)
})

test('the same gate runs from cursor-shaped hook calls', (assert) => {
  const repo = createRepo(true)
  const id = `cursor-${seq}`
  const call = (event, extra) => {
    const ran = runHook(repo, undefined, event, {
      cursor_version: '3.14.7',
      conversation_id: id,
      workspace_roots: [repo],
      cwd: '',
      ...extra
    })

    return { status: ran.status, stderr: ran.stderr, out: JSON.parse(ran.stdout) }
  }

  const started = call('sessionStart', { session_id: id })
  assert.is(started.status, 0)
  assert.truthy(started.out.additional_context.includes(path.join(repo, NOTES)))
  assert.deepEqual(call('beforeSubmitPrompt').out, { continue: true })
  assert.deepEqual(call('stop').out, {})

  write(repo, 'work.txt', 'work\n')
  call('postToolUse', {
    tool_name: 'Write',
    tool_input: { file_path: path.join(repo, 'work.txt') }
  })
  write(repo, 'more.txt', 'more\n')
  call('postToolUse', { tool_name: 'Shell', tool_input: { command: 'SECRET=hunter2 x' } })

  const refused = call('stop', { status: 'completed', loop_count: 0 })
  assert.is(refused.status, 0)
  assert.is(refused.stderr, '')
  assert.truthy(refused.out.followup_message.includes('- Write work.txt\n- Bash'))
  assert.is(read(repo, JOURNAL).includes('hunter2'), false)

  write(repo, HANDOFF, '# Handoff\n\nDid work.\n')
  call('afterFileEdit', { file_path: path.join(repo, HANDOFF) })
  assert.deepEqual(call('stop').out, {})
})

test('the cursor plugin points at hooks that exist, at the same version', (assert) => {
  const plugin = JSON.parse(read(HERE, '.cursor-plugin', 'plugin.json'))
  const { hooks, version } = JSON.parse(read(HERE, plugin.hooks))

  assert.is(plugin.version, JSON.parse(read(HERE, 'package.json')).version)
  assert.is(version, 1)
  assert.deepEqual(
    Object.keys(hooks),
    ['beforeShellExecution', 'sessionStart', 'beforeSubmitPrompt', 'postToolUse', 'stop']
  )
  Object.values(hooks).flat().forEach(({ command }) => {
    assert.truthy(command.endsWith('/hooks/hook.js"'))
  })
  assert.truthy(hooks.stop[0].loop_limit > MAX_BLOCKS)
})

// subagents

test('live counts subagents still out and drops ones never seen to stop', (assert) => {
  const hour = 60 * 60 * 1000
  const at = 100 * hour

  assert.is(live(undefined, at), 0)
  assert.is(live({}, at), 0)
  assert.is(live({ a: at - hour, b: at - 5 * hour }, at), 2)
  assert.is(live({ a: at - hour, b: at - 7 * hour }, at), 1)
})

test('decide waits for subagents only when a handoff is owed', (assert) => {
  const owed = [{ tool: 'Edit', what: 'a.js', handoff: false }]
  const cases = [
    [{ stamped: true, pending: owed }, { handoff: 'a' }, 1, false, 'agents-out'],
    [{ stamped: true, pending: owed }, { handoff: 'a' }, 0, true, 'handoff-stale'],
    [{ stamped: true, pending: [] }, { handoff: 'a' }, 2, false, 'current'],
    [{ stamped: true, pending: owed }, { handoff: null }, 2, true, 'handoff-missing']
  ]

  cases.forEach(([state, now, out, block, why]) => {
    assert.deepEqual(decide(state, now, out), { block, why })
  })
})

test('a subagent\'s moves are the session\'s, and stop waits until it is back', (assert) => {
  const session = startSession(true)
  const agent = { agent_id: 'a1', agent_type: 'general-purpose' }
  const file = path.join(session.repo, 'sub.txt')

  session.hook('SubagentStart', agent)
  write(session.repo, 'sub.txt', 'sub\n')
  session.hook('PostToolUse', { ...agent, tool_name: 'Write', tool_input: { file_path: file } })

  assert.is(session.stop().status, 0)
  assert.deepEqual(
    session.journal().map(({ tool, what, agent }) => [tool, what, agent]),
    [['Write', 'sub.txt', 'a1']]
  )

  session.hook('SubagentStop', agent)
  const refused = session.stop()
  assert.is(refused.status, 2)
  assert.truthy(refused.stderr.includes('- Write sub.txt'))
})

test('work in a subagent\'s worktree is journaled and owed', (assert) => {
  const session = startSession(true)
  const { repo } = session
  const inside = '.claude/worktrees/agent-a2'
  const tree = path.join(repo, '.claude', 'worktrees', 'agent-a2')
  const agent = { agent_id: 'a2', cwd: tree }
  const post = (tool, toolInput) => session.hook('PostToolUse', {
    ...agent,
    tool_name: tool,
    tool_input: toolInput
  })
  git(repo, 'worktree', 'add', '-q', '-b', 'side', tree)

  session.hook('SubagentStart', agent)
  post('Bash', { description: 'Look around' })
  write(repo, `${inside}/wt.txt`, 'wt\n')
  post('Write', { file_path: path.join(tree, 'wt.txt') })
  post('Bash', { description: 'Look again' })
  git(tree, 'add', '-A')
  git(tree, 'commit', '-q', '-m', 'Side work')
  post('Bash', { description: 'Commit it' })
  session.hook('SubagentStop', agent)

  assert.deepEqual(
    session.journal().map(({ tool, what, agent }) => [tool, what, agent]),
    [['Write', `${inside}/wt.txt`, 'a2'], ['Bash', 'Commit it', 'a2']]
  )
  assert.is(fs.existsSync(path.join(tree, JOURNAL)), false)

  const refused = session.stop()
  assert.is(refused.status, 2)
  assert.truthy(refused.stderr.includes(`- Write ${inside}/wt.txt`))
  assert.truthy(refused.stderr.includes(path.join(repo, HANDOFF)))
  assert.is(refused.stderr.includes('untracked'), false)

  git(repo, 'merge', '-q', 'side')
  session.hook('PostToolUse', { tool_name: 'Bash', tool_input: { description: 'Merge side' } })
  session.edit(HANDOFF, '# Handoff\n\nMerged side.\n')
  assert.is(session.stop().status, 0)
})

test('a directory in some other repo is not a side of this one', (assert) => {
  const session = startSession(true)
  const other = createRepo(false)

  write(other, 'elsewhere.txt', 'x\n')
  session.hook('PostToolUse', {
    cwd: other,
    tool_name: 'Write',
    tool_input: { file_path: path.join(other, 'elsewhere.txt') }
  })

  assert.deepEqual(session.journal(), [])
  assert.is(session.stop().status, 0)
})

// the commit and push guard

// assembled here so that this file never holds a line that looks secret
const FAKE = {
  aws: ['AKIA', 'ABCDEFGHIJKLMNOP'].join(''),
  github: ['ghp_', 'a'.repeat(36)].join(''),
  key: ['-----BEGIN RSA', ' PRIVATE KEY-----'].join(''),
  assigned: ['api_key = "', 'q'.repeat(20), '"'].join('')
}

test('classify spots commits and anything that leaves the machine', (assert) => {
  const cases = [
    ['git commit -m "x"', ['commit']],
    ['git -C sub commit --amend', ['commit']],
    ['git add -A && git commit -m x && git push', ['commit', 'push']],
    ['git push origin main', ['push']],
    ['git tag 1.0.0', ['push']],
    ['git tag -a 1.0.0 -m "One"', ['push']],
    ['git tag -d 1.0.0', ['push']],
    ['git tag', []],
    ['git tag -l', []],
    ['git tag --list "1.*"', []],
    ['git tag --points-at HEAD', []],
    ['git tag --contains abc123', []],
    ['git tag -v 1.0.0', []],
    ['npm publish', ['push']],
    ['cargo publish --dry-run', ['push']],
    ['gh release create 1.0.0', ['push']],
    ['gh release upload 1.0.0 a.zip', ['push']],
    ['gh release view 1.0.0', []],
    ['gh release download 1.0.0', []],
    ['gh release list', []],
    ['gh pr create --fill', ['push']],
    ['heeded config --local', ['settings']],
    ['node "C:/x/bin/heeded.js" config', ['settings']],
    ['heeded config --show', []],
    ['heeded help config', []],
    ['git status', []],
    ['git log --grep commit', []],
    ['echo commit', []],
    ['npm run check', []],
    ['', []]
  ]

  cases.forEach(([command, acts]) => assert.deepEqual(classify(command), acts, command))
  assert.is(stages('git add -A && git commit -m x'), true)
  assert.is(stages('git commit -am x'), true)
  assert.is(stages('git commit --all -m x'), true)
  assert.is(stages('git commit -m "add a thing"'), false)
})

test('secrets are found by kind, and never in ordinary lines', (assert) => {
  const cases = [
    [`+const k = "${FAKE.aws}"`, 'an AWS access key'],
    [`+token: ${FAKE.github}`, 'a GitHub token'],
    [`+${FAKE.key}`, 'a private key'],
    [`+${FAKE.assigned}`, 'a hard-coded secret'],
    [`+${FAKE.assigned} // heeded: not a secret`, null],
    ['+const token = process.env.TOKEN', null],
    ['+password = "hunter2"', null],
    ['+const sha = "0123456789abcdef0123456789abcdef01234567"', null]
  ]
  const diff = [
    'diff --git a/lib/a.js b/lib/a.js',
    '--- a/lib/a.js',
    '+++ b/lib/a.js',
    '@@ -0,0 +1 @@',
    `+const k = "${FAKE.aws}"`,
    `-const old = "${FAKE.github}"`,
    '+++ b/lib/b.js',
    '+const fine = 1'
  ].join('\n')

  cases.forEach(([line, kind]) => assert.is(secretIn(line), kind, line.slice(0, 20)))
  assert.deepEqual(scanDiff(diff), [['lib/a.js', 'an AWS access key']])
  assert.deepEqual(
    ['.env', 'a/.env.local', 'id_rsa', 'x.pem', '.env.example', 'env.js', 'key.js']
      .map(secretFile),
    [true, true, true, true, false, false, false]
  )
})

test('identity, attribution and changelog checks', (assert) => {
  const me = { name: 'A C', email: 'a@c.dev' }
  const other = { name: 'A C', email: 'work@corp.com' }
  const identity = [
    ['git commit -m x', me, me, false],
    ['git commit -m x', other, me, true],
    ['git commit -m x', other, { name: '', email: '' }, false],
    ['git commit --author="X <x@y.z>" -m x', me, me, true],
    ['git -c user.email=a@c.dev -c user.name="A C" commit -m x', other, me, false]
  ]
  const trailer = 'git commit -m "x\n\nCo-Authored-By: M <m@x.y>"'
  const attribution = [
    [trailer, 'any', false],
    [trailer, 'forbidden', true],
    [trailer, 'required', false],
    ['git commit -m x', 'required', true],
    ['git commit -m x', 'forbidden', false]
  ]
  const owed = [
    [['lib/a.js'], '.heeded', true],
    [['lib/a.js', 'CHANGELOG.md'], '.heeded', false],
    [['test.js', 'docs/x.md', 'README.md', '.gitignore'], '.heeded', false],
    [['notes/plan.txt'], 'notes', false],
    [['notes/plan.txt'], '', true],
    [[], '.heeded', false]
  ]

  identity.forEach(([command, actual, wanted, bad]) => {
    assert.is(Boolean(identityProblem(command, actual, wanted)), bad, command)
  })
  attribution.forEach(([command, policy, bad]) => {
    assert.is(Boolean(attributionProblem(command, policy)), bad, policy)
  })
  owed.forEach(([files, notes, is]) => assert.is(changelogOwed(files, notes), is))
})

test('settings layer the repo over the user over the defaults', (assert) => {
  const home = path.join(tmp, `home-${++seq}`)
  const repo = path.join(tmp, `settings-${seq}`)
  fs.mkdirSync(repo)

  assert.deepEqual(settingsFor(repo, home), DEFAULTS)

  write(home, 'config.json', JSON.stringify({
    permission: { commit: 'allow' },
    identity: { email: 'a@c.dev' },
    junk: 1
  }))
  write(repo, MARKER, JSON.stringify({ notes: 'n', permission: { push: 'allow', commit: 'ask' } }))

  assert.deepEqual(settingsFor(repo, home), {
    permission: { commit: 'ask', push: 'allow' },
    identity: { name: '', email: 'a@c.dev', attribution: 'any' },
    checks: { changelog: false, quiet: [] },
    directives: {}
  })

  write(home, 'config.json', 'not json')
  assert.deepEqual(settingsFor(repo, home).identity, DEFAULTS.identity)
})

test('the guard asks, refuses or stays out of the way', (assert) => {
  const session = startSession(true)
  const { repo } = session
  const pre = (tool, toolInput) => {
    const ran = session.hook('PreToolUse', { tool_name: tool, tool_input: toolInput })
    const out = ran.stdout ? JSON.parse(ran.stdout).hookSpecificOutput : {}

    assert.is(ran.status, 0)

    return [out.permissionDecision, out.permissionDecisionReason || '']
  }
  const bash = (command) => pre('Bash', { command })

  assert.deepEqual(bash('npm run check'), [undefined, ''])
  assert.is(bash('git push')[0], 'ask')
  assert.is(pre('Edit', { file_path: path.join(repo, 'lib', 'a.js') })[0], undefined)
  assert.is(pre('Edit', { file_path: path.join(repo, MARKER) })[0], 'ask')
  assert.is(pre('Write', { file_path: path.join(tmp, 'home', 'config.json') })[0], 'ask')

  write(repo, 'lib/a.js', 'export const a = 1\n')
  git(repo, 'add', '-A')
  assert.is(bash('git commit -m "Add a"')[0], 'ask')

  write(repo, MARKER, JSON.stringify({ notes: NOTES, checks: { changelog: true } }))
  const nag = bash('git commit -m "Add a"')
  assert.is(nag[0], 'deny')
  assert.truthy(nag[1].includes('CHANGELOG.md'))
  assert.is(bash('git commit -m "Add a"')[0], 'ask')

  write(repo, 'CHANGELOG.md', '# Changelog\n\n## [Unreleased]\n\n- You can now a.\n')
  write(repo, 'lib/b.js', `export const k = "${FAKE.aws}"\n`)
  const leaked = bash('git add -A && git commit -m "Add b"')
  assert.is(leaked[0], 'deny')
  assert.truthy(leaked[1].includes('lib/b.js: an AWS access key'))
  assert.is(leaked[1].includes(FAKE.aws), false)
  assert.is(bash('git commit -m "Add a"')[0], 'ask')

  fs.rmSync(path.join(repo, 'lib', 'b.js'))
  write(repo, '.env', 'X=1\n')
  assert.truthy(bash('git add -A && git commit -m x')[1].includes('.env: a credentials file'))
  fs.rmSync(path.join(repo, '.env'))

  write(repo, MARKER, JSON.stringify({
    notes: NOTES,
    permission: { commit: 'allow' },
    identity: { email: 'someone@else.dev', attribution: 'forbidden' }
  }))
  const wrong = bash('git commit -m "x\n\nCo-Authored-By: M <m@x.y>"')
  assert.is(wrong[0], 'deny')
  assert.truthy(wrong[1].includes('git user.email is "test@example.com"'))
  assert.truthy(wrong[1].includes('Co-Authored-By'))

  git(repo, 'config', 'user.email', 'someone@else.dev')
  assert.deepEqual(bash('git commit -m "Add a"'), [undefined, ''])
  assert.is(bash('git commit -m "Add a" && git push')[0], 'ask')
})

test('the guard says nothing in a repo that has not opted in', (assert) => {
  const session = startSession(false)
  write(session.repo, 'lib/b.js', `export const k = "${FAKE.aws}"\n`)
  const ran = session.hook('PreToolUse', {
    tool_name: 'Bash',
    tool_input: { command: 'git add -A && git commit -m x && git push' }
  })

  assert.is(ran.status, 0)
  assert.is(ran.stdout, '')
})

// the rework log

test('checkRework holds a line to the closed sets', (assert) => {
  const cases = [
    [{ task: 'x', outcome: 'kept', failureClass: 'none' }, []],
    [{ task: 'x', outcome: 'reworked', failureClass: 'spec-gap' }, []],
    [{ task: 'x', outcome: 'discarded', failureClass: 'wrong-context' }, []],
    [{ task: 'x', outcome: 'kept', failureClass: 'spec-gap' }, ['class-for-outcome']],
    [{ task: 'x', outcome: 'reworked', failureClass: 'none' }, ['class-for-outcome']],
    [{ task: ' ', outcome: 'fine', failureClass: 'vibes' }, ['task', 'outcome', 'class']],
    [{}, ['task', 'outcome', 'class']]
  ]

  cases.forEach(([entry, problems]) => assert.deepEqual(checkRework(entry), problems))
  assert.is(OUTCOMES.length, 3)
  assert.is(CLASSES.length, 6)
})

test('heeded retro appends one line, or refuses and says why', (assert) => {
  const repo = createRepo(true)
  const good = ['retro', '--task', 'Add init', '--outcome', 'reworked', '--class', 'spec-gap']
  const ran = runBin(repo, ...good, '--note', 'docs/ was not ours')
  const bad = runBin(repo, 'retro', '--task', 'x', '--outcome', 'kept', '--class', 'spec-gap')
  const lines = read(repo, NOTES, 'rework.jsonl').trim().split('\n').map(JSON.parse)

  assert.is(ran.status, 0)
  assert.is(bad.status, 1)
  assert.truthy(bad.stderr.startsWith('Not logged, bad class-for-outcome.'))
  assert.is(lines.length, 1)
  assert.truthy(!isNaN(Date.parse(lines[0].ts)))
  assert.deepEqual({ ...lines[0], ts: 0 }, {
    ts: 0,
    task: 'Add init',
    outcome: 'reworked',
    failureClass: 'spec-gap',
    note: 'docs/ was not ours'
  })
  assert.is(runBin(createRepo(false), ...good).status, 1)
})

// the user's config and their directives

test('directives layer by name, come from text or a file, and can be switched off', (assert) => {
  const home = path.join(tmp, `home-${++seq}`)
  const repo = path.join(tmp, `directed-${seq}`)
  const long = 'x'.repeat(config.DIRECTIVE_MAX + 50)
  fs.mkdirSync(repo)
  write(home, 'voice.md', 'Short sentences.\nNo filler.\n')
  write(home, 'long.md', long)
  write(repo, 'notes/style.md', 'Tabs here.\n')
  write(path.dirname(repo), 'outside.md', 'not for you\n')
  write(home, 'config.json', JSON.stringify({
    directives: {
      voice: { file: 'voice.md' },
      style: 'Two spaces.',
      tone: 'Dry.',
      long: { file: 'long.md' },
      blank: '  ',
      junk: 7
    }
  }))
  write(repo, MARKER, JSON.stringify({
    notes: 'n',
    directives: {
      style: { file: 'notes/style.md' },
      tone: false,
      missing: { file: 'nope.md' },
      escape: { file: '../outside.md' },
      absolute: { file: path.join(home, 'voice.md') }
    }
  }))

  const said = Object.fromEntries(directivesOf(settingsFor(repo, home)))

  assert.deepEqual(Object.keys(said), ['voice', 'style', 'long', 'missing'])
  assert.is(said.voice, 'Short sentences.\nNo filler.')
  assert.is(said.style, 'Tabs here.')
  assert.truthy(said.long.endsWith(`(cut at ${config.DIRECTIVE_MAX} characters)`))
  assert.truthy(said.missing.startsWith('(heeded could not read '))
  assert.deepEqual(directivesOf(settingsFor(repo, path.join(tmp, 'no-such-home'))).map(([name]) => name), ['style', 'missing'])
  assert.deepEqual(directivesOf(DEFAULTS), [])
})

test('a stamped session is told the directives after the rules', (assert) => {
  const home = path.join(tmp, `home-${++seq}`)
  const gate = createGate(path.join(tmp, `state-${seq}`), home)
  const repo = createRepo(true)
  const start = (dir) => gate.handle({ name: 'start', id: `d-${++seq}`, dir }).context

  assert.is(start(repo).includes('directives'), false)

  write(home, 'voice.md', 'Short sentences.\nNo filler.\n')
  write(home, 'config.json', JSON.stringify({
    directives: { style: 'Two spaces.', voice: { file: 'voice.md' } }
  }))
  const told = start(repo)

  assert.truthy(told.includes('- **style**: Two spaces.\n- **voice**:\nShort sentences.\nNo filler.'))
  assert.truthy(told.indexOf('working notes are in') < told.indexOf('standing directives'))
  assert.truthy(told.includes('can add to these, not loosen them.\n- **style**'))
  assert.is(start(createRepo(false)), undefined)
})

test('heeded config writes the user config once and shows what is in effect', (assert) => {
  const home = path.join(tmp, `home-${++seq}`)
  const repo = createRepo(true)
  const file = path.join(home, 'config.json')
  const global = (key) => {
    const ran = spawnSync('git', ['config', '--global', key], { encoding: 'utf8' })

    return (ran.stdout || '').trim()
  }
  const first = runBinAt(home, repo, 'config')
  const written = JSON.parse(read(file))

  assert.is(first.status, 0)
  assert.truthy(first.stdout.startsWith(`created ${file}\n`))
  assert.deepEqual(written, {
    ...DEFAULTS,
    identity: { name: global('user.name'), email: global('user.email'), attribution: 'any' }
  })

  write(home, 'config.json', JSON.stringify({
    permission: { commit: 'allow' },
    directives: { voice: 'Dry.' }
  }))
  const second = runBinAt(home, repo, 'config')
  const shown = JSON.parse(second.stdout.slice(second.stdout.indexOf('{')))

  assert.truthy(second.stdout.startsWith(`kept ${file}\n`))
  assert.is(JSON.parse(read(file)).permission.commit, 'allow')
  assert.deepEqual(shown.permission, { commit: 'allow', push: 'ask' })
  assert.deepEqual(shown.directives, ['voice (4 characters)'])
})

// the interactive config, help, and the gazelle fixes

const scripted = (answers) => {
  const asked = []
  const said = []
  const ask = async (question) => {
    asked.push(question)

    return answers.length ? answers.shift() : ''
  }

  return { ask, say: (text) => said.push(text), asked, said }
}

test('pick reads an answer for each kind of step', (assert) => {
  const [commit, , name, , attribution, changelog, quiet] = STEPS
  const KEEP = pick(commit, '', false)
  const cases = [
    [commit, 'allow', false, 'allow'],
    [commit, 'al', false, 'allow'],
    [commit, 'a', false, null],
    [commit, 'nope', false, null],
    [commit, 'inherit', true, undefined],
    [commit, 'inherit', false, null],
    [attribution, 'f', false, 'forbidden'],
    [changelog, 'on', false, true],
    [changelog, 'off', false, false],
    [name, 'A C', false, 'A C'],
    [name, '-', false, ''],
    [name, 'inherit', true, undefined]
  ]

  cases.forEach(([step, answer, local, value]) => {
    assert.is(pick(step, answer, local), value, `${step.key} ${answer}`)
  })
  assert.is(typeof KEEP, 'symbol')
  assert.is(pick(name, '   ', false), KEEP)
  assert.deepEqual(pick(quiet, 'a/, b ,,c', false), ['a/', 'b', 'c'])
  assert.deepEqual(pick(quiet, '-', false), [])
})

test('set and get walk one level of settings', (assert) => {
  const start = { notes: 'n', permission: { commit: 'ask' } }

  assert.is(get(start, 'permission.commit'), 'ask')
  assert.is(get(start, 'identity.name'), undefined)
  assert.deepEqual(set(start, 'identity.name', 'A'), { ...start, identity: { name: 'A' } })
  assert.deepEqual(set(start, 'permission.commit', undefined), { notes: 'n' })
  assert.deepEqual(start, { notes: 'n', permission: { commit: 'ask' } })
})

test('walk steps through every setting and the directives, globally', async (assert) => {
  const start = { ...DEFAULTS, directives: { tone: 'Dry.' }, extra: 1 }
  const io = scripted([
    'nope', 'allow', '', 'A C', '-', 'f', 'on', 'e2e/, xtask/',
    'voice', 'Short.', 'style', '@style.md', 'tone', 'remove', 'bad name!', 'x', 'off', ''
  ])
  const next = await walk(start, { ...io, local: false, inherited: DEFAULTS })

  assert.deepEqual(next, {
    permission: { commit: 'allow', push: 'ask' },
    identity: { name: 'A C', email: '', attribution: 'forbidden' },
    checks: { changelog: true, quiet: ['e2e/', 'xtask/'] },
    directives: { voice: 'Short.', style: { file: 'style.md' } },
    extra: 1
  })
  assert.is(io.asked.filter((q) => q.startsWith('Commit without asking')).length, 2)
  assert.truthy(io.asked[0].includes('[ask/allow] (now ask)'))
  assert.truthy(io.said.includes('One of: ask/allow.'))
  assert.truthy(io.said.includes('A name is letters, digits and dashes.'))
  assert.truthy(io.said.includes('Not an answer; nothing changed.'))
})

test('walk locally offers inherit, and keeps the marker small', async (assert) => {
  const inherited = { ...DEFAULTS, permission: { commit: 'allow', push: 'ask' } }
  const start = { notes: '.agent', permission: { commit: 'ask' }, directives: { voice: 'x' } }
  const io = scripted(['inherit', '', '', '', '', '', 'web/e2e/', 'voice', 'off', 'tone', 'remove', ''])
  const next = await walk(start, { ...io, local: true, inherited })

  assert.deepEqual(next, {
    notes: '.agent',
    checks: { quiet: ['web/e2e/'] },
    directives: { voice: false }
  })
  assert.truthy(io.asked[0].includes('[ask/allow/inherit] (now ask)'))
  assert.truthy(io.asked[1].includes('(now inherited: ask)'))

  const empty = await walk({ notes: 'n' }, { ...scripted([]), local: true, inherited })
  assert.deepEqual(empty, { notes: 'n' })
})

test('heeded help prints the summary, or one command, and succeeds', (assert) => {
  const cases = [[], ['help'], ['-h'], ['--help'], ['help', 'config'], ['config', '--help']]

  cases.forEach((args) => {
    const ran = runBin(tmp, ...args)

    assert.is(ran.status, 0, args.join(' '))
    assert.is(ran.stderr, '', args.join(' '))
  })
  assert.truthy(runBin(tmp).stdout.includes('config   step through your settings'))
  assert.truthy(runBin(tmp, 'help', 'config').stdout.startsWith('heeded config [dir]'))
  assert.truthy(runBin(tmp, 'config', '-h').stdout.startsWith('heeded config [dir]'))
  Object.keys(HELP_TOPICS).forEach((topic) => {
    assert.truthy(runBin(tmp, 'help', topic).stdout.startsWith(`heeded ${topic}`), topic)
  })
})

test('heeded config --global and --local write the file from answers', (assert) => {
  const home = path.join(tmp, `home-${++seq}`)
  const repo = createRepo(true)
  const answer = (args, lines) => spawnSync(...cli(['config', ...args]), {
    cwd: repo,
    encoding: 'utf8',
    env: { ...process.env, HEEDED_HOME: home },
    input: lines.join('\n')
  })

  const global = answer(['--global'], ['allow', '', '', '', '', 'on', '', 'voice', 'Short.', ''])
  const written = JSON.parse(read(home, 'config.json'))
  assert.is(global.status, 0, global.stderr)
  assert.truthy(global.stdout.includes(`wrote ${path.join(home, 'config.json')}`))
  assert.is(written.permission.commit, 'allow')
  assert.is(written.checks.changelog, true)
  assert.deepEqual(written.directives, { voice: 'Short.' })

  const local = answer(['--local'], ['ask', '', '', '', '', '', '', 'voice', 'off', ''])
  const marker = JSON.parse(read(repo, MARKER))
  assert.is(local.status, 0, local.stderr)
  assert.deepEqual(marker, {
    notes: NOTES,
    permission: { commit: 'ask' },
    directives: { voice: false }
  })
  assert.truthy(local.stdout.includes('(now inherited: allow)'))
  assert.truthy(local.stdout.includes('"directives": []'))

  const unstamped = spawnSync(...cli(['config', '--local']), {
    cwd: createRepo(false),
    encoding: 'utf8',
    env: { ...process.env, HEEDED_HOME: home },
    input: ''
  })
  assert.is(unstamped.status, 1)
  assert.truthy(unstamped.stderr.includes(`has no ${MARKER}`))
})

test('the changelog reminder stays quiet on paths a repo names', (assert) => {
  const cases = [
    [['web/apps/web/e2e/a.spec.ts'], ['web/apps/web/e2e/'], false],
    [['xtask/src/main.rs'], ['xtask'], false],
    [['xtaskish/main.rs'], ['xtask'], true],
    [['Cargo.lock'], ['Cargo.lock'], false],
    [['Cargo.lock', 'src/a.rs'], ['Cargo.lock'], true],
    [['src/a.rs'], [], true]
  ]

  cases.forEach(([files, quiet, owed]) => {
    assert.is(changelogOwed(files, '.heeded', quiet), owed, files.join(' '))
  })
})

test('targetOf follows git -C, quoted or not', (assert) => {
  const cwd = path.join(tmp, 'outer')
  const cases = [
    ['git commit -m x', cwd],
    ['git -C .agent commit -m x', path.join(cwd, '.agent')],
    ['git -C "my notes" push', path.join(cwd, 'my notes')],
    [`git -C ${path.join(tmp, 'abs')} commit`, path.join(tmp, 'abs')],
    ['git -c user.name=x commit', cwd],
    ['cd sub && git commit -m x', path.join(cwd, 'sub')],
    ['cd a; cd ../b && git commit', path.join(cwd, 'b')],
    ['cd "$S/plain" && git commit', cwd],
    ['git commit && cd elsewhere', cwd],
    [`cd ${path.join(tmp, 'abs')} && git -C inner push`, path.join(tmp, 'abs', 'inner')]
  ]

  cases.forEach(([command, where]) => assert.is(targetOf(command, cwd), where, command))
})

test('the guard judges a git -C commit by the repo it names', (assert) => {
  const session = startSession(true)
  const { repo } = session
  const inner = path.join(repo, 'private')
  write(repo, '.gitignore', 'private\n')
  fs.mkdirSync(inner)
  git(inner, 'init', '-q')
  write(repo, 'lib/a.js', 'x\n')
  git(repo, 'add', '-A')

  const bash = (command) => {
    const ran = session.hook('PreToolUse', { tool_name: 'Bash', tool_input: { command } })

    return ran.stdout ? JSON.parse(ran.stdout).hookSpecificOutput.permissionDecision : undefined
  }

  assert.is(bash('git commit -m x'), 'ask')
  assert.is(bash('git -C private commit -m notes'), undefined)
  assert.is(bash('git -C private push'), undefined)
  assert.is(bash('gh release view 1.0.0 && git tag -l'), undefined)
  assert.is(bash('heeded config --local'), 'ask')

  write(inner, MARKER, JSON.stringify({ notes: '.', permission: { commit: 'allow' } }))
  write(inner, 'handoff.md', '# Handoff\n')
  assert.is(bash('git -C private commit -m notes'), undefined)
  assert.is(bash('git -C private push'), 'ask')
})

test('a repo that never opted in costs no git and leaves no state', (assert) => {
  const repo = createRepo(false)
  const id = `fast-${seq}`
  const events = [
    ['SessionStart', {}],
    ['UserPromptSubmit', {}],
    ['PreToolUse', { tool_name: 'Bash', tool_input: { command: 'git commit -m x' } }],
    ['PostToolUse', { tool_name: 'Bash', tool_input: { description: 'x' } }],
    ['Stop', {}]
  ]

  events.forEach(([event, extra]) => {
    const ran = runHook(repo, id, event, extra)

    assert.is(ran.status, 0, event)
    assert.is(ran.stdout + ran.stderr, '', event)
  })
  assert.is(createStore(stateDir).has(id), false)
})
