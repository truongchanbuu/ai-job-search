import { describe, expect, test } from "bun:test"
import {
  CONTRACT_VERSION,
  isPortalRunResult,
  isSearchCriteria,
  type CanonicalJobListing,
  type FitEvidence,
  type ListingVerification,
} from "../src/contracts.js"

describe("shared contracts", () => {
  test("accepts normalized search criteria without changing mixed-language tokens", () => {
    expect(
      isSearchCriteria({
        queryTerms: ["Java backend", "lập trình viên", "ReactJS"],
        skills: ["Java", "VueJS"],
        locations: ["Việt Nam"],
        maxExperienceYears: 1,
        seniority: ["fresher"],
        postedWithinDays: 30,
        workModes: [],
        employmentTypes: [],
        exclusions: ["senior"],
        languageTerms: ["Java", "lập trình viên"],
        limitPerSource: 10,
      }),
    ).toBe(true)
  })

  test("rejects a portal result with an invalid status", () => {
    expect(
      isPortalRunResult({
        sourceId: "topcv",
        status: "okay",
        startedAt: new Date().toISOString(),
        completedAt: new Date().toISOString(),
        appliedConstraints: {},
        unsupportedConstraints: [],
        manualReviewUrl: null,
        rawDiscoveryCount: 0,
        verifiedActiveCount: 0,
        duplicateCount: 0,
        expiredOrRemovedCount: 0,
        inaccessibleLeadCount: 0,
        actionableCount: 0,
        listings: [],
        reasonCode: null,
        message: null,
        warnings: [],
      }),
    ).toBe(false)
  })

  test("models verified jobs and evidence-backed fit without optional fabrication", () => {
    const verification: ListingVerification = {
      status: "verified_active",
      detailAccessible: true,
      active: true,
      descriptionSufficient: true,
      experienceDecisionSupported: true,
      checkedAt: new Date().toISOString(),
      reasonCodes: [],
    }
    const fit: FitEvidence = {
      eligibility: "eligible",
      matchedSkills: ["Java"],
      missingSkills: [],
      experienceDecision: "within_limit",
      experienceEvidenceRefs: ["topcv:1"],
      locationDecision: "match",
      workModeDecision: "unknown",
      employmentTypeDecision: "unknown",
      hardConstraintViolations: [],
      recommendation: "strong",
      rationale: "Direct experience evidence is within the requested limit.",
    }
    const job: CanonicalJobListing = {
      jobId: "job_1",
      canonicalKey: "example|java-developer|ha-noi",
      sourceListings: [],
      title: "Java Developer",
      employer: "Example",
      location: "Ha Noi",
      skills: ["Java"],
      experience: null,
      seniority: null,
      employmentType: null,
      workMode: null,
      postedAt: null,
      deadline: null,
      salary: null,
      conflicts: [],
      verificationStatus: verification.status,
      existingSeenState: null,
      existingApplicationStatus: null,
      fitEvidence: fit,
    }
    expect(CONTRACT_VERSION).toBe("1")
    expect(job.salary).toBeNull()
    expect(job.fitEvidence?.experienceEvidenceRefs).toEqual(["topcv:1"])
  })
})
