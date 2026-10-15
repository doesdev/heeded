export const REWORK = 'rework.jsonl'

export const OUTCOMES = ['kept', 'reworked', 'discarded']

export const CLASSES = [
  'none',
  'wrong-context',
  'wrong-assumption',
  'missing-test',
  'style-mismatch',
  'spec-gap'
]

// both sets are closed on purpose: a free-text class cannot be counted
export const checkRework = ({ task, outcome, failureClass }) => {
  const checks = [
    ['task', typeof task === 'string' && task.trim().length > 0],
    ['outcome', OUTCOMES.includes(outcome)],
    ['class', CLASSES.includes(failureClass)],
    ['class-for-outcome', (outcome === 'kept') === (failureClass === 'none')]
  ]

  return checks.filter(([, ok]) => !ok).map(([name]) => name)
}
