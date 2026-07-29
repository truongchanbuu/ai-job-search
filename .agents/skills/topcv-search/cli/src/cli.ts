#!/usr/bin/env bun

import { runSearch } from "./commands/search.js"
import { writeError } from "./helpers.js"

type Flags = { _: string[]; [key: string]: string | string[] }

function parse(argv: string[]): Flags {
  const out: Flags = { _: [] }
  const repeat = new Set(["location", "seniority", "work-mode"])
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index]
    if (!token.startsWith("-")) {
      out._.push(token)
      continue
    }
    const raw = token.replace(/^-+/, "")
    const key = ({ q: "query", l: "location", n: "limit" } as Record<string, string>)[raw] ?? raw
    const value = argv[index + 1]
    if (!value || value.startsWith("-")) throw new Error(`--${key} requires a value`)
    index += 1
    if (repeat.has(key)) {
      out[key] = [...(Array.isArray(out[key]) ? out[key] : []), value]
    } else out[key] = value
  }
  return out
}

function list(value: string | string[] | undefined): string[] {
  return Array.isArray(value) ? value : typeof value === "string" ? [value] : []
}

function numeric(value: string | string[] | undefined, name: string): number | null {
  if (value === undefined) return null
  const number = Number(value)
  if (!Number.isFinite(number) || number < 0) throw new Error(`--${name} must be a non-negative number`)
  return number
}

export async function main(argv = process.argv.slice(2)): Promise<number> {
  try {
    const options = parse(argv)
    if (options._[0] === "search") {
      return runSearch({
        query: typeof options.query === "string" ? options.query : undefined,
        locations: list(options.location),
        maxExperience: numeric(options["max-experience"], "max-experience"),
        seniority: list(options.seniority),
        workModes: list(options["work-mode"]),
        jobage: numeric(options.jobage, "jobage"),
        limit: numeric(options.limit, "limit") ?? undefined,
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
