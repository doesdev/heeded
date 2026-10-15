const STATUSES = {
  accepted: true,
  rejected: true,
  amended: true,
  superseded: true
}

const field = (text, name) => {
  const found = text.match(new RegExp(`^${name}: (.+)$`, 'm'))

  return found ? found[1].trim() : ''
}

// a record is ratified by a model id, whoever proposed it
export const checkRecord = (text) => {
  const proposer = field(text, 'Proposed by')
  const checks = [
    ['date', /^\d{4}-\d\d-\d\dT\d\d:\d\dZ$/.test(field(text, 'Date'))],
    ['status', STATUSES[field(text, 'Status').split(' ')[0]]],
    ['proposed-by', /^(human|model)\b/.test(proposer)],
    ['ratified-by', /^(?!human)[a-z][a-z0-9.-]+$/.test(field(text, 'Ratified by'))],
    ['assessment', !/^human\b/.test(proposer) || /^## Assessment$/m.test(text)]
  ]

  return checks.filter(([, ok]) => !ok).map(([name]) => name)
}
