import type {
  CanonicalJobListing,
  PortalSearchRun,
  SourceListing,
} from "./contracts.js"

function unknown(value: string | null, label: string): string {
  return value?.trim() || `Unknown ${label}`
}

function conflictLines(job: CanonicalJobListing): string[] {
  return job.conflicts.map((conflict) => {
    const values = conflict.values
      .map((value) => `${value.sourceId}=${String(value.value)}`)
      .join(" | ")
    return `  Conflict ${conflict.field}: ${values} (${conflict.preferenceReason})`
  })
}

function jobLines(job: CanonicalJobListing): string[] {
  return [
    `${unknown(job.title, "title")} — ${unknown(job.employer, "employer")}`,
    `  Location: ${unknown(job.location, "location")}`,
    `  Verification: ${job.verificationStatus}`,
    `  Recommendation: ${job.fitEvidence?.recommendation ?? "none"}`,
    `  Experience: ${job.fitEvidence?.experienceDecision ?? "unknown"}`,
    `  Existing state: ${job.existingApplicationStatus ?? job.existingSeenState ?? "new"}`,
    ...job.sourceListings.map(
      (listing) => `  ${listing.sourceId}: ${listing.canonicalUrl}`,
    ),
    ...conflictLines(job),
  ]
}

function leadLines(lead: SourceListing): string[] {
  return [
    `${unknown(lead.title, "title")} — ${unknown(lead.employer, "employer")}`,
    `  ${unknown(lead.location, "location")} · ${lead.verification.status}`,
    `  ${lead.canonicalUrl}`,
  ]
}

export function renderRun(
  run: PortalSearchRun,
  format: "json" | "table" | "plain" | string,
): string {
  if (format === "json") return JSON.stringify(run, null, 2)
  const lines: string[] = ["Portal coverage"]
  for (const row of run.coverage.sourceRows) {
    lines.push(
      `${String(row.source).padEnd(14)} ${row.status.padEnd(15)} raw=${row.rawDiscoveries} verified=${row.verifiedActive} duplicate=${row.duplicates} expired=${row.expiredOrRemoved} inaccessible=${row.inaccessibleLeads} actionable=${row.actionable}`,
      `  applied: ${row.appliedConstraints.length ? row.appliedConstraints.join(", ") : "none"}`,
      `  unapplied: ${row.unsupportedConstraints.length ? row.unsupportedConstraints.join(", ") : "none"}`,
    )
    if (row.reason) lines.push(`  reason: ${row.reason}`)
    if (row.manualReviewUrl) lines.push(`  review: ${row.manualReviewUrl}`)
  }
  lines.push(
    "",
    `Totals: raw=${run.coverage.rawDiscoveries} verified=${run.coverage.verifiedActive} duplicate=${run.coverage.duplicates} expired=${run.coverage.expiredOrRemoved} inaccessible=${run.coverage.inaccessibleLeads} actionable=${run.coverage.actionableMatches}`,
    "",
    "Actionable shortlist",
  )
  if (run.jobs.length === 0) lines.push("None")
  for (const job of run.jobs) lines.push(...jobLines(job), "")
  lines.push("Unverified or non-actionable leads (no fit score)")
  if (run.unverifiedLeads.length === 0) lines.push("None")
  for (const lead of run.unverifiedLeads) lines.push(...leadLines(lead), "")
  if (run.warnings.length) {
    lines.push("Warnings", ...run.warnings.map((warning) => `- ${warning}`))
  }
  return lines.join("\n").trimEnd()
}
