#!/usr/bin/env bun

import { runEvaluate } from "./commands/evaluate.js"
import { runProposal } from "./commands/proposal.js"
import { runReadiness } from "./commands/readiness.js"
import { runSave } from "./commands/save.js"
import { runSearch } from "./commands/search.js"
import { writeError } from "./render.js"
import type { Flags, OutputFormat } from "./models.js"

function parseFlags(argv: string[]): Flags {
  const flags: Flags = { _: [] }
  const alias: Record<string, string> = { q: "query", p: "profile", f: "format" }
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i]
    if (arg.startsWith("--") || arg.startsWith("-")) {
      const key = alias[arg.replace(/^-+/, "")] ?? arg.replace(/^-+/, "")
      const next = argv[i + 1]
      if (next === undefined || next.startsWith("-")) flags[key] = true
      else if (flags[key] !== undefined) {
        flags[key] = Array.isArray(flags[key]) ? [...(flags[key] as string[]), next] : [flags[key] as string, next]
        i++
      } else {
        flags[key] = next
        i++
      }
    } else flags._.push(arg)
  }
  return flags
}

function repeatable(value: string | boolean | string[] | undefined): string[] {
  if (value === undefined || typeof value === "boolean") return []
  return Array.isArray(value) ? value : [value]
}

function format(flags: Flags): OutputFormat {
  return typeof flags.format === "string" && ["json", "table", "plain"].includes(flags.format) ? flags.format as OutputFormat : "json"
}

const HELP = `freelance-search-cli

USAGE
  freelance-search search --profile <path> [--source upwork] [--query text]
  freelance-search readiness --platform upwork --account-status limited --known-blocker "Connects unknown"
  freelance-search evaluate --opportunity <path> --profile <path>
  freelance-search proposal --opportunity <path> --profile <path>
  freelance-search save --opportunity <path> --status proposal_drafted
`

async function main(): Promise<number> {
  const flags = parseFlags(process.argv.slice(2))
  const cmd = flags._[0]
  if (!cmd || flags.help || flags.h) {
    process.stdout.write(HELP)
    return cmd ? 0 : 1
  }
  try {
    const fmt = format(flags)
    if (cmd === "search") return runSearch({ profile: flags.profile as string | undefined, query: flags.query as string | undefined, sources: repeatable(flags.source), userSupplied: flags["user-supplied"] as string | undefined, format: fmt })
    if (cmd === "readiness") return runReadiness({ platform: flags.platform as string | undefined, accountStatus: flags["account-status"] as string | undefined, serviceCategories: repeatable(flags.category), portfolioRefs: repeatable(flags.portfolio), knownBlockers: repeatable(flags["known-blocker"]), verificationStatus: flags["verification-status"] as string | undefined, format: fmt })
    if (cmd === "evaluate") return runEvaluate({ opportunity: flags.opportunity as string | undefined, profile: flags.profile as string | undefined, format: fmt })
    if (cmd === "proposal") return runProposal({ opportunity: flags.opportunity as string | undefined, profile: flags.profile as string | undefined, materialType: flags["material-type"] as any, format: fmt })
    if (cmd === "save") return runSave({ opportunity: flags.opportunity as string | undefined, status: flags.status as any, materialRefs: repeatable(flags.material), notes: flags.notes as string | undefined, format: fmt })
    writeError(`Unknown command "${cmd}"`, "BAD_CMD")
    return 1
  } catch (error) {
    writeError(error instanceof Error ? error.message : String(error), "FAILED")
    return 1
  }
}

if (import.meta.main) {
  main().then((code) => process.exit(code))
}
