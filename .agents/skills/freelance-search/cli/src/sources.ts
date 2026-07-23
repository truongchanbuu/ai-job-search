import type { FreelanceSource, SourceStatusSummary } from "./models.js"

export function defaultSources(): FreelanceSource[] {
  return [
    { source_id: "upwork", source_type: "upwork", display_name: "Upwork", enabled: true, access_mode: "manual", status: "needs_setup", notes: "Readiness determines actionability." },
    { source_id: "fiverr", source_type: "fiverr", display_name: "Fiverr", enabled: true, access_mode: "manual", status: "needs_setup", notes: "Seller profile and gig readiness determine actionability." },
    { source_id: "google", source_type: "google", display_name: "Google", enabled: true, access_mode: "public", status: "available", notes: "Public-page discovery leads require verification." },
    { source_id: "user_supplied", source_type: "user_supplied", display_name: "User Supplied", enabled: true, access_mode: "user_supplied", status: "available" },
  ]
}

export function filterSources(sources: FreelanceSource[], allowlist: string[] = []): FreelanceSource[] {
  const wanted = new Set(allowlist.map((source) => source.toLowerCase()))
  return sources.filter((source) => source.enabled && (wanted.size === 0 || wanted.has(source.source_id.toLowerCase()) || wanted.has(source.source_type)))
}

export function summarizeSource(source: string, status: SourceStatusSummary["status"], count: number, message?: string): SourceStatusSummary {
  return { source, status, count, message }
}
