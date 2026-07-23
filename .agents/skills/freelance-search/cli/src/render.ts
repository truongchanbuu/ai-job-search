import type { OutputFormat, SearchOutput } from "./models.js"

export function render(value: unknown, format: OutputFormat): string {
  if (format === "json") return JSON.stringify(value, null, 2)
  if (format === "plain") return renderPlain(value)
  return renderTable(value)
}

function renderPlain(value: unknown): string {
  if (typeof value === "string") return value
  if (isSearchOutput(value)) {
    return value.opportunities.map((o) => `${o.title}\n  ${o.platform} | ${o.budget ?? "budget unknown"} | ${o.fit?.recommendation ?? "review"}\n  ${o.url ?? "no url"}`).join("\n\n")
  }
  return JSON.stringify(value, null, 2)
}

function renderTable(value: unknown): string {
  if (isSearchOutput(value)) {
    const header = "TITLE".padEnd(38) + " PLATFORM".padEnd(14) + " REC".padEnd(16) + " BUDGET"
    const rows = value.opportunities.map((o) =>
      `${o.title.slice(0, 38).padEnd(38)} ${String(o.platform).slice(0, 13).padEnd(13)} ${(o.fit?.recommendation ?? "review").padEnd(15)} ${o.budget ?? "-"}`,
    )
    const statuses = value.sourceStatuses.map((s) => `${s.source}: ${s.status} (${s.count})`).join(" | ")
    return [statuses, header, "-".repeat(header.length), ...rows].join("\n")
  }
  if (Array.isArray(value)) return value.map((item) => JSON.stringify(item)).join("\n")
  return JSON.stringify(value, null, 2)
}

function isSearchOutput(value: unknown): value is SearchOutput {
  return Boolean(value && typeof value === "object" && "opportunities" in value && "sourceStatuses" in value)
}

export function writeOutput(value: unknown, format: OutputFormat): void {
  process.stdout.write(render(value, format) + "\n")
}

export function writeError(error: string, code: string): void {
  process.stderr.write(JSON.stringify({ error, code }) + "\n")
}
