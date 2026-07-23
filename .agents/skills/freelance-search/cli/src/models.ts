export type OutputFormat = "json" | "table" | "plain"
export type SourceType = "fiverr" | "upwork" | "google" | "social" | "community" | "marketplace" | "user_supplied"
export type SourceStatus = "available" | "needs_setup" | "unavailable" | "blocked" | "skipped"
export type VerificationStatus = "draft" | "needs_confirmation" | "verified" | "ready"
export type AccountStatus = "not_configured" | "draft" | "ready" | "limited" | "blocked" | "unknown"
export type Recommendation = "proposal_worthy" | "review" | "setup_first" | "watch" | "skip"
export type BudgetFit = "strong" | "acceptable" | "low" | "unknown" | "skip"
export type TimelineFit = "available" | "tight" | "unavailable" | "unknown"
export type Confidence = "high" | "medium" | "limited"

export interface FreelanceProfileExtension {
  profile_id: string
  target_services: string[]
  portfolio_links?: string[]
  rate_preferences?: {
    hourly?: string
    fixed_min?: number
    currency?: string
    notes?: string
  }
  availability?: {
    weekly_hours?: number
    start_date?: string
    timezone?: string
    notes?: string
  }
  work_type_preferences?: string[]
  languages?: string[]
  platform_profiles?: Record<string, string>
  exclusions?: string[]
  main_skills?: string[]
  updated_at?: string
}

export interface FreelanceSource {
  source_id: string
  source_type: SourceType
  display_name: string
  enabled: boolean
  scope?: string
  access_mode: "public" | "account_required" | "manual" | "user_supplied" | "blocked"
  status: SourceStatus
  notes?: string
  last_checked_at?: string
}

export interface PlatformAccountReadiness {
  platform: "fiverr" | "upwork" | string
  accountStatus: AccountStatus
  profileCompleteness: string[]
  serviceCategories: string[]
  portfolioRefs: string[]
  verificationStatus: "not_started" | "pending" | "verified" | "failed" | "unknown"
  actionCurrency?: string
  knownBlockers: string[]
  lastUpdatedAt: string
}

export interface SourceReference {
  source_id: string
  url_or_identifier: string | null
  retrieved_at: string
  raw_title?: string
  raw_client_or_source?: string
  raw_budget?: string
  status: "retrieved" | "partial" | "discovery_lead" | "unavailable" | "blocked" | "expired"
  reliability_note_id?: string
}

export interface FreelanceOpportunity {
  opportunityId: string
  canonicalKey: string
  sourceOpportunityId?: string
  title: string
  clientOrSource: string | null
  platform: SourceType | string
  url: string | null
  budget: string | null
  timeline: string | null
  description: string | null
  skills: string[]
  language: "en" | "vi" | "mixed" | "unknown"
  sourceRefs: SourceReference[]
  riskFlags: string[]
  discoveredAt: string
  postedAt?: string | null
  sourceStatus: string
  requiresVerification: boolean
  existingRecordStatus?: string | null
  fit?: FreelanceFitMatch
}

export interface SourceStatusSummary {
  source: string
  status: SourceStatus
  count: number
  message?: string
}

export interface SearchOutput {
  profileId: string
  sourceStatuses: SourceStatusSummary[]
  opportunities: FreelanceOpportunity[]
  warnings: string[]
}

export interface FreelanceFitMatch {
  fitId: string
  opportunityId: string
  profileId: string
  matchedSkills: string[]
  missingRequirements: string[]
  budgetFit: BudgetFit
  timelineFit: TimelineFit
  platformReadiness: "ready" | "setup_required" | "limited" | "blocked" | "unknown"
  riskFlags: string[]
  confidence: Confidence
  recommendation: Recommendation
  nextAction: string
  createdAt: string
}

export interface ProposalMaterial {
  materialId: string
  opportunityId: string
  profileId: string
  materialType: "proposal" | "response_outline" | "gig_note" | "follow_up"
  language: "en" | "vi" | "mixed"
  filePath: string
  evidenceRefs: string[]
  unsupportedClaims: string[]
  createdAt: string
  verificationStatus: VerificationStatus
}

export interface FreelanceOpportunityRecord {
  recordId: string
  opportunityId: string
  status: "discovered" | "reviewing" | "proposal_drafted" | "applied" | "interviewing" | "active" | "won" | "lost" | "ignored" | "scam" | "archived"
  savedAt: string
  updatedAt: string
  actionedAt?: string
  sourceRefs: SourceReference[]
  materialRefs: string[]
  platform: string
  budget?: string | null
  notes?: string
  outcome?: string
}

export interface SourceReliabilityNote {
  noteId: string
  sourceId: string
  reliability: "verified" | "partial" | "discovery_lead" | "stale" | "private" | "risky" | "unknown"
  reason: string
  checkedAt: string
  recommendedHandling: "verify_original" | "request_user_context" | "skip" | "safe_to_use"
}

export interface Flags {
  _: string[]
  [k: string]: string | boolean | string[]
}

export function nowIso(): string {
  return new Date().toISOString()
}

export function stableId(prefix: string, parts: Array<string | null | undefined>): string {
  const base = parts.filter(Boolean).join("|").toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "")
  return `${prefix}-${base.slice(0, 72) || Date.now()}`
}
