#!/usr/bin/env bun

import type { SearchCriteria } from "./contracts.js"

export interface ParsedFlags {
  _: string[]
  [key: string]: string | string[] | boolean
}

const REPEATABLE = new Set([
  "location",
  "seniority",
  "work-mode",
  "employment-type",
  "source",
  "skill",
  "exclude",
])

export function parseFlags(argv: string[]): ParsedFlags {
  const flags: ParsedFlags = { _: [] }
  const aliases: Record<string, string> = { q: "query", l: "location", n: "limit" }
  for (let index = 0; index < argv.length; index += 1) {
    const token = argv[index]
    if (!token.startsWith("-")) {
      flags._.push(token)
      continue
    }
    const key = aliases[token.replace(/^-+/, "")] ?? token.replace(/^-+/, "")
    const next = argv[index + 1]
    const value: string | boolean =
      next === undefined || next.startsWith("-") ? true : next
    if (value !== true) index += 1
    if (REPEATABLE.has(key)) {
      const existing = flags[key]
      flags[key] = [
        ...(Array.isArray(existing) ? existing : typeof existing === "string" ? [existing] : []),
        String(value),
      ]
    } else {
      flags[key] = value
    }
  }
  return flags
}

function strings(value: string | string[] | boolean | undefined): string[] {
  if (Array.isArray(value)) return value
  if (typeof value === "string") return [value]
  return []
}

function numberFlag(
  flags: ParsedFlags,
  key: string,
  fallback: number | null,
): number | null {
  const raw = flags[key]
  if (raw === undefined) return fallback
  if (typeof raw !== "string" || raw.trim() === "") {
    throw new Error(`--${key} requires a number`)
  }
  const value = Number(raw)
  if (!Number.isFinite(value) || value < 0) {
    throw new Error(`--${key} must be a non-negative number`)
  }
  return value
}

export function criteriaFromFlags(flags: ParsedFlags): SearchCriteria {
  const query = typeof flags.query === "string" ? flags.query.trim() : ""
  const skills = strings(flags.skill)
  return {
    queryTerms: query ? [query] : [],
    skills,
    locations: strings(flags.location),
    maxExperienceYears: numberFlag(flags, "max-experience", null),
    seniority: strings(flags.seniority),
    postedWithinDays: numberFlag(flags, "jobage", null),
    workModes: strings(flags["work-mode"]),
    employmentTypes: strings(flags["employment-type"]),
    exclusions: strings(flags.exclude),
    languageTerms: [...(query ? [query] : []), ...skills],
    limitPerSource: numberFlag(flags, "limit", 10) ?? 10,
  }
}

const HELP = `job-scrape — search configured job portals

USAGE
  bun run src/cli.ts search --query <text> [flags]

FLAGS
  --location, -l <text>       Repeatable location
  --skill <text>              Repeatable technology/skill
  --max-experience <years>    Maximum required experience
  --seniority <level>         Repeatable seniority
  --jobage <days>             Posting recency
  --work-mode <mode>          remote | hybrid | onsite
  --employment-type <value>   Repeatable employment type
  --exclude <text>            Repeatable hard exclusion
  --source <id>               Repeatable source allowlist
  --limit, -n <number>        Per-source discovery cap
  --profile-path <path>       Optional profile path
  --seen-path <path>          Seen state path
  --tracker-path <path>       Application tracker path
  --save-session              Persist a private session report
  --format <json|table|plain> Output format
`

export async function main(argv = process.argv.slice(2)): Promise<number> {
  const flags = parseFlags(argv)
  const command = flags._[0]
  if (!command || flags.help || flags.h) {
    process.stdout.write(HELP)
    return command ? 0 : 1
  }
  if (command !== "search") {
    process.stderr.write(
      JSON.stringify({ error: `Unknown command "${command}"`, code: "BAD_CMD" }) + "\n",
    )
    return 1
  }
  try {
    const criteria = criteriaFromFlags(flags)
    const { runSearch } = await import("./commands/search.js")
    return runSearch({ flags, criteria })
  } catch (error) {
    process.stderr.write(
      JSON.stringify({
        error: error instanceof Error ? error.message : String(error),
        code: "BAD_ARG",
      }) + "\n",
    )
    return 1
  }
}

if (import.meta.main) {
  main().then((code) => {
    process.exitCode = code
  })
}
