import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

export const MARKER = '.heeded.json'
export const NOTES = '.heeded'
export const HANDOFF = 'handoff.md'
export const JOURNAL = 'journal.jsonl'
export const JOURNAL_MAX = 500
export const MAX_BLOCKS = 3
export const SETTINGS = 'config.json'
export const DIRECTIVE_MAX = 4000
export const HOME_DIR = process.env.HEEDED_HOME ||
  path.join(os.homedir(), '.heeded')
export const STATE_DIR = process.env.CLAUDE_PLUGIN_DATA ||
  path.join(HOME_DIR, 'state')
export const TEMPLATES = fileURLToPath(new URL('templates', import.meta.url))
