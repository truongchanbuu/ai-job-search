export const CONTRACT_VERSION = "1" as const

export const VIETNAM_SOURCE_IDS = [
  "careerviet",
  "vieclam24h",
  "topcv",
  "itviec",
  "vietnamworks",
] as const

export type VietnamSourceId = (typeof VIETNAM_SOURCE_IDS)[number]
export type SourceId = VietnamSourceId | "linkedin" | "freehire" | "google" | string
export type AccessMode = "enabled_public" | "manual_only" | "restricted"
export type SearchCapability = "automated" | "official_link" | "manual"
export type DetailCapability =
  | "automated_public"
  | "user_supplied_public"
  | "pasted_text"
export type ConstraintName =
  | "query"
  | "location"
  | "experience"
  | "seniority"
  | "recency"
  | "workMode"
  | "employmentType"
  | "salary"
export type PortalRunStatus =
  | "searched"
  | "no_matches"
  | "manual_required"
  | "restricted"
  | "unavailable"
  | "failed"
export type VerificationStatus =
  | "unverified_lead"
  | "verified_active"
  | "insufficient_detail"
  | "expired"
  | "removed"
  | "restricted"
  | "failed"

export interface PortalSource {
  sourceId: VietnamSourceId
  displayName: string
  baseUrl: string
  cliPath: string
  accessMode: AccessMode
  enabledByDefault: boolean
  searchCapability: SearchCapability
  detailCapability: DetailCapability
  supportedConstraints: ConstraintName[]
  policyNotes: string
  accessReviewedAt: string
  requestTimeoutMs: number
}

export interface SearchCriteria {
  queryTerms: string[]
  skills: string[]
  locations: string[]
  maxExperienceYears: number | null
  seniority: string[]
  postedWithinDays: number | null
  workModes: string[]
  employmentTypes: string[]
  exclusions: string[]
  languageTerms: string[]
  limitPerSource: number
}

export interface FieldEvidence {
  field: string
  value: unknown
  sourceId: SourceId
  sourceUrl: string
  evidenceKind: "detail_text" | "structured_data" | "explicit_label" | "user_supplied_text"
  observedAt: string
  confidence: "direct" | "parsed" | "ambiguous"
}

export interface ListingVerification {
  status: VerificationStatus
  detailAccessible: boolean
  active: boolean | null
  descriptionSufficient: boolean
  experienceDecisionSupported: boolean
  checkedAt: string
  reasonCodes: string[]
}

export interface SourceListing {
  sourceId: SourceId
  sourceJobId: string | null
  sourceUrl: string
  canonicalUrl: string
  discoveredAt: string
  verifiedAt: string | null
  title: string | null
  employer: string | null
  location: string | null
  descriptionSummary: string | null
  skills: string[]
  experienceEvidence: FieldEvidence[]
  minExperienceYears: number | null
  maxExperienceYears: number | null
  seniority: string | null
  employmentType: string | null
  workMode: string | null
  postedAt: string | null
  deadline: string | null
  salary: string | null
  applyUrl: string | null
  language: string | null
  activeEvidence: string[]
  closedEvidence: string[]
  discoveryKind: "search_result" | "indexed_lead" | "user_supplied_url" | "pasted_description"
  verification: ListingVerification
  rawFieldEvidence: FieldEvidence[]
  warnings: string[]
}

export interface PortalRunResult {
  sourceId: SourceId
  status: PortalRunStatus
  startedAt: string
  completedAt: string
  appliedConstraints: Record<string, unknown>
  unsupportedConstraints: string[]
  manualReviewUrl: string | null
  rawDiscoveryCount: number
  verifiedActiveCount: number
  duplicateCount: number
  expiredOrRemovedCount: number
  inaccessibleLeadCount: number
  actionableCount: number
  listings: SourceListing[]
  reasonCode: string | null
  message: string | null
  warnings: string[]
}

export interface FieldConflictValue {
  value: unknown
  sourceId: SourceId
  sourceUrl: string
  observedAt: string
}

export interface FieldConflict {
  field: string
  values: FieldConflictValue[]
  preferredValue: unknown | null
  preferenceReason: string
}

export interface FitEvidence {
  eligibility: "eligible" | "ineligible" | "insufficient_evidence"
  matchedSkills: string[]
  missingSkills: string[]
  experienceDecision: "within_limit" | "exceeds_limit" | "explicit_exception" | "unknown"
  experienceEvidenceRefs: string[]
  locationDecision: "match" | "mismatch" | "unknown"
  workModeDecision: "match" | "mismatch" | "unknown"
  employmentTypeDecision: "match" | "mismatch" | "unknown"
  hardConstraintViolations: string[]
  recommendation: "strong" | "possible" | "weak" | "skip" | null
  rationale: string
}

export interface SourceReference {
  source: SourceId
  id: string | null
  url: string
}

export interface CanonicalJobListing {
  jobId: string
  canonicalKey: string
  sourceListings: SourceListing[]
  title: string | null
  employer: string | null
  location: string | null
  skills: string[]
  experience: string | null
  seniority: string | null
  employmentType: string | null
  workMode: string | null
  postedAt: string | null
  deadline: string | null
  salary: string | null
  conflicts: FieldConflict[]
  verificationStatus: VerificationStatus
  existingSeenState: "new" | "seen" | "dismissed" | "shortlisted" | null
  existingApplicationStatus: string | null
  fitEvidence: FitEvidence | null
}

export interface SeenJobState {
  jobId: string
  canonicalKey: string
  sourceRefs: SourceReference[]
  state: "seen" | "dismissed" | "shortlisted"
  firstSeenAt: string
  lastSeenAt: string
  notes: string | null
}

export interface SourceCoverageRow {
  source: SourceId
  status: PortalRunStatus
  appliedConstraints: string[]
  unsupportedConstraints: string[]
  rawDiscoveries: number
  verifiedActive: number
  duplicates: number
  expiredOrRemoved: number
  inaccessibleLeads: number
  actionable: number
  reason: string | null
  manualReviewUrl: string | null
}

export interface SourceCoverageSummary {
  totalSources: number
  statusCounts: Record<PortalRunStatus, number>
  rawDiscoveries: number
  verifiedActive: number
  duplicates: number
  expiredOrRemoved: number
  inaccessibleLeads: number
  actionableMatches: number
  sourceRows: SourceCoverageRow[]
}

export interface PortalSearchRun {
  contractVersion: typeof CONTRACT_VERSION
  runId: string
  startedAt: string
  completedAt: string
  criteria: SearchCriteria
  sourceStatuses: PortalRunResult[]
  jobs: CanonicalJobListing[]
  unverifiedLeads: SourceListing[]
  coverage: SourceCoverageSummary
  warnings: string[]
  sessionPath?: string | null
}

export interface AdapterSearchResult {
  id: string | null
  source: SourceId
  title: string | null
  company: string | null
  location: string | null
  date: string | null
  deadline: string | null
  url: string
  discoveredAt: string
  verificationStatus: "unverified_lead"
  requiresVerification: true
}

export interface AdapterSearchEnvelope {
  contractVersion: typeof CONTRACT_VERSION
  source: SourceId
  accessMode: AccessMode
  status: PortalRunStatus
  query: Record<string, unknown>
  appliedConstraints: Record<string, unknown>
  unsupportedConstraints: string[]
  manualReviewUrl: string | null
  meta: { count: number; page: number; total?: number }
  results: AdapterSearchResult[]
  warnings: string[]
}

export interface AdapterDetailEnvelope {
  contractVersion: typeof CONTRACT_VERSION
  source: SourceId
  status: Exclude<VerificationStatus, "unverified_lead">
  job: Record<string, unknown> | null
  verification: Omit<ListingVerification, "checkedAt"> & { checkedAt?: string }
  warnings: string[]
}

const PORTAL_STATUSES = new Set<PortalRunStatus>([
  "searched",
  "no_matches",
  "manual_required",
  "restricted",
  "unavailable",
  "failed",
])

export function isSearchCriteria(value: unknown): value is SearchCriteria {
  if (!value || typeof value !== "object") return false
  const item = value as Partial<SearchCriteria>
  return (
    Array.isArray(item.queryTerms) &&
    Array.isArray(item.skills) &&
    Array.isArray(item.locations) &&
    (item.maxExperienceYears === null ||
      (typeof item.maxExperienceYears === "number" && item.maxExperienceYears >= 0)) &&
    Array.isArray(item.seniority) &&
    (item.postedWithinDays === null ||
      (typeof item.postedWithinDays === "number" && item.postedWithinDays >= 0)) &&
    Array.isArray(item.workModes) &&
    Array.isArray(item.employmentTypes) &&
    Array.isArray(item.exclusions) &&
    Array.isArray(item.languageTerms) &&
    typeof item.limitPerSource === "number" &&
    item.limitPerSource >= 0
  )
}

export function isPortalRunResult(value: unknown): value is PortalRunResult {
  if (!value || typeof value !== "object") return false
  const item = value as Partial<PortalRunResult>
  const counts = [
    item.rawDiscoveryCount,
    item.verifiedActiveCount,
    item.duplicateCount,
    item.expiredOrRemovedCount,
    item.inaccessibleLeadCount,
    item.actionableCount,
  ]
  return (
    typeof item.sourceId === "string" &&
    typeof item.status === "string" &&
    PORTAL_STATUSES.has(item.status as PortalRunStatus) &&
    typeof item.startedAt === "string" &&
    typeof item.completedAt === "string" &&
    counts.every((count) => typeof count === "number" && count >= 0) &&
    Array.isArray(item.listings) &&
    Array.isArray(item.unsupportedConstraints) &&
    Array.isArray(item.warnings)
  )
}
