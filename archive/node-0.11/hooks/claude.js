const EVENTS = {
  SessionStart: 'start',
  PreToolUse: 'pre',
  UserPromptSubmit: 'prompt',
  PostToolUse: 'tool',
  SubagentStart: 'agent-start',
  SubagentStop: 'agent-stop',
  Stop: 'stop'
}

export const parse = (input, env) => {
  return {
    name: EVENTS[input.hook_event_name],
    id: input.session_id,
    dir: env.CLAUDE_PROJECT_DIR || input.cwd || process.cwd(),
    cwd: input.cwd,
    agent: input.agent_id,
    tool: input.tool_name,
    input: input.tool_input
  }
}

const permit = (permissionDecision, permissionDecisionReason) => {
  const hookSpecificOutput = {
    hookEventName: 'PreToolUse',
    permissionDecision,
    permissionDecisionReason
  }

  return { stdout: JSON.stringify({ hookSpecificOutput }) }
}

// context is plain stdout; a refusal is stderr and exit 2
export const emit = (event, { context, refusal, notice, ask, deny }) => {
  if (deny) return permit('deny', deny)
  if (ask) return permit('ask', ask)
  if (refusal) return { stderr: refusal, code: 2 }
  if (notice) return { stdout: JSON.stringify({ systemMessage: notice }) }

  return { stdout: context }
}
