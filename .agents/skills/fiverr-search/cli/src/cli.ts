#!/usr/bin/env bun

interface Flags {
  _: string[]
  [key: string]: string | boolean | string[]
}

interface LinkLead {
  source: "fiverr"
  resultType: "manual_discovery_query" | "manual_lead"
  requiresVerification: true
  query: string
  url: string
  budget?: string
}

function parseFlags(argv: string[]): Flags {
  const flags: Flags = { _: [] }
  const alias: Record<string, string> = { q: "query" }
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i]
    if (arg.startsWith("-")) {
      const key = alias[arg.replace(/^-+/, "")] ?? arg.replace(/^-+/, "")
      const next = argv[i + 1]
      if (next === undefined || next.startsWith("-")) flags[key] = true
      else { flags[key] = next; i++ }
    } else flags._.push(arg)
  }
  return flags
}

export function fiverrUrl(query: string, service?: string): string {
  const q = [query, service].filter(Boolean).join(" ")
  return `https://www.fiverr.com/search/gigs?query=${encodeURIComponent(q)}`
}

export function buildLead(query = "freelance developer", service?: string, budget?: string): LinkLead {
  return {
    source: "fiverr",
    resultType: "manual_discovery_query",
    requiresVerification: true,
    query: [query, service].filter(Boolean).join(" "),
    url: fiverrUrl(query, service),
    budget,
  }
}

function render(value: unknown, fmt: string): string {
  if (fmt === "table") {
    const lead = value as LinkLead
    return `QUERY${" ".repeat(35)} URL\n${lead.query.padEnd(40)} ${lead.url}`
  }
  if (fmt === "plain") {
    const lead = value as LinkLead
    return `${lead.query}\n${lead.url}`
  }
  return JSON.stringify(value, null, 2)
}

async function main(): Promise<number> {
  const flags = parseFlags(process.argv.slice(2))
  const cmd = flags._[0]
  if (!cmd || flags.help) {
    process.stdout.write("fiverr-search queries|lead -q <query> [--service text] [--budget text]\n")
    return cmd ? 0 : 1
  }
  const lead = buildLead(flags.query as string | undefined, flags.service as string | undefined, flags.budget as string | undefined)
  process.stdout.write(render({ ...lead, resultType: cmd === "lead" ? "manual_lead" : lead.resultType }, flags.format as string || "json") + "\n")
  return 0
}

if (import.meta.main) {
  main().then((code) => process.exit(code))
}
