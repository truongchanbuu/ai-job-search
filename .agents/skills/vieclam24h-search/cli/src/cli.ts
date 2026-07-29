#!/usr/bin/env bun

import { runSearch } from "./commands/search.js"
import { writeError } from "./helpers.js"

type Flags = { _: string[]; [key: string]: string | string[] }

function parse(argv: string[]): Flags {
  const out: Flags = { _: [] }
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index]
    if (!token.startsWith("-")) out._.push(token)
    else {
      const key = token.replace(/^-+/, "").replace(/^q$/, "query")
      const value = argv[index + 1]
      if (!value || value.startsWith("-")) throw new Error(`--${key} requires a value`)
      out[key] = value
      index += 1
    }
  }
  return out
}

export async function main(argv = process.argv.slice(2)): Promise<number> {
  try {
    const options = parse(argv)
    if (options._[0] === "search") {
      return runSearch({
        query: typeof options.query === "string" ? options.query : undefined,
        format: options.format === "table" || options.format === "plain" ? options.format : "json",
      })
    }
    if (options._[0] === "detail") {
      const { runDetail } = await import("./commands/detail.js")
      return runDetail({ input: options._[1] ?? "", format: options.format === "plain" ? "plain" : "json" })
    }
    writeError(`Unknown command "${options._[0] ?? ""}"`, "BAD_CMD")
    return 1
  } catch (error) {
    writeError(error instanceof Error ? error.message : String(error), "BAD_ARG")
    return 1
  }
}

if (import.meta.main) main().then((code) => (process.exitCode = code))
