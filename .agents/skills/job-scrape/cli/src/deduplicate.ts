import type {
  CanonicalJobListing,
  FieldConflict,
  SearchCriteria,
  SourceListing,
  VerificationStatus,
} from "./contracts.js"
import { evaluateFit } from "./fit.js"
import { canonicalizeUrl, normalizeIdentityText } from "./normalize.js"

function stableId(value: string): string {
  let hash = 2166136261
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index)
    hash = Math.imul(hash, 16777619)
  }
  return `job_${(hash >>> 0).toString(16).padStart(8, "0")}`
}

function identity(listing: SourceListing): string {
  const parts = [
    normalizeIdentityText(listing.employer),
    normalizeIdentityText(listing.title),
    normalizeIdentityText(listing.location),
  ]
  return parts.every(Boolean) ? parts.join("|") : ""
}

function tokens(value: string | null): Set<string> {
  return new Set(
    normalizeIdentityText(value)
      .split("-")
      .filter((item) => item.length >= 3),
  )
}

function similarity(left: string | null, right: string | null): number {
  const a = tokens(left)
  const b = tokens(right)
  if (!a.size || !b.size) return 0
  const intersection = [...a].filter((item) => b.has(item)).length
  return intersection / new Set([...a, ...b]).size
}

function sameVacancy(left: SourceListing, right: SourceListing): boolean {
  if (
    canonicalizeUrl(left.canonicalUrl) === canonicalizeUrl(right.canonicalUrl) ||
    (left.sourceId === right.sourceId &&
      left.sourceJobId != null &&
      left.sourceJobId === right.sourceJobId)
  ) {
    return true
  }
  const leftIdentity = identity(left)
  const rightIdentity = identity(right)
  if (leftIdentity && leftIdentity === rightIdentity) return true
  const sameEmployer =
    normalizeIdentityText(left.employer) !== "" &&
    normalizeIdentityText(left.employer) === normalizeIdentityText(right.employer)
  const sameTitle =
    normalizeIdentityText(left.title) !== "" &&
    normalizeIdentityText(left.title) === normalizeIdentityText(right.title)
  const sameLocation =
    normalizeIdentityText(left.location) !== "" &&
    normalizeIdentityText(left.location) === normalizeIdentityText(right.location)
  return (
    sameEmployer &&
    sameTitle &&
    sameLocation &&
    similarity(left.descriptionSummary, right.descriptionSummary) >= 0.75
  )
}

function conflict(
  field: string,
  listings: SourceListing[],
  value: (listing: SourceListing) => unknown,
): FieldConflict | null {
  const observed = listings
    .map((listing) => ({
      value: value(listing),
      sourceId: listing.sourceId,
      sourceUrl: listing.canonicalUrl,
      observedAt: listing.verifiedAt ?? listing.discoveredAt,
    }))
    .filter((item) => item.value !== null && item.value !== undefined && item.value !== "")
  const unique = new Set(observed.map((item) => JSON.stringify(item.value)))
  if (unique.size <= 1) return null
  const preferred = listings.find(
    (listing) =>
      listing.verification.status === "verified_active" &&
      value(listing) !== null &&
      value(listing) !== undefined,
  )
  return {
    field,
    values: observed,
    preferredValue: preferred ? value(preferred) : null,
    preferenceReason: preferred
      ? "verified active source with public detail evidence"
      : "UNRESOLVED",
  }
}

function preferred<T>(
  listings: SourceListing[],
  value: (listing: SourceListing) => T | null,
): T | null {
  const verified = listings.find(
    (listing) => listing.verification.status === "verified_active" && value(listing) != null,
  )
  return verified ? value(verified) : (listings.map(value).find((item) => item != null) ?? null)
}

function verificationStatus(listings: SourceListing[]): VerificationStatus {
  const order: VerificationStatus[] = [
    "verified_active",
    "insufficient_detail",
    "unverified_lead",
    "restricted",
    "expired",
    "removed",
    "failed",
  ]
  return order.find((status) => listings.some((listing) => listing.verification.status === status)) ?? "failed"
}

function canonicalJob(
  listings: SourceListing[],
  criteria: SearchCriteria,
): CanonicalJobListing {
  const key = identity(listings[0]) || canonicalizeUrl(listings[0].canonicalUrl)
  const conflicts = [
    conflict("title", listings, (listing) => listing.title),
    conflict("employer", listings, (listing) => listing.employer),
    conflict("location", listings, (listing) => listing.location),
    conflict("salary", listings, (listing) => listing.salary),
    conflict("deadline", listings, (listing) => listing.deadline),
    conflict("experience", listings, (listing) => [
      listing.minExperienceYears,
      listing.maxExperienceYears,
    ]),
    conflict("workMode", listings, (listing) => listing.workMode),
    conflict("employmentType", listings, (listing) => listing.employmentType),
  ].filter((item): item is FieldConflict => item !== null)
  const bestFit = listings
    .map((listing) => evaluateFit(listing, criteria))
    .find((fit) => fit.recommendation != null)
  const min = preferred(listings, (listing) => listing.minExperienceYears)
  const max = preferred(listings, (listing) => listing.maxExperienceYears)
  return {
    jobId: stableId(key),
    canonicalKey: key,
    sourceListings: listings,
    title: preferred(listings, (listing) => listing.title),
    employer: preferred(listings, (listing) => listing.employer),
    location: preferred(listings, (listing) => listing.location),
    skills: [...new Set(listings.flatMap((listing) => listing.skills))],
    experience: min == null && max == null ? null : `${min ?? "?"}-${max ?? "?"} years`,
    seniority: preferred(listings, (listing) => listing.seniority),
    employmentType: preferred(listings, (listing) => listing.employmentType),
    workMode: preferred(listings, (listing) => listing.workMode),
    postedAt: preferred(listings, (listing) => listing.postedAt),
    deadline: preferred(listings, (listing) => listing.deadline),
    salary: preferred(listings, (listing) => listing.salary),
    conflicts,
    verificationStatus: verificationStatus(listings),
    existingSeenState: null,
    existingApplicationStatus: null,
    fitEvidence: bestFit ?? null,
  }
}

export function deduplicateListings(
  listings: SourceListing[],
  criteria: SearchCriteria,
): CanonicalJobListing[] {
  const groups: SourceListing[][] = []
  for (const listing of listings) {
    const group = groups.find((candidate) =>
      candidate.some((existing) => sameVacancy(existing, listing)),
    )
    if (group) group.push(listing)
    else groups.push([listing])
  }
  return groups.map((group) => canonicalJob(group, criteria))
}
