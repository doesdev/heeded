// steps through every setting in one config file. ask(question) returns the
// answer; Enter keeps what is there. Locally, inherit drops the repo's own
// value so the user's applies.

const ON = { on: true, off: false }
const NAME = /^[a-z0-9][a-z0-9-]*$/i

export const STEPS = [
  {
    key: 'permission.commit',
    say: 'Commit without asking you first',
    choices: ['ask', 'allow']
  },
  {
    key: 'permission.push',
    say: 'Push, tag, publish or release without asking you first',
    choices: ['ask', 'allow']
  },
  { key: 'identity.name', say: 'Author name every commit must carry' },
  { key: 'identity.email', say: 'Author email every commit must carry' },
  {
    key: 'identity.attribution',
    say: 'Co-Authored-By lines on commits',
    choices: ['any', 'required', 'forbidden']
  },
  {
    key: 'checks.changelog',
    say: 'Remind once when a commit has no changelog line',
    choices: ['off', 'on'],
    values: ON
  },
  {
    key: 'checks.quiet',
    say: 'Paths the changelog reminder ignores, comma separated',
    list: true
  }
]

const KEEP = Symbol('keep')

export const get = (obj, key) => {
  return key.split('.').reduce((at, part) => (at == null ? undefined : at[part]), obj)
}

export const set = (obj, key, value) => {
  const [head, tail] = key.split('.')
  const inner = { ...(obj[head] || {}) }

  if (value === undefined) delete inner[tail]
  else inner[tail] = value

  const next = { ...obj, [head]: inner }
  if (!Object.keys(inner).length) delete next[head]

  return next
}

const shown = (step, value) => {
  if (value === undefined) return undefined
  if (step.values) return Object.keys(step.values).find((name) => step.values[name] === value)
  if (step.list) return value.length ? value.join(', ') : '(none)'

  return value === '' ? '(any)' : value
}

// the answer as a value, KEEP, or null when it is not an answer
export const pick = (step, answer, local) => {
  const said = answer.trim()

  if (!said) return KEEP
  if (local && said === 'inherit') return undefined
  if (step.list) return said === '-' ? [] : said.split(',').map((part) => part.trim()).filter(Boolean)
  if (!step.choices) return said === '-' ? '' : said

  const found = step.choices.filter((choice) => choice.startsWith(said.toLowerCase()))
  if (found.length !== 1) return null

  return step.values ? step.values[found[0]] : found[0]
}

const optionsFor = (step, local) => {
  const base = step.choices ? step.choices.join('/') : 'text, - for none'

  return local ? `${base}/inherit` : base
}

const question = (step, own, inherited, local) => {
  const theirs = shown(step, get(inherited, step.key))
  const mine = shown(step, own)
  const now = mine === undefined
    ? (local ? `inherited: ${theirs || '(any)'}` : theirs || '(any)')
    : mine

  return `${step.say} [${optionsFor(step, local)}] (now ${now}): `
}

const describe = (value) => {
  if (value === false) return 'off here'
  if (value && value.file) return `from ${value.file}`
  if (typeof value !== 'string') return '(not usable)'

  return value.length > 60 ? `${value.slice(0, 57)}...` : value
}

const directive = (answer, local) => {
  const said = answer.trim()

  if (!said) return KEEP
  if (said === 'remove') return undefined
  if (said === 'off') return local ? false : null
  if (said.startsWith('@')) return said.length > 1 ? { file: said.slice(1) } : null

  return said
}

const directives = async (start, { ask, say, local }) => {
  let own = { ...(start.directives || {}) }

  for (;;) {
    const listed = Object.entries(own).map(([name, value]) => `  ${name}: ${describe(value)}`)
    say(`Directives in this file:\n${listed.length ? listed.join('\n') : '  (none)'}`)

    const name = (await ask('Name one to add or change, or Enter to finish: ')).trim()
    if (!name) break
    if (!NAME.test(name)) {
      say('A name is letters, digits and dashes.')
      continue
    }

    const off = local ? ', off to switch it off here' : ''
    const value = directive(await ask(
      `${name}: new text, @path to read it from a file${off}, remove to delete, Enter to keep: `
    ), local)

    if (value === null) say('Not an answer; nothing changed.')
    else if (value === undefined) delete own[name]
    else if (value !== KEEP) own = { ...own, [name]: value }
  }

  const next = { ...start, directives: own }
  if (!Object.keys(own).length && local) delete next.directives

  return next
}

export const walk = async (start, { ask, say, local, inherited }) => {
  let config = start

  for (const step of STEPS) {
    let value = null

    while (value === null) {
      value = pick(step, await ask(question(step, get(config, step.key), inherited, local)), local)
      if (value === null) say(`One of: ${optionsFor(step, local)}.`)
    }

    if (value !== KEEP) config = set(config, step.key, value)
  }

  return directives(config, { ask, say, local })
}
