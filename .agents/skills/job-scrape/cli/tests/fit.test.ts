import { describe, expect, test } from "bun:test"
import { evaluateFit } from "../src/fit.js"
import type { SearchCriteria, SourceListing } from "../src/contracts.js"

const criteria: SearchCriteria = {
  queryTerms: ["Java backend"],
  skills: ["Java"],
  locations: ["Ha Noi"],
  maxExperienceYears: 1,
  seniority: [],
  postedWithinDays: null,
  workModes: [],
  employmentTypes: [],
  exclusions: ["senior"],
  languageTerms: ["Java backend"],
  limitPerSource: 10,
}

function listing(maxExperienceYears: number | null, status = "verified_active"): SourceListing {
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
    descriptionSummary: "Java role with explicit experience evidence",
    skills: ["Java"],
    experienceEvidence:
      maxExperienceYears == null
        ? []
        : [
            {
              field: "experience",
              value: `${maxExperienceYears} years`,
              sourceId: "topcv",
              sourceUrl: "https://example.test/job/1",
              evidenceKind: "explicit_label",
              observedAt: now,
              confidence: "direct",
            },
          ],
    minExperienceYears: maxExperienceYears,
    maxExperienceYears,
    seniority: "junior",
    employmentType: null,
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
      status: status as SourceListing["verification"]["status"],
      detailAccessible: status === "verified_active",
      active: status === "verified_active",
      descriptionSufficient: status === "verified_active",
      experienceDecisionSupported: maxExperienceYears != null,
      checkedAt: now,
      reasonCodes: [],
    },
    rawFieldEvidence: [],
    warnings: [],
  }
}

describe("fit evidence gate", () => {
  test("description experience overrides a junior title", () => {
    const fit = evaluateFit(listing(2), criteria)
    expect(fit.experienceDecision).toBe("exceeds_limit")
    expect(fit.recommendation).toBe("skip")
  })

  test("allows an evidence-backed within-limit role", () => {
    const fit = evaluateFit(listing(1), criteria)
    expect(fit.experienceDecision).toBe("within_limit")
    expect(fit.experienceEvidenceRefs).toEqual(["topcv:1"])
    expect(fit.recommendation).toBe("strong")
  })

  test("never recommends unverified or experience-unknown listings", () => {
    expect(evaluateFit(listing(1, "unverified_lead"), criteria).recommendation).toBeNull()
    expect(evaluateFit(listing(null), criteria).recommendation).toBeNull()
  })
})
