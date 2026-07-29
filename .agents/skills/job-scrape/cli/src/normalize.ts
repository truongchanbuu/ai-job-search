import type {
  AdapterDetailEnvelope,
  AdapterSearchEnvelope,
  PortalRunResult,
  PortalRunStatus,
  SourceId,
  SourceListing,
} from "./contracts.js"

export function canonicalizeUrl(raw: string): string {
  try {
    const url = new URL(raw)
    const tracking = [
      "utm_source",
      "utm_medium",
      "utm_campaign",
      "utm_term",
      "utm_content",
      "ref",
      "source",
      "tracking",
    ]
    for (const key of tracking) url.searchParams.delete(key)
    url.hash = ""
    const value = url.toString()
    return value.endsWith("/") && url.pathname !== "/" ? value.slice(0, -1) : value
  } catch {
    return raw.trim()
  }
}

export function normalizeIdentityText(value: string | null | undefined): string {
  return (value ?? "")
    .normalize("NFKD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase()
    .replace(/đ/g, "d")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "")
}

function unverifiedListing(sourceId: SourceId, raw: Record<string, unknown>): SourceListing {
  const now = String(raw.discoveredAt ?? new Date().toISOString())
  const url = canonicalizeUrl(String(raw.url ?? ""))
  return {
    sourceId,
    sourceJobId: raw.id == null ? null : String(raw.id),
    sourceUrl: String(raw.url ?? ""),
    canonicalUrl: url,
    discoveredAt: now,
    verifiedAt: null,
    title: raw.title == null ? null : String(raw.title),
    employer:
      raw.company == null
        ? raw.employer == null
          ? null
          : String(raw.employer)
        : String(raw.company),
    location: raw.location == null ? null : String(raw.location),
    descriptionSummary: null,
    skills: [],
    experienceEvidence: [],
    minExperienceYears: null,
    maxExperienceYears: null,
    seniority: null,
    employmentType: null,
    workMode: null,
    postedAt: raw.date == null ? null : String(raw.date),
    deadline: raw.deadline == null ? null : String(raw.deadline),
    salary: null,
    applyUrl: null,
    language: null,
    activeEvidence: [],
    closedEvidence: [],
    discoveryKind: "search_result",
    verification: {
      status: "unverified_lead",
      detailAccessible: false,
      active: null,
      descriptionSufficient: false,
      experienceDecisionSupported: false,
      checkedAt: now,
      reasonCodes: ["DETAIL_NOT_CHECKED"],
    },
    rawFieldEvidence: [],
    warnings: [],
  }
}

export function normalizeAdapterSearch(
  sourceId: SourceId,
  value: unknown,
  startedAt: string,
  completedAt: string,
): PortalRunResult {
  if (!value || typeof value !== "object") {
    return failedRun(sourceId, startedAt, completedAt, "INVALID_OUTPUT", "Adapter returned invalid JSON")
  }
  const raw = value as Record<string, unknown>
  const rich = raw.contractVersion === "1" && raw.source
  const results = Array.isArray(raw.results)
    ? raw.results.filter((result): result is Record<string, unknown> => Boolean(result && typeof result === "object"))
    : []
  const listings = results.map((result) => unverifiedListing(sourceId, result))
  const status = rich
    ? normalizeStatus(raw.status, listings.length)
    : listings.length
      ? "searched"
      : "no_matches"
  const envelope = raw as unknown as Partial<AdapterSearchEnvelope>
  return {
    sourceId,
    status,
    startedAt,
    completedAt,
    appliedConstraints:
      envelope.appliedConstraints && typeof envelope.appliedConstraints === "object"
        ? envelope.appliedConstraints
        : {},
    unsupportedConstraints: Array.isArray(envelope.unsupportedConstraints)
      ? envelope.unsupportedConstraints.map(String)
      : [],
    manualReviewUrl:
      typeof envelope.manualReviewUrl === "string" ? envelope.manualReviewUrl : null,
    rawDiscoveryCount: listings.length,
    verifiedActiveCount: 0,
    duplicateCount: 0,
    expiredOrRemovedCount: 0,
    inaccessibleLeadCount: listings.length,
    actionableCount: 0,
    listings,
    reasonCode: null,
    message: null,
    warnings: Array.isArray(envelope.warnings) ? envelope.warnings.map(String) : [],
  }
}

function normalizeStatus(value: unknown, count: number): PortalRunStatus {
  const allowed: PortalRunStatus[] = [
    "searched",
    "no_matches",
    "manual_required",
    "restricted",
    "unavailable",
    "failed",
  ]
  if (typeof value === "string" && allowed.includes(value as PortalRunStatus)) {
    return value as PortalRunStatus
  }
  return count ? "searched" : "no_matches"
}

export function failedRun(
  sourceId: SourceId,
  startedAt: string,
  completedAt: string,
  reasonCode: string,
  message: string,
  status: PortalRunStatus = "failed",
): PortalRunResult {
  return {
    sourceId,
    status,
    startedAt,
    completedAt,
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
    reasonCode,
    message,
    warnings: [],
  }
}

function nullableString(value: unknown): string | null {
  return value == null ? null : String(value)
}

function stringList(value: unknown): string[] {
  return Array.isArray(value) ? value.map(String) : []
}

export function normalizeAdapterDetail(
  original: SourceListing,
  value: unknown,
): SourceListing {
  if (!value || typeof value !== "object") {
    return {
      ...original,
      verification: {
        status: "failed",
        detailAccessible: false,
        active: null,
        descriptionSufficient: false,
        experienceDecisionSupported: false,
        checkedAt: new Date().toISOString(),
        reasonCodes: ["INVALID_DETAIL_OUTPUT"],
      },
    }
  }
  const envelope = value as Partial<AdapterDetailEnvelope>
  const job =
    envelope.job && typeof envelope.job === "object"
      ? (envelope.job as Record<string, unknown>)
      : {}
  const checkedAt =
    typeof envelope.verification?.checkedAt === "string"
      ? envelope.verification.checkedAt
      : new Date().toISOString()
  const rawEvidence = Array.isArray(job.experienceEvidence)
    ? job.experienceEvidence
    : []
  const experienceEvidence = rawEvidence.map((item) => {
    const record =
      item && typeof item === "object" ? (item as Record<string, unknown>) : { value: item }
    return {
      field: "experience",
      value: record.value ?? null,
      sourceId: original.sourceId,
      sourceUrl: original.canonicalUrl,
      evidenceKind: "detail_text" as const,
      observedAt: checkedAt,
      confidence: "direct" as const,
    }
  })
  const status =
    typeof envelope.status === "string"
      ? envelope.status
      : "insufficient_detail"
  return {
    ...original,
    sourceJobId: nullableString(job.id) ?? original.sourceJobId,
    sourceUrl: nullableString(job.url) ?? original.sourceUrl,
    canonicalUrl: canonicalizeUrl(nullableString(job.url) ?? original.canonicalUrl),
    verifiedAt: nullableString(job.verifiedAt) ?? checkedAt,
    title: nullableString(job.title) ?? original.title,
    employer: nullableString(job.company) ?? nullableString(job.employer) ?? original.employer,
    location: nullableString(job.location) ?? original.location,
    descriptionSummary: nullableString(job.descriptionSummary),
    skills: stringList(job.skills),
    experienceEvidence,
    minExperienceYears:
      typeof job.minExperienceYears === "number" ? job.minExperienceYears : null,
    maxExperienceYears:
      typeof job.maxExperienceYears === "number" ? job.maxExperienceYears : null,
    seniority: nullableString(job.seniority),
    employmentType: nullableString(job.employmentType),
    workMode: nullableString(job.workMode),
    postedAt: nullableString(job.postedAt) ?? original.postedAt,
    deadline: nullableString(job.deadline) ?? original.deadline,
    salary: nullableString(job.salary),
    applyUrl: nullableString(job.applyUrl),
    language: nullableString(job.language),
    activeEvidence: stringList(job.activeEvidence),
    closedEvidence: stringList(job.closedEvidence),
    verification: {
      status: status as SourceListing["verification"]["status"],
      detailAccessible: Boolean(envelope.verification?.detailAccessible),
      active:
        typeof envelope.verification?.active === "boolean"
          ? envelope.verification.active
          : null,
      descriptionSufficient: Boolean(envelope.verification?.descriptionSufficient),
      experienceDecisionSupported: Boolean(
        envelope.verification?.experienceDecisionSupported,
      ),
      checkedAt,
      reasonCodes: Array.isArray(envelope.verification?.reasonCodes)
        ? envelope.verification.reasonCodes.map(String)
        : [],
    },
    rawFieldEvidence: experienceEvidence,
    warnings: Array.isArray(envelope.warnings) ? envelope.warnings.map(String) : [],
  }
}
