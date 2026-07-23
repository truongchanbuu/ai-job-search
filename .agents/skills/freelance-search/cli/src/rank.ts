import type { FreelanceFitMatch, FreelanceOpportunity, FreelanceProfileExtension, PlatformAccountReadiness } from "./models.js"
import { nowIso, stableId } from "./models.js"
import { profileKeywords } from "./profile.js"
import { readinessFor } from "./readiness-store.js"
import { readinessStatus } from "./readiness-model.js"

export function createFitMatch(
  opportunity: FreelanceOpportunity,
  profile: FreelanceProfileExtension,
  readiness: PlatformAccountReadiness[] = [],
): FreelanceFitMatch {
  const keywords = profileKeywords(profile)
  const haystack = `${opportunity.title} ${opportunity.description ?? ""} ${opportunity.skills.join(" ")}`.toLowerCase()
  const matchedSkills = keywords.filter((keyword) => haystack.includes(keyword))
  const platformReady = readinessStatus(readinessFor(String(opportunity.platform), readiness))
  const budgetFit = scoreBudget(opportunity.budget, profile)
  const timelineFit = opportunity.timeline?.toLowerCase().includes("today") ? "tight" : opportunity.timeline ? "available" : "unknown"
  const highRisk = opportunity.riskFlags.some((flag) => ["scam_risk", "unpaid_or_speculative", "not_freelance"].includes(flag))
  const confidence = opportunity.requiresVerification ? "limited" : matchedSkills.length > 0 ? "high" : "medium"
  const recommendation =
    highRisk ? "skip" :
    platformReady === "blocked" ? "skip" :
    platformReady === "setup_required" || platformReady === "unknown" || platformReady === "limited" ? "setup_first" :
    matchedSkills.length > 0 && budgetFit !== "low" && !opportunity.requiresVerification ? "proposal_worthy" :
    opportunity.requiresVerification ? "watch" : "review"

  return {
    fitId: stableId("fit", [opportunity.opportunityId, profile.profile_id]),
    opportunityId: opportunity.opportunityId,
    profileId: profile.profile_id,
    matchedSkills: matchedSkills.map((item) => item[0].toUpperCase() + item.slice(1)),
    missingRequirements: [],
    budgetFit,
    timelineFit,
    platformReadiness: platformReady,
    riskFlags: opportunity.riskFlags,
    confidence,
    recommendation,
    nextAction: nextAction(recommendation, opportunity.requiresVerification),
    createdAt: nowIso(),
  }
}

export function rankOpportunities(
  opportunities: FreelanceOpportunity[],
  profile: FreelanceProfileExtension,
  readiness: PlatformAccountReadiness[] = [],
): FreelanceOpportunity[] {
  return opportunities
    .map((opportunity) => ({ ...opportunity, fit: createFitMatch(opportunity, profile, readiness) }))
    .sort((a, b) => score(b) - score(a))
}

function score(opportunity: FreelanceOpportunity): number {
  const fit = opportunity.fit
  if (!fit) return 0
  const recScore: Record<string, number> = { proposal_worthy: 50, review: 35, watch: 20, setup_first: 15, skip: -50 }
  return (recScore[fit.recommendation] ?? 0) + fit.matchedSkills.length * 10 - fit.riskFlags.length * 8
}

function scoreBudget(budget: string | null, profile: FreelanceProfileExtension): FreelanceFitMatch["budgetFit"] {
  if (!budget) return "unknown"
  const min = profile.rate_preferences?.fixed_min
  const amount = Number((budget.match(/\d+/)?.[0] ?? "").trim())
  if (!Number.isFinite(amount) || amount <= 0) return "unknown"
  if (min && amount < min) return "low"
  if (amount >= 500) return "strong"
  return "acceptable"
}

function nextAction(recommendation: FreelanceFitMatch["recommendation"], requiresVerification: boolean): string {
  if (requiresVerification) return "Verify the original opportunity details before drafting."
  if (recommendation === "proposal_worthy") return "Draft a targeted proposal."
  if (recommendation === "setup_first") return "Resolve account readiness before action."
  if (recommendation === "skip") return "Skip or archive this opportunity."
  return "Review details and decide whether to proceed."
}
