import type { FreelanceOpportunity, SourceReliabilityNote } from "./models.js"
import { nowIso, stableId } from "./models.js"

export function reliabilityForOpportunity(opportunity: FreelanceOpportunity): SourceReliabilityNote {
  if (opportunity.requiresVerification) {
    return {
      noteId: stableId("rel", [opportunity.opportunityId, "discovery"]),
      sourceId: String(opportunity.platform),
      reliability: "discovery_lead",
      reason: "Original opportunity content has not been verified.",
      checkedAt: nowIso(),
      recommendedHandling: "verify_original",
    }
  }
  return {
    noteId: stableId("rel", [opportunity.opportunityId, "verified"]),
    sourceId: String(opportunity.platform),
    reliability: "verified",
    reason: "Opportunity details were provided or verified by the user.",
    checkedAt: nowIso(),
    recommendedHandling: "safe_to_use",
  }
}
