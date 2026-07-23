import { writeFileSync } from "node:fs"
import { join } from "node:path"
import type { FreelanceOpportunity, FreelanceProfileExtension, ProposalMaterial } from "./models.js"
import { nowIso, stableId } from "./models.js"
import { defaultFreelanceDir, ensureDir } from "./paths.js"
import { findUnsupportedClaims, redactSecrets } from "./safety.js"

export function draftProposal(opportunity: FreelanceOpportunity, profile: FreelanceProfileExtension, materialType: ProposalMaterial["materialType"] = "proposal"): ProposalMaterial {
  const dir = defaultFreelanceDir()
  ensureDir(dir)
  const evidenceRefs = [`profile:${profile.profile_id}`, `opportunity:${opportunity.opportunityId}`]
  const body = redactSecrets([
    `# ${materialType.replace("_", " ")}: ${opportunity.title}`,
    "",
    `Source: ${opportunity.platform}`,
    `Budget: ${opportunity.budget ?? "Unknown"}`,
    "",
    "## Draft",
    `I can help with ${opportunity.title}. Relevant services: ${(profile.target_services ?? []).join(", ") || "to be confirmed"}.`,
    "",
    "## Verification",
    opportunity.requiresVerification ? "- Verify original opportunity details before sending." : "- Opportunity details were provided or verified by the user.",
  ].join("\n"))
  const unsupportedClaims = findUnsupportedClaims(body)
  const filePath = join(dir, `${opportunity.opportunityId}-${materialType}.md`)
  writeFileSync(filePath, body + "\n", "utf8")
  return {
    materialId: stableId("mat", [opportunity.opportunityId, materialType]),
    opportunityId: opportunity.opportunityId,
    profileId: profile.profile_id,
    materialType,
    language: opportunity.language === "vi" ? "vi" : opportunity.language === "mixed" ? "mixed" : "en",
    filePath,
    evidenceRefs,
    unsupportedClaims,
    createdAt: nowIso(),
    verificationStatus: unsupportedClaims.length > 0 || opportunity.requiresVerification ? "needs_confirmation" : "ready",
  }
}
