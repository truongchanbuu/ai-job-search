import { describe, expect, test } from "bun:test"
import { deriveCoverage } from "../src/coverage.js"
import type { PortalRunResult } from "../src/contracts.js"

function result(
  sourceId: string,
  status: PortalRunResult["status"],
  counts: Partial<PortalRunResult> = {},
): PortalRunResult {
  const now = "2026-07-28T00:00:00Z"
  return {
    sourceId,
    status,
    startedAt: now,
    completedAt: now,
    appliedConstraints: { query: "Java" },
    unsupportedConstraints: ["experience"],
    manualReviewUrl:
      status === "manual_required" ? `https://${sourceId}.example/search` : null,
    rawDiscoveryCount: 0,
    verifiedActiveCount: 0,
    duplicateCount: 0,
    expiredOrRemovedCount: 0,
    inaccessibleLeadCount: 0,
    actionableCount: 0,
    listings: [],
    reasonCode: status === "failed" ? "SEARCH_FAILED" : null,
    message: status === "failed" ? "parser failed" : null,
    warnings: [],
    ...counts,
  }
}

describe("coverage aggregation", () => {
  test("derives one row per source and internally consistent totals", () => {
    const coverage = deriveCoverage([
      result("topcv", "searched", {
        rawDiscoveryCount: 4,
        verifiedActiveCount: 2,
        duplicateCount: 1,
        expiredOrRemovedCount: 1,
        inaccessibleLeadCount: 1,
        actionableCount: 1,
      }),
      result("itviec", "no_matches"),
      result("careerviet", "manual_required"),
      result("vieclam24h", "restricted"),
      result("vietnamworks", "failed"),
    ])
    expect(coverage.totalSources).toBe(5)
    expect(coverage.sourceRows).toHaveLength(5)
    expect(coverage.statusCounts).toMatchObject({
      searched: 1,
      no_matches: 1,
      manual_required: 1,
      restricted: 1,
      failed: 1,
    })
    expect(coverage).toMatchObject({
      rawDiscoveries: 4,
      verifiedActive: 2,
      duplicates: 1,
      expiredOrRemoved: 1,
      inaccessibleLeads: 1,
      actionableMatches: 1,
    })
    expect(coverage.sourceRows[0].appliedConstraints).toEqual(["query"])
    expect(coverage.sourceRows[0].unsupportedConstraints).toEqual(["experience"])
  })
})
