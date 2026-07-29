import type { PortalSource } from "./contracts.js"

export interface PortalProcessResult {
  sourceId: string
  exitCode: number
  stdout: string
  stderr: string
  timedOut: boolean
}

export async function runPortalProcess(
  source: PortalSource,
  args: string[],
): Promise<PortalProcessResult> {
  const processHandle = Bun.spawn(["bun", "run", source.cliPath, ...args], {
    stdout: "pipe",
    stderr: "pipe",
    env: { ...process.env, NO_COLOR: "1" },
  })
  let timedOut = false
  const timer = setTimeout(() => {
    timedOut = true
    processHandle.kill()
  }, source.requestTimeoutMs)
  try {
    const [stdout, stderr, exitCode] = await Promise.all([
      new Response(processHandle.stdout).text(),
      new Response(processHandle.stderr).text(),
      processHandle.exited,
    ])
    return {
      sourceId: source.sourceId,
      exitCode,
      stdout: stdout.trim(),
      stderr: stderr.trim(),
      timedOut,
    }
  } finally {
    clearTimeout(timer)
  }
}

export function parseProcessJson(result: PortalProcessResult): unknown {
  const body = result.stdout || result.stderr
  if (!body) return null
  try {
    return JSON.parse(body)
  } catch {
    return null
  }
}
