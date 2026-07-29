export interface ManualSearchOptions {
  query?: string
  locations?: string[]
  maxExperience?: number | null
}

export function buildSearchEnvelope(options: ManualSearchOptions) {
  const params = new URLSearchParams()
  if (options.query) params.set("q", options.query)
  const manualReviewUrl = `https://vieclam24h.vn/tim-kiem-viec-lam-nhanh${params.size ? `?${params}` : ""}`
  return {
    contractVersion: "1",
    source: "vieclam24h",
    accessMode: "restricted",
    status: "manual_required",
    query: options,
    appliedConstraints: { query: options.query ?? null },
    unsupportedConstraints: ["automatedSearch", "verifiedPublicDetail"],
    manualReviewUrl,
    meta: { count: 0, page: 1, total: 0 },
    results: [],
    warnings: [
      "Ordinary unauthenticated access returned 403 during the 2026-07-28 review; no retry or bypass was attempted.",
    ],
  } as const
}

export function normalizePublicReference(input: string): string | null {
  try {
    const url = new URL(input)
    if (!/(^|\.)vieclam24h\.vn$/i.test(url.hostname)) return null
    url.hash = ""
    return url.toString()
  } catch {
    return null
  }
}

export function buildDetailEnvelope(input: string) {
  const url = normalizePublicReference(input)
  if (!url) throw new Error("detail requires a public Vieclam24h URL")
  const checkedAt = new Date().toISOString()
  return {
    contractVersion: "1",
    source: "vieclam24h",
    status: "restricted",
    job: {
      id: null,
      source: "vieclam24h",
      url,
      title: null,
      company: null,
      location: null,
      descriptionSummary: null,
      skills: [],
      experienceEvidence: [],
      minExperienceYears: null,
      maxExperienceYears: null,
      seniority: null,
      employmentType: null,
      workMode: null,
      postedAt: null,
      deadline: null,
      salary: null,
      applyUrl: null,
      language: null,
      activeEvidence: [],
      closedEvidence: [],
      verifiedAt: null,
    },
    verification: {
      status: "restricted",
      detailAccessible: false,
      active: null,
      descriptionSufficient: false,
      experienceDecisionSupported: false,
      checkedAt,
      reasonCodes: ["HTTP_403", "USER_SUPPLIED_TEXT_REQUIRED"],
    },
    warnings: ["No protected request was made; provide posting text for manual evidence review."],
  } as const
}

export function writeError(message: string, code: string): void {
  process.stderr.write(JSON.stringify({ error: message, code, source: "vieclam24h" }) + "\n")
}
