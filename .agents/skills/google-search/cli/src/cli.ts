#!/usr/bin/env bun

import { runQueries, type QueryOpts } from "./commands/queries.js"
import { runSearch, type SearchOpts } from "./commands/search.js"
import { parseRepeatable } from "./helpers.js"

interface Flags {
  _: string[]
  [k: string]: string | boolean | string[]
}

function parseFlags(argv: string[]): Flags {
  const flags: Flags = { _: [] }
  const alias: Record<string, string> = { q: "query", l: "location", n: "limit" }
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (a.startsWith("--") || a.startsWith("-")) {
      const key = alias[a.replace(/^-+/, "")] ?? a.replace(/^-+/, "")
      const next = argv[i + 1]
      if (next === undefined || next.startsWith("-")) {
        flags[key] = true
      } else if (flags[key] !== undefined) {
        flags[key] = Array.isArray(flags[key]) ? [...flags[key], next] : [flags[key] as string, next]
        i++
      } else {
        flags[key] = next
        i++
      }
    } else {
      flags._.push(a)
    }
  }
  return flags
}

const HELP = `google-search-cli - supplemental Google job discovery

Google support discovers public posting pages. It does not implement or scrape
the Google Jobs UI, and returned leads require original-posting verification.

USAGE
  bun run src/cli.ts queries [flags]
  bun run src/cli.ts search [flags]

FLAGS
  --query, -q <text>      Role, skill, or keyword query.
  --location, -l <text>   Location text, e.g. "Ho Chi Minh City" or "Remote Vietnam".
  --jobage <days>         Recency in days. API search maps this to dateRestrict=dN.
  --site <domain>         Repeatable site filter, e.g. --site itviec.com --site greenhouse.io.
  --limit, -n <n>         Result cap. API search maxes at 10 per call.
  --format <fmt>          json (default) | table | plain.

ENV FOR LIVE SEARCH
  GOOGLE_API_KEY and GOOGLE_CSE_ID are required for the search command.

EXAMPLES
  bun run src/cli.ts queries -q "Backend Engineer Node.js NestJS" -l "Ho Chi Minh City" --jobage 30 --format table
  bun run src/cli.ts search -q "Backend Engineer Node.js NestJS" -l "Ho Chi Minh City" --jobage 30 --limit 10 --format json
`

function numberFlag(flags: Flags, name: string, fallback: number | undefined): number | undefined {
  if (flags[name] === undefined) return fallback
  const parsed = parseInt(flags[name] as string, 10)
  if (Number.isNaN(parsed)) {
    throw new Error(`--${name} must be a number, got "${flags[name]}"`)
  }
  return parsed
}

async function main(): Promise<number> {
  const flags = parseFlags(process.argv.slice(2))
  const cmd = flags._[0]
  if (!cmd || flags.help || flags.h) {
    process.stdout.write(HELP)
    return cmd ? 0 : 1
  }

  try {
    const fmt = typeof flags.format === "string" && ["json", "table", "plain"].includes(flags.format)
      ? flags.format
      : "json"
    const base = {
      query: typeof flags.query === "string" ? flags.query : undefined,
      location: typeof flags.location === "string" ? flags.location : undefined,
      jobage: numberFlag(flags, "jobage", undefined),
      sites: parseRepeatable(flags.site),
      format: fmt as "json" | "table" | "plain",
    }

    if (cmd === "queries") return runQueries(base satisfies QueryOpts)
    if (cmd === "search") {
      return runSearch({
        ...base,
        limit: numberFlag(flags, "limit", 10),
      } satisfies SearchOpts)
    }
  } catch (e) {
    process.stderr.write(JSON.stringify({ error: e instanceof Error ? e.message : String(e), code: "BAD_ARG" }) + "\n")
    return 1
  }

  process.stderr.write(JSON.stringify({ error: `Unknown command "${cmd}"`, code: "BAD_CMD" }) + "\n")
  return 1
}

main().then((code) => process.exit(code))
