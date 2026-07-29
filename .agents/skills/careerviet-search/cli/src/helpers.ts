export interface ManualSearchOptions {
  query?: string
  locations?: string[]
  maxExperience?: number | null
}

function slug(value: string): string {
  return value
    .normalize("NFKD")
    .replace(/\p{Diacritic}/gu, "")
    .replace(/đ/gi, "d")
    .toLowerCase()
    .replace(/[^a-z0-9+#.]+/g, "-")
    .replace(/^-|-$/g, "")
}

export function buildSearchEnvelope(options: ManualSearchOptions) {
  const query = slug(options.query || "viec-lam")
  const experience = options.maxExperience != null && options.maxExperience <= 1 ? "-kn3" : ""
  const manualReviewUrl = `https://careerviet.vn/viec-lam/${encodeURIComponent(query)}${experience}-vi.html`
  return {
    contractVersion: "1",
    source: "careerviet",
    accessMode: "manual_only",
    status: "manual_required",
    query: options,
    appliedConstraints: {
      query: options.query ?? null,
      locations: options.locations ?? [],
      maxExperience: options.maxExperience ?? null,
    },
    unsupportedConstraints: ["automatedSearch"],
    manualReviewUrl,
    meta: { count: 0, page: 1, total: 0 },
    results: [],
    warnings: [
      "CareerViet terms require ordinary browser/manual search; no unattended request was made.",
    ],
  } as const
}

export function normalizePublicReference(input: string): string | null {
  try {
    const url = new URL(input)
    if (!/(^|\.)careerviet\.vn$/i.test(url.hostname)) return null
    url.hash = ""
    for (const key of [...url.searchParams.keys()]) {
      if (key.startsWith("utm_") || key === "ref") url.searchParams.delete(key)
    }
    return url.toString()
  } catch {
    return null
  }
}

export function buildDetailEnvelope(input: string) {
  const url = normalizePublicReference(input)
  if (!url) throw new Error("detail requires a public CareerViet URL")
  const id = url.match(/\.([A-F0-9]{6,})\.html/i)?.[1] ?? null
  const checkedAt = new Date().toISOString()
  return {
    contractVersion: "1",
    source: "careerviet",
    status: "insufficient_detail",
    job: {
      id,
      source: "careerviet",
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
      status: "insufficient_detail",
      detailAccessible: false,
      active: null,
      descriptionSufficient: false,
      experienceDecisionSupported: false,
      checkedAt,
      reasonCodes: ["MANUAL_BROWSER_REVIEW_REQUIRED"],
    },
    warnings: [
      "The reference was normalized without unattended retrieval; provide the posting text for evidence review.",
    ],
  } as const
}

export function writeError(message: string, code: string): void {
  process.stderr.write(JSON.stringify({ error: message, code, source: "careerviet" }) + "\n")
}
