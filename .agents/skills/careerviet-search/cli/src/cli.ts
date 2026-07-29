#!/usr/bin/env bun

import { runSearch } from "./commands/search.js"
import { writeError } from "./helpers.js"

function flags(argv: string[]): { _: string[]; [key: string]: string | string[] } {
  const out: { _: string[]; [key: string]: string | string[] } = { _: [] }
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index]
    if (!token.startsWith("-")) {
      out._.push(token)
      continue
    }
    const key = ({ q: "query", l: "location", n: "limit" } as Record<string, string>)[
      token.replace(/^-+/, "")
    ] ?? token.replace(/^-+/, "")
    const value = argv[index + 1]
    if (!value || value.startsWith("-")) throw new Error(`--${key} requires a value`)
    index += 1
    if (key === "location") {
      out[key] = [...(Array.isArray(out[key]) ? out[key] : []), value]
    } else out[key] = value
  }
  return out
}

export async function main(argv = process.argv.slice(2)): Promise<number> {
  try {
    const parsed = flags(argv)
    const command = parsed._[0]
    if (command === "search") {
      const maxExperience =
        typeof parsed["max-experience"] === "string"
          ? Number(parsed["max-experience"])
          : null
      if (maxExperience != null && (!Number.isFinite(maxExperience) || maxExperience < 0)) {
        throw new Error("--max-experience must be a non-negative number")
      }
      return runSearch({
        query: typeof parsed.query === "string" ? parsed.query : undefined,
        locations: Array.isArray(parsed.location)
          ? parsed.location
          : typeof parsed.location === "string"
            ? [parsed.location]
            : [],
        maxExperience,
        format:
          parsed.format === "table" || parsed.format === "plain"
            ? parsed.format
            : "json",
      })
    }
    if (command === "detail") {
      const { runDetail } = await import("./commands/detail.js")
      return runDetail({
        input: parsed._[1] ?? "",
        format: parsed.format === "plain" ? "plain" : "json",
      })
    }
    writeError(`Unknown command "${command ?? ""}"`, "BAD_CMD")
    return 1
  } catch (error) {
    writeError(error instanceof Error ? error.message : String(error), "BAD_ARG")
    return 1
  }
}

if (import.meta.main) main().then((code) => (process.exitCode = code))
