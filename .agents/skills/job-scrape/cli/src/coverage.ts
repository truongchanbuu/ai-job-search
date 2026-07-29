import type {
  PortalRunResult,
  PortalRunStatus,
  SourceCoverageSummary,
} from "./contracts.js"

const STATUSES: PortalRunStatus[] = [
  "searched",
  "no_matches",
  "manual_required",
  "restricted",
  "unavailable",
  "failed",
]

export function deriveCoverage(
  results: PortalRunResult[],
): SourceCoverageSummary {
  const statusCounts = Object.fromEntries(
    STATUSES.map((status) => [status, 0]),
  ) as Record<PortalRunStatus, number>
  for (const result of results) statusCounts[result.status] += 1
  const sum = (pick: (result: PortalRunResult) => number): number =>
    results.reduce((total, result) => total + pick(result), 0)
  return {
    totalSources: results.length,
    statusCounts,
    rawDiscoveries: sum((result) => result.rawDiscoveryCount),
    verifiedActive: sum((result) => result.verifiedActiveCount),
    duplicates: sum((result) => result.duplicateCount),
    expiredOrRemoved: sum((result) => result.expiredOrRemovedCount),
    inaccessibleLeads: sum((result) => result.inaccessibleLeadCount),
    actionableMatches: sum((result) => result.actionableCount),
    sourceRows: results.map((result) => ({
      source: result.sourceId,
      status: result.status,
      appliedConstraints: Object.keys(result.appliedConstraints),
      unsupportedConstraints: result.unsupportedConstraints,
      rawDiscoveries: result.rawDiscoveryCount,
      verifiedActive: result.verifiedActiveCount,
      duplicates: result.duplicateCount,
      expiredOrRemoved: result.expiredOrRemovedCount,
      inaccessibleLeads: result.inaccessibleLeadCount,
      actionable: result.actionableCount,
      reason: result.message ?? result.reasonCode,
      manualReviewUrl: result.manualReviewUrl,
    })),
  }
}
