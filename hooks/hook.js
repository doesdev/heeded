// A session started before 0.12.0 still runs this file. It hands the event
// to the heeded binary and passes the answer back. Delete it once no such
// session is left.
import { spawnSync } from 'node:child_process'
import fs from 'node:fs'

const ran = spawnSync('heeded', ['hook'], { input: fs.readFileSync(0), shell: false })

if (ran.stdout) process.stdout.write(ran.stdout)
if (ran.stderr) process.stderr.write(ran.stderr)
process.exitCode = ran.status === null ? 1 : ran.status
