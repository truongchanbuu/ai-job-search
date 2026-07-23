import type { FreelanceOpportunity, SourceReference, SourceType } from "./models.js"
import { nowIso, stableId } from "./models.js"
import { detectRiskFlags } from "./risk.js"

export interface RawOpportunity {
  title?: string
  clientOrSource?: string
  platform?: string
  url?: string | null
  budget?: string | null
  timeline?: string | null
  description?: string | null
  skills?: string[]
  language?: "en" | "vi" | "mixed" | "unknown"
  requiresVerification?: boolean
  source?: string
}

export function canonicalKey(input: Pick<FreelanceOpportunity, "title" | "clientOrSource" | "platform" | "url" | "budget">): string {
  return [input.platform, input.title, input.clientOrSource, input.budget, input.url]
    .filter(Boolean)
    .join("|")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "")
}

export function normalizeOpportunity(raw: RawOpportunity, source = raw.source ?? raw.platform ?? "user_supplied"): FreelanceOpportunity {
  const title = raw.title?.trim() || "Untitled freelance opportunity"
  const platform = (raw.platform ?? source) as SourceType | string
  const sourceRef: SourceReference = {
    source_id: source,
    url_or_identifier: raw.url ?? null,
    retrieved_at: nowIso(),
    raw_title: title,
    raw_client_or_source: raw.clientOrSource,
    raw_budget: raw.budget ?? undefined,
    status: raw.requiresVerification === false ? "retrieved" : "discovery_lead",
  }
  const opportunity: FreelanceOpportunity = {
    opportunityId: stableId("opp", [platform, title, raw.clientOrSource, raw.url]),
    canonicalKey: "",
    sourceOpportunityId: stableId("src", [platform, title, raw.url]),
    title,
    clientOrSource: raw.clientOrSource ?? null,
    platform,
    url: raw.url ?? null,
    budget: raw.budget ?? null,
    timeline: raw.timeline ?? null,
    description: raw.description ?? null,
    skills: raw.skills ?? extractSkills(`${title} ${raw.description ?? ""}`),
    language: raw.language ?? inferLanguage(`${title} ${raw.description ?? ""}`),
    sourceRefs: [sourceRef],
    riskFlags: [],
    discoveredAt: nowIso(),
    sourceStatus: sourceRef.status,
    requiresVerification: raw.requiresVerification ?? true,
  }
  opportunity.canonicalKey = canonicalKey(opportunity)
  opportunity.riskFlags = detectRiskFlags(opportunity)
  return opportunity
}

export function extractSkills(text: string): string[] {
  const known = ["flutter", "firebase", "react", "node", "typescript", "api", "stripe", "wordpress", "figma", "python", "automation"]
  const lower = text.toLowerCase()
  return known.filter((skill) => lower.includes(skill)).map((skill) => skill === "api" ? "API" : skill[0].toUpperCase() + skill.slice(1))
}

export function inferLanguage(text: string): "en" | "vi" | "mixed" | "unknown" {
  const hasVietnamese = /[ăâđêôơưáàảãạấầẩẫậắằẳẵặéèẻẽẹếềểễệíìỉĩịóòỏõọốồổỗộớờởỡợúùủũụứừửữựýỳỷỹỵ]/i.test(text)
  const hasAsciiWords = /[a-z]{3,}/i.test(text)
  if (hasVietnamese && hasAsciiWords) return "mixed"
  if (hasVietnamese) return "vi"
  if (hasAsciiWords) return "en"
  return "unknown"
}
