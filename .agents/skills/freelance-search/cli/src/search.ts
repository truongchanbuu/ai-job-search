import type { FreelanceOpportunity, SearchOutput } from "./models.js"
import { dedupeOpportunities } from "./dedupe.js"
import { googleDiscovery } from "./google-source.js"
import { normalizeOpportunity } from "./normalize.js"
import { loadProfile } from "./profile.js"
import { loadReadiness } from "./readiness-store.js"
import { rankOpportunities } from "./rank.js"
import { defaultSources, filterSources, summarizeSource } from "./sources.js"
import { readJsonArray } from "./storage.js"
import { resolveWorkspacePath } from "./paths.js"

export interface SearchOptions {
  profile?: string
  sources?: string[]
  query?: string
  userSupplied?: string
  format?: string
}

export function runFreelanceSearch(opts: SearchOptions): SearchOutput {
  const profile = loadProfile(opts.profile)
  const sources = filterSources(defaultSources(), opts.sources)
  const readiness = loadReadiness()
  const warnings: string[] = []
  const all: FreelanceOpportunity[] = []
  const statuses = []

  for (const source of sources) {
    if (source.source_id === "google") {
      const results = googleDiscovery(profile, opts.query)
      all.push(...results)
      statuses.push(summarizeSource("google", "available", results.length))
    } else if (source.source_id === "user_supplied" && opts.userSupplied) {
      const supplied = readJsonArray<FreelanceOpportunity>(resolveWorkspacePath(opts.userSupplied))
      all.push(...supplied)
      statuses.push(summarizeSource("user_supplied", "available", supplied.length))
    } else if (source.source_id === "upwork") {
      const result = normalizeOpportunity({
        title: `${opts.query || profile.target_services[0] || "Freelance"} on Upwork`,
        clientOrSource: "Upwork discovery",
        platform: "upwork",
        url: `https://www.upwork.com/nx/search/jobs/?q=${encodeURIComponent(opts.query || profile.target_services[0] || "freelance")}`,
        description: "Manual Upwork discovery lead. Verify posting and Connects before action.",
        requiresVerification: true,
        source: "upwork",
      }, "upwork")
      all.push(result)
      statuses.push(summarizeSource("upwork", "needs_setup", 1, "Check account readiness and Connects before proposals."))
    } else if (source.source_id === "fiverr") {
      const result = normalizeOpportunity({
        title: `${opts.query || profile.target_services[0] || "Freelance"} on Fiverr`,
        clientOrSource: "Fiverr marketplace discovery",
        platform: "fiverr",
        url: `https://www.fiverr.com/search/gigs?query=${encodeURIComponent(opts.query || profile.target_services[0] || "freelance")}`,
        description: "Manual Fiverr discovery lead. Verify marketplace context and seller/gig readiness before action.",
        requiresVerification: true,
        source: "fiverr",
      }, "fiverr")
      all.push(result)
      statuses.push(summarizeSource("fiverr", "needs_setup", 1, "Check seller profile and gig readiness before action."))
    } else {
      statuses.push(summarizeSource(source.source_id, source.status, 0, source.notes))
    }
  }

  if ((profile.target_services ?? []).length === 0) warnings.push("Freelance profile has no target_services; ranking confidence is limited.")
  const opportunities = rankOpportunities(dedupeOpportunities(all), profile, readiness)
  return { profileId: profile.profile_id, sourceStatuses: statuses, opportunities, warnings }
}
