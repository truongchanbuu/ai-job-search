import { describe, expect, test } from "bun:test"
import { verifyListing } from "../src/verify.js"
import type { SourceListing } from "../src/contracts.js"

function listing(overrides: Partial<SourceListing> = {}): SourceListing {
  const now = "2026-07-28T00:00:00Z"
  return {
    sourceId: "topcv",
    sourceJobId: "1",
    sourceUrl: "https://example.test/job/1",
    canonicalUrl: "https://example.test/job/1",
    discoveredAt: now,
    verifiedAt: now,
    title: "Junior Java Developer",
    employer: "Example",
    location: "Ha Noi",
    descriptionSummary: "Build Java services. Two years of experience required.",
    skills: ["Java"],
    experienceEvidence: [],
    minExperienceYears: 2,
    maxExperienceYears: 2,
    seniority: "junior",
    employmentType: null,
    workMode: null,
    postedAt: null,
    deadline: "2026-08-15",
    salary: null,
    applyUrl: null,
    language: "en",
    activeEvidence: ["future deadline"],
    closedEvidence: [],
    discoveryKind: "search_result",
    verification: {
      status: "unverified_lead",
      detailAccessible: true,
      active: null,
      descriptionSufficient: false,
      experienceDecisionSupported: false,
      checkedAt: now,
      reasonCodes: [],
    },
    rawFieldEvidence: [],
    warnings: [],
    ...overrides,
  }
}

describe("verification transitions", () => {
  test("verifies active sufficient detail", () => {
    expect(verifyListing(listing()).status).toBe("verified_active")
  })

  test("blocks title-only, expired, removed, and restricted details", () => {
    expect(
      verifyListing(listing({ descriptionSummary: null })).status,
    ).toBe("insufficient_detail")
    expect(
      verifyListing(listing({ closedEvidence: ["expired marker"] })).status,
    ).toBe("expired")
    expect(
      verifyListing(
        listing({
          verification: {
            ...listing().verification,
            status: "restricted",
            detailAccessible: false,
          },
        }),
      ).status,
    ).toBe("restricted")
  })
})
