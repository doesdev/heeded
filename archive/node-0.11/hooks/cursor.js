// Written from Cursor's published docs on 2026-10-09 and never run inside
// Cursor. Decision 0014 lists what was confirmed and what was not.

// lower-cased, so Claude-style names from an imported plugin land here too
const EVENTS = {
  sessionstart: 'start',
  pretooluse: 'pre',
  beforeshellexecution: 'pre',
  beforesubmitprompt: 'prompt',
  userpromptsubmit: 'prompt',
  posttooluse: 'tool',
  afterfileedit: 'tool',
  aftershellexecution: 'tool',
  stop: 'stop'
}

const TOOLS = { Shell: 'Bash', Write: 'Write', Edit: 'Edit' }

// the two older per-tool events carry no tool name
const IMPLIED = {
  afterfileedit: (input) => ['Write', { file_path: input.file_path }],
  aftershellexecution: () => ['Bash', {}],
  beforeshellexecution: (input) => ['Bash', { command: input.command }]
}

export const parse = (input, env) => {
  const event = String(input.hook_event_name || '').toLowerCase()
  const roots = input.workspace_roots || []
  const [tool, toolInput] = IMPLIED[event]
    ? IMPLIED[event](input)
    : [TOOLS[input.tool_name] || input.tool_name, input.tool_input]

  return {
    name: EVENTS[event],
    id: input.conversation_id || input.session_id,
    dir: env.CURSOR_PROJECT_DIR || env.CLAUDE_PROJECT_DIR || roots[0] ||
      input.cwd || process.cwd(),
    tool,
    input: toolInput
  }
}

// everything is JSON on stdout and exit 0; a refusal is a follow-up message
export const emit = (event, { context, refusal, notice, ask, deny }) => {
  const permission = deny ? 'deny' : ask ? 'ask' : 'allow'
  const said = deny || ask
  const OUT = {
    pre: () => said
      ? { permission, user_message: said, agent_message: said }
      : { permission },
    start: () => context ? { additional_context: context } : {},
    prompt: () => ({ continue: true }),
    tool: () => ({}),
    stop: () => refusal ? { followup_message: refusal } : {}
  }
  const out = OUT[event.name] ? OUT[event.name]() : {}

  return { stdout: JSON.stringify(notice ? { ...out, user_message: notice } : out) }
}
