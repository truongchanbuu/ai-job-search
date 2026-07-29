import { describe, expect, test } from "bun:test"
import { renderRun } from "../src/render.js"
import type { PortalSearchRun } from "../src/contracts.js"

const run: PortalSearchRun = {
  contractVersion: "1",
  runId: "run_1",
  startedAt: "2026-07-28T00:00:00Z",
  completedAt: "2026-07-28T00:00:01Z",
  criteria: {
    queryTerms: ["Java"],
    skills: ["Java"],
    locations: [],
    maxExperienceYears: 1,
    seniority: [],
    postedWithinDays: null,
    workModes: [],
    employmentTypes: [],
    exclusions: [],
    languageTerms: ["Java"],
    limitPerSource: 10,
  },
  sourceStatuses: [],
  jobs: [
    {
      jobId: "job_1",
      canonicalKey: "example|java|ha-noi",
      sourceListings: [],
      title: "Java Developer",
      employer: null,
      location: null,
      skills: ["Java"],
      experience: "0-1 years",
      seniority: "junior",
      employmentType: null,
      workMode: null,
      postedAt: null,
      deadline: null,
      salary: null,
      conflicts: [
        {
          field: "salary",
          values: [
            {
              value: "10m",
              sourceId: "topcv",
              sourceUrl: "https://topcv.example/1",
              observedAt: "2026-07-28T00:00:00Z",
            },
            {
              value: "Negotiable",
              sourceId: "itviec",
              sourceUrl: "https://itviec.example/2",
              observedAt: "2026-07-28T00:00:00Z",
            },
          ],
          preferredValue: "10m",
          preferenceReason: "verified evidence",
        },
      ],
      verificationStatus: "verified_active",
      existingSeenState: "seen",
      existingApplicationStatus: null,
      fitEvidence: {
        eligibility: "eligible",
        matchedSkills: ["Java"],
        missingSkills: [],
        experienceDecision: "within_limit",
        experienceEvidenceRefs: ["topcv:1"],
        locationDecision: "unknown",
        workModeDecision: "unknown",
        employmentTypeDecision: "unknown",
        hardConstraintViolations: [],
        recommendation: "strong",
        rationale: "Verified evidence.",
      },
    },
  ],
  unverifiedLeads: [],
  coverage: {
    totalSources: 1,
    statusCounts: {
      searched: 0,
      no_matches: 0,
      manual_required: 1,
      restricted: 0,
      unavailable: 0,
      failed: 0,
    },
    rawDiscoveries: 0,
    verifiedActive: 0,
    duplicates: 0,
    expiredOrRemoved: 0,
    inaccessibleLeads: 0,
    actionableMatches: 1,
    sourceRows: [
      {
        source: "careerviet",
        status: "manual_required",
        appliedConstraints: ["query"],
        unsupportedConstraints: ["automatedSearch"],
        rawDiscoveries: 0,
        verifiedActive: 0,
        duplicates: 0,
        expiredOrRemoved: 0,
        inaccessibleLeads: 0,
        actionable: 0,
        reason: "manual policy",
        manualReviewUrl: "https://careerviet.example/search",
      },
    ],
  },
  warnings: ["Access is limited."],
}

describe("run rendering", () => {
  test("keeps the full JSON contract", () => {
    expect(JSON.parse(renderRun(run, "json")).runId).toBe("run_1")
  })

  test("shows coverage, constraints, unknown facts, shortlist, conflicts, and warnings", () => {
    const output = renderRun(run, "table")
    expect(output).toContain("Portal coverage")
    expect(output).toContain("manual_required")
    expect(output).toContain("unapplied: automatedSearch")
    expect(output).toContain("Actionable shortlist")
    expect(output).toContain("Unknown employer")
    expect(output).toContain("Conflict salary")
    expect(output).toContain("Access is limited.")
  })
})
