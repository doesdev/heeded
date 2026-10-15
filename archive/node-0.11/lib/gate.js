import { JOURNAL_MAX, MAX_BLOCKS } from '../config.js'
import { snapshot } from './handoff.js'
import { createJournal, unhanded } from './journal.js'
import {
  OUTSIDE,
  UNTRACKED,
  decide,
  directivesNote,
  live,
  leftoverNote,
  notesNote,
  open,
  reasonFor,
  toMove,
  track
} from './session.js'
import { directivesOf, settingsFor } from './settings.js'
import { createStore } from './store.js'

// the whole gate, with no idea which harness is asking: an event comes in
// as { name, id, dir, cwd, agent, tool, input } and what to say goes out as
// { context, refusal, notice }
export const createGate = (stateDir, home) => {
  const store = createStore(stateDir)

  const handle = ({ name, id, dir, cwd, agent, tool, input }) => {
    const now = snapshot(dir, cwd)
    const journal = createJournal(now.journal)

    const sync = (move) => {
      const { state, entry } = track(store.load(id) || open(now), now, move)
      const ts = new Date().toISOString()

      if (entry && now.journal) journal.append({ ts, session: id, ...entry })
      store.save(id, state)

      return state
    }

    const start = () => {
      const state = sync({ tool: OUTSIDE, what: '' })
      if (!state.stamped || !now.notes) return {}

      journal.trim(JOURNAL_MAX)
      const left = unhanded(journal.read(), id)
      const context = [
        notesNote(now.notes),
        directivesNote(directivesOf(settingsFor(now.root, home))),
        left.length ? leftoverNote(now.file, left) : ''
      ]

      return { context: context.filter(Boolean).join('\n') }
    }

    const stop = () => {
      const state = sync({ tool: UNTRACKED, what: '' })
      const { block, why } = decide(state, now, live(state.agents, Date.now()))

      if (!block && state.blocks) store.save(id, { ...state, blocks: 0 })
      if (!block) return {}

      if (state.blocks >= MAX_BLOCKS) {
        const notice = 'heeded: stopped without a handoff after ' +
          `${MAX_BLOCKS} refusals (${why}). ${now.file || 'It'} is out of date.`
        return { notice }
      }

      store.save(id, { ...state, blocks: state.blocks + 1 })

      return { refusal: reasonFor(why, now.file, state.pending) }
    }

    // a subagent's moves arrive under this session's id, so all that is
    // kept is whether it is still out, and where its worktree started
    const agents = (change) => {
      const state = store.load(id) || open(now)
      const sides = { ...state.sides }
      if (now.side) sides[now.side.root] = now.side.tree

      store.save(id, { ...state, sides, agents: change({ ...state.agents }) })

      return {}
    }

    const forget = (out) => {
      delete out[agent]

      return out
    }

    const HANDLERS = {
      start,
      prompt: () => sync({ tool: OUTSIDE, what: '' }) && {},
      tool: () => sync({ ...toMove(tool, input, now.root), agent }) && {},
      'agent-start': () => agents((out) => ({ ...out, [agent]: Date.now() })),
      'agent-stop': () => agents(forget),
      stop
    }

    return HANDLERS[name] ? HANDLERS[name]() : {}
  }

  return { handle }
}
