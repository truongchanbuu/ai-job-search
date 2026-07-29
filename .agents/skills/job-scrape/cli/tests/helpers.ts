import { join } from "node:path"

const CLI_PATH = join(import.meta.dir, "../src/cli.ts")

export interface CLIResult {
  stdout: string
  stderr: string
  exitCode: number
}

export async function runCLI(
  args: string[],
  env: Record<string, string> = {},
): Promise<CLIResult> {
  const processHandle = Bun.spawn(["bun", "run", CLI_PATH, ...args], {
    stdout: "pipe",
    stderr: "pipe",
    env: { ...process.env, ...env },
  })
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(processHandle.stdout).text(),
    new Response(processHandle.stderr).text(),
    processHandle.exited,
  ])
  return { stdout: stdout.trim(), stderr: stderr.trim(), exitCode }
}

export function captureWrites(
  stream: NodeJS.WriteStream,
): { read: () => string; restore: () => void } {
  const original = stream.write
  let output = ""
  stream.write = ((chunk: string | Uint8Array) => {
    output += chunk.toString()
    return true
  }) as typeof stream.write
  return {
    read: () => output,
    restore: () => {
      stream.write = original
    },
  }
}

export async function fixture(path: string): Promise<string> {
  return Bun.file(join(import.meta.dir, "fixtures", path)).text()
}
