import { describe, expect, test } from "bun:test"
import { executeDiscovery, type PortalExecutor } from "../src/commands/search.js"
import { PORTAL_REGISTRY } from "../src/portal-registry.js"
import type { AdapterSearchEnvelope, SearchCriteria } from "../src/contracts.js"

const criteria: SearchCriteria = {
  queryTerms: ["Java backend ReactJS"],
  skills: ["Java"],
  locations: ["Việt Nam"],
  maxExperienceYears: 1,
  seniority: ["fresher"],
  postedWithinDays: 30,
  workModes: [],
  employmentTypes: [],
  exclusions: [],
  languageTerms: ["Java backend ReactJS"],
  limitPerSource: 10,
}

function envelope(source: string, status: AdapterSearchEnvelope["status"]): AdapterSearchEnvelope {
  return {
    contractVersion: "1",
    source,
    accessMode: status === "searched" ? "enabled_public" : "restricted",
    status,
    query: {},
    appliedConstraints: { query: criteria.queryTerms[0] },
    unsupportedConstraints: ["experience"],
    manualReviewUrl: status === "manual_required" ? `https://${source}.example/` : null,
    meta: { count: 0, page: 1, total: 0 },
    results: [],
    warnings: [],
  }
}

describe("five-source discovery", () => {
  test("returns exactly one status per enabled source despite a failure", async () => {
    const executor: PortalExecutor = async (source) => {
      if (source.sourceId === "vietnamworks") throw new Error("timeout")
      return envelope(
        source.sourceId,
        source.accessMode === "enabled_public" ? "no_matches" : "manual_required",
      )
    }
    const results = await executeDiscovery(criteria, PORTAL_REGISTRY, executor)
    expect(results).toHaveLength(5)
    expect(new Set(results.map((result) => result.sourceId)).size).toBe(5)
    const failed = results.find((result) => result.sourceId === "vietnamworks")
    expect(failed?.status).toBe("failed")
    expect(failed?.unsupportedConstraints).toEqual([
      "query",
      "locations",
      "maxExperience",
      "seniority",
      "recency",
    ])
    expect(results.find((result) => result.sourceId === "topcv")?.status).toBe(
      "no_matches",
    )
  })
})
