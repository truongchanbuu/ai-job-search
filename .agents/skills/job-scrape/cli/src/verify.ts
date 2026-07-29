import type { ListingVerification, SourceListing } from "./contracts.js"

export function verifyListing(
  listing: SourceListing,
  checkedAt = new Date().toISOString(),
): ListingVerification {
  const incoming = listing.verification
  if (["restricted", "failed", "removed"].includes(incoming.status)) {
    return { ...incoming, checkedAt }
  }
  if (listing.closedEvidence.length > 0 || incoming.status === "expired") {
    return {
      status: "expired",
      detailAccessible: incoming.detailAccessible,
      active: false,
      descriptionSufficient: Boolean(listing.descriptionSummary?.trim()),
      experienceDecisionSupported:
        listing.experienceEvidence.length > 0 ||
        listing.minExperienceYears != null ||
        listing.maxExperienceYears != null,
      checkedAt,
      reasonCodes: [...new Set([...incoming.reasonCodes, "CLOSED_MARKER"])],
    }
  }
  if (!incoming.detailAccessible) {
    return {
      ...incoming,
      status: incoming.status === "unverified_lead" ? "insufficient_detail" : incoming.status,
      active: null,
      descriptionSufficient: false,
      checkedAt,
      reasonCodes: [...new Set([...incoming.reasonCodes, "DETAIL_INACCESSIBLE"])],
    }
  }
  const descriptionSufficient = (listing.descriptionSummary?.trim().length ?? 0) >= 30
  if (!descriptionSufficient) {
    return {
      status: "insufficient_detail",
      detailAccessible: true,
      active: incoming.active,
      descriptionSufficient: false,
      experienceDecisionSupported: false,
      checkedAt,
      reasonCodes: [...new Set([...incoming.reasonCodes, "NO_REQUIREMENTS"])],
    }
  }
  return {
    status: "verified_active",
    detailAccessible: true,
    active: true,
    descriptionSufficient: true,
    experienceDecisionSupported:
      listing.experienceEvidence.length > 0 ||
      listing.minExperienceYears != null ||
      listing.maxExperienceYears != null,
    checkedAt,
    reasonCodes: incoming.reasonCodes.filter(
      (reason) => !["DETAIL_NOT_CHECKED", "NO_REQUIREMENTS"].includes(reason),
    ),
  }
}
