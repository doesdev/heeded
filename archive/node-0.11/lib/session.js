import { MARKER } from '../config.js'
import { within } from './notes.js'

export const OUTSIDE = 'outside'
export const UNTRACKED = 'untracked'

const ASK = 'Rewrite it before stopping, using the heeded:handoff skill: ' +
  'what this session did, what it left open, and what comes next.'
const LISTED = 20
const AGENT_TTL = 6 * 60 * 60 * 1000

const relative = (root, file) => file ? within(root, file) : ''

// commands can carry secrets, so only the description is kept
const WHAT = {
  Edit: (input, root) => relative(root, input.file_path),
  Write: (input, root) => relative(root, input.file_path),
  NotebookEdit: (input, root) => relative(root, input.notebook_path),
  Bash: (input) => input.description || '',
  PowerShell: (input) => input.description || ''
}

export const list = (moves) => {
  const shown = moves.slice(-LISTED).map(({ tool, what }) => {
    return `- ${tool}${what ? ` ${what}` : ''}`
  })
  const more = moves.length - shown.length

  return (more > 0 ? [`- (${more} earlier)`, ...shown] : shown).join('\n')
}

const REASONS = {
  'marker-missing': () => `${MARKER} is gone, and it is what opts this repo ` +
    'in to heeded. Put it back before stopping, unless the owner asked for ' +
    'this repo to opt out.',
  'handoff-missing': (file) => `${file} is gone. ${ASK}`,
  'handoff-stale': (file, pending) => 'This session changed the repo after ' +
    `the last write to ${file}. ${ASK}\nMoves since:\n${list(pending)}`
}

export const open = (now) => {
  const stamped = now.handoff !== null

  return { stamped, last: now, pending: [], blocks: 0, agents: {}, sides: {} }
}

export const toMove = (tool, input, root) => {
  const what = WHAT[tool] ? WHAT[tool](input || {}, root) : ''

  return { tool, what }
}

// compares a snapshot with the last one; a change is one journal entry.
// now.side is a worktree a subagent is in: work there is this session's too
export const track = (state, now, move) => {
  const { last } = state
  const wrote = now.handoff !== null && now.handoff !== last.handoff
  const lost = now.handoff === null && last.handoff !== null
  const sides = state.sides || {}
  const aside = Boolean(now.side) && sides[now.side.root] !== now.side.tree
  const worked = lost || now.tree !== last.tree || aside
  const stamped = state.stamped || wrote
  const seen = now.side ? { ...sides, [now.side.root]: now.side.tree } : sides
  const next = { ...state, stamped, last: now, sides: seen }

  if (!stamped || (!wrote && !worked)) return { state: next, entry: null }

  const entry = { ...move, handoff: wrote }
  if (wrote) return { state: { ...next, pending: [] }, entry }
  if (move.tool === OUTSIDE) return { state: next, entry }

  return { state: { ...next, pending: [...state.pending, entry] }, entry }
}

// subagents still out, ignoring any whose stop was never seen
export const live = (agents, at) => {
  return Object.values(agents || {}).filter((ts) => at - ts < AGENT_TTL).length
}

export const decide = (state, now, out) => {
  if (!state.stamped) return { block: false, why: 'unstamped' }
  if (now.notes === null) return { block: true, why: 'marker-missing' }
  if (now.handoff === null) return { block: true, why: 'handoff-missing' }
  if (state.pending.length && out) return { block: false, why: 'agents-out' }
  if (state.pending.length) return { block: true, why: 'handoff-stale' }

  return { block: false, why: 'current' }
}

export const reasonFor = (why, file, pending) => REASONS[why](file, pending)

export const notesNote = (notes) => {
  return `heeded: this repo's working notes are in ${notes}. Before anything ` +
    'else read handoff.md there (what the last session did and left open) ' +
    'and plan.md (current intent). decisions/ holds one record per ' +
    'decision. `npm run check` is the oracle.'
}

// the owner's own additions, from config; one line stays inline
export const directivesNote = (directives) => {
  if (!directives.length) return ''

  const items = directives.map(([name, text]) => {
    return `- **${name}**:${text.includes('\n') ? '\n' : ' '}${text}`
  })

  return "heeded: the owner's standing directives, from their config. A " +
    "repo's own instructions can add to these, not loosen them.\n" +
    items.join('\n')
}

export const leftoverNote = (file, moves) => {
  return `heeded: earlier sessions changed this repo after ${file} was last ` +
    'written, so it may not describe the current state. Moves since:\n' +
    list(moves)
}
