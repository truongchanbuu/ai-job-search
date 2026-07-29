import { describe, expect, test } from "bun:test"
import { deduplicateListings } from "../src/deduplicate.js"
import type { SearchCriteria, SourceListing } from "../src/contracts.js"

const criteria: SearchCriteria = {
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
}

function listing(
  sourceId: string,
  sourceJobId: string,
  overrides: Partial<SourceListing> = {},
): SourceListing {
  const now = "2026-07-28T00:00:00Z"
  return {
    sourceId,
    sourceJobId,
    sourceUrl: `https://${sourceId}.example/jobs/${sourceJobId}`,
    canonicalUrl: `https://${sourceId}.example/jobs/${sourceJobId}`,
    discoveredAt: now,
    verifiedAt: now,
    title: "Junior Java Developer",
    employer: "Example Company",
    location: "Ho Chi Minh City",
    descriptionSummary: "Build Java services with up to 1 year of experience.",
    skills: ["Java"],
    experienceEvidence: [
      {
        field: "experience",
        value: "up to 1 year",
        sourceId,
        sourceUrl: `https://${sourceId}.example/jobs/${sourceJobId}`,
        evidenceKind: "detail_text",
        observedAt: now,
        confidence: "direct",
      },
    ],
    minExperienceYears: 0,
    maxExperienceYears: 1,
    seniority: "junior",
    employmentType: "FULL_TIME",
    workMode: null,
    postedAt: null,
    deadline: null,
    salary: null,
    applyUrl: null,
    language: "en",
    activeEvidence: ["active"],
    closedEvidence: [],
    discoveryKind: "search_result",
    verification: {
      status: "verified_active",
      detailAccessible: true,
      active: true,
      descriptionSufficient: true,
      experienceDecisionSupported: true,
      checkedAt: now,
      reasonCodes: [],
    },
    rawFieldEvidence: [],
    warnings: [],
    ...overrides,
  }
}

describe("layered deduplication", () => {
  test("merges cross-portal copies and preserves links and conflicts", () => {
    const jobs = deduplicateListings(
      [
        listing("topcv", "1", { salary: "10-12m" }),
        listing("itviec", "2", { salary: "Negotiable" }),
      ],
      criteria,
    )
    expect(jobs).toHaveLength(1)
    expect(jobs[0].sourceListings).toHaveLength(2)
    expect(jobs[0].conflicts.find((conflict) => conflict.field === "salary")?.values).toHaveLength(2)
  })

  test("merges tracking variants of an exact URL", () => {
    const base = listing("topcv", "1")
    const jobs = deduplicateListings(
      [
        base,
        listing("topcv", "1-copy", {
          sourceUrl: `${base.sourceUrl}?utm_source=x`,
          canonicalUrl: base.canonicalUrl,
        }),
      ],
      criteria,
    )
    expect(jobs).toHaveLength(1)
  })

  test("keeps same-title roles in different locations separate", () => {
    const jobs = deduplicateListings(
      [
        listing("topcv", "1"),
        listing("itviec", "2", { location: "Ha Noi" }),
      ],
      criteria,
    )
    expect(jobs).toHaveLength(2)
  })
})
