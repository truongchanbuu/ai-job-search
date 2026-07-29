import type { FitEvidence, SearchCriteria, SourceListing } from "./contracts.js"

function includes(value: string | null, expected: string): boolean {
  return (value ?? "").toLocaleLowerCase().includes(expected.toLocaleLowerCase())
}

function decision(
  actual: string | null,
  requested: string[],
): "match" | "mismatch" | "unknown" {
  if (requested.length === 0) return "unknown"
  if (!actual) return "unknown"
  return requested.some((value) => includes(actual, value)) ? "match" : "mismatch"
}

export function evaluateFit(
  listing: SourceListing,
  criteria: SearchCriteria,
): FitEvidence {
  const base: FitEvidence = {
    eligibility: "insufficient_evidence",
    matchedSkills: [],
    missingSkills: [],
    experienceDecision: "unknown",
    experienceEvidenceRefs: [],
    locationDecision: decision(listing.location, criteria.locations),
    workModeDecision: decision(listing.workMode, criteria.workModes),
    employmentTypeDecision: decision(listing.employmentType, criteria.employmentTypes),
    hardConstraintViolations: [],
    recommendation: null,
    rationale: "Listing is not verified with enough source evidence.",
  }
  if (listing.verification.status !== "verified_active") return base

  const evidenceText = [
    listing.title,
    listing.descriptionSummary,
    listing.skills.join(" "),
  ]
    .filter(Boolean)
    .join(" ")
    .toLocaleLowerCase()
  const requestedSkills = criteria.skills
  base.matchedSkills = requestedSkills.filter((skill) =>
    evidenceText.includes(skill.toLocaleLowerCase()),
  )
  base.missingSkills = requestedSkills.filter(
    (skill) => !base.matchedSkills.includes(skill),
  )
  for (const exclusion of criteria.exclusions) {
    if (evidenceText.includes(exclusion.toLocaleLowerCase())) {
      base.hardConstraintViolations.push(`excluded term: ${exclusion}`)
    }
  }
  if (base.locationDecision === "mismatch") {
    base.hardConstraintViolations.push("location mismatch")
  }
  if (base.workModeDecision === "mismatch") {
    base.hardConstraintViolations.push("work-mode mismatch")
  }
  if (base.employmentTypeDecision === "mismatch") {
    base.hardConstraintViolations.push("employment-type mismatch")
  }

  if (criteria.maxExperienceYears != null) {
    if (
      listing.maxExperienceYears == null ||
      !listing.verification.experienceDecisionSupported
    ) {
      base.experienceDecision = "unknown"
      base.rationale = "Experience limit cannot be evaluated from source evidence."
      return base
    }
    base.experienceEvidenceRefs = [
      `${listing.sourceId}:${listing.sourceJobId ?? listing.canonicalUrl}`,
    ]
    if (listing.maxExperienceYears > criteria.maxExperienceYears) {
      base.experienceDecision = "exceeds_limit"
      base.hardConstraintViolations.push("experience requirement exceeds limit")
    } else if (
      listing.minExperienceYears === 0 &&
      listing.experienceEvidence.some((item) =>
        /fresh|no experience|mới tốt nghiệp/i.test(String(item.value)),
      )
    ) {
      base.experienceDecision = "explicit_exception"
    } else {
      base.experienceDecision = "within_limit"
    }
  } else {
    base.experienceDecision = "unknown"
  }

  if (base.hardConstraintViolations.length > 0) {
    base.eligibility = "ineligible"
    base.recommendation = "skip"
    base.rationale = base.hardConstraintViolations.join("; ")
    return base
  }
  base.eligibility = "eligible"
  const allSkillsMatched =
    requestedSkills.length === 0 || base.missingSkills.length === 0
  base.recommendation = allSkillsMatched ? "strong" : "possible"
  base.rationale = [
    "Public detail is active and sufficiently detailed.",
    criteria.maxExperienceYears == null
      ? "No experience limit requested."
      : "Source evidence supports the experience decision.",
  ].join(" ")
  return base
}
