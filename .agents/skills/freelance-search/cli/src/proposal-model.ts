import type { ProposalMaterial } from "./models.js"

export function validateProposalMaterial(material: ProposalMaterial): void {
  if (!material.opportunityId) throw new Error("Proposal material requires opportunityId")
  if (!material.profileId) throw new Error("Proposal material requires profileId")
  if (material.unsupportedClaims.length > 0 && material.verificationStatus === "ready") {
    throw new Error("Proposal material with unsupported claims cannot be ready")
  }
}
