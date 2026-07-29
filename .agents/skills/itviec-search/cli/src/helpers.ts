export interface SearchOptions {
  query?: string
  locations?: string[]
  seniority?: string[]
  workModes?: string[]
  jobage?: number | null
  maxExperience?: number | null
  limit?: number
}

export interface JobCard {
  id: string
  source: "itviec"
  title: string | null
  company: string | null
  location: string | null
  date: string | null
  deadline: null
  url: string
  discoveredAt: string
  verificationStatus: "unverified_lead"
  requiresVerification: true
}

export interface DetailResult {
  contractVersion: "1"
  source: "itviec"
  status:
    | "verified_active"
    | "insufficient_detail"
    | "expired"
    | "removed"
    | "restricted"
    | "failed"
  job: Record<string, unknown>
  verification: {
    status: DetailResult["status"]
    detailAccessible: boolean
    active: boolean | null
    descriptionSufficient: boolean
    experienceDecisionSupported: boolean
    checkedAt: string
    reasonCodes: string[]
  }
  warnings: string[]
}

export class PortalError extends Error {
  constructor(
    message: string,
    public readonly code: string,
  ) {
    super(message)
  }
}

function slug(value: string): string {
  return value
    .normalize("NFKD")
    .replace(/\p{Diacritic}/gu, "")
    .replace(/đ/gi, "d")
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9+#.]+/g, "-")
    .replace(/^-|-$/g, "")
}

function text(value: string): string {
  return value
    .replace(/<[^>]+>/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/&nbsp;/g, " ")
    .replace(/&#(\d+);/g, (_, code: string) => String.fromCodePoint(Number(code)))
    .replace(/\s+/g, " ")
    .trim()
}

function attribute(markup: string, name: string): string | null {
  return markup.match(new RegExp(`${name}=["']([^"']+)["']`, "i"))?.[1] ?? null
}

function citySlug(location: string | undefined): string | null {
  if (!location) return null
  const value = slug(location)
  if (/ho-chi-minh|hcm|sai-gon/.test(value)) return "ho-chi-minh-hcm"
  if (/ha-noi|hanoi/.test(value)) return "ha-noi"
  if (/da-nang/.test(value)) return "da-nang"
  return value || null
}

export function buildSearchUrl(options: SearchOptions): string {
  const keyword = slug(options.query || options.seniority?.[0] || "it")
  const city = citySlug(options.locations?.[0])
  const path = `/it-jobs/${encodeURIComponent(keyword)}${city ? `/${city}` : ""}`
  const params = new URLSearchParams()
  for (const mode of options.workModes ?? []) params.append("work_model", mode)
  return `https://itviec.com${path}${params.size ? `?${params}` : ""}`
}

function challengePage(html: string): boolean {
  return /(captcha|cloudflare|access denied|verify you are human|sign in to continue)/i.test(html)
}

function findJobPosting(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== "object") return null
  if (Array.isArray(value)) {
    for (const item of value) {
      const found = findJobPosting(item)
      if (found) return found
    }
    return null
  }
  const record = value as Record<string, unknown>
  const type = record["@type"]
  if (type === "JobPosting" || (Array.isArray(type) && type.includes("JobPosting"))) return record
  for (const nested of Object.values(record)) {
    const found = findJobPosting(nested)
    if (found) return found
  }
  return null
}

function structuredJob(html: string): Record<string, unknown> | null {
  for (const match of html.matchAll(
    /<script\b[^>]*type=["']application\/ld\+json["'][^>]*>([\s\S]*?)<\/script>/gi,
  )) {
    try {
      const found = findJobPosting(JSON.parse(match[1]))
      if (found) return found
    } catch {
      // Continue past unrelated malformed JSON-LD.
    }
  }
  return null
}

function organizationName(value: unknown): string | null {
  if (typeof value === "string") return value
  if (value && typeof value === "object") {
    const name = (value as Record<string, unknown>).name
    return typeof name === "string" ? name : null
  }
  return null
}

function locationName(value: unknown): string | null {
  const first = Array.isArray(value) ? value[0] : value
  if (!first || typeof first !== "object") return null
  const address = (first as Record<string, unknown>).address
  if (typeof address === "string") return address
  if (address && typeof address === "object") {
    const locality = (address as Record<string, unknown>).addressLocality
    return typeof locality === "string" ? locality : null
  }
  return null
}

function experienceRange(value: string): { min: number | null; max: number | null; evidence: string[] } {
  const values: number[] = []
  const evidence: string[] = []
  for (const match of value.matchAll(/(\d+(?:[.,]\d+)?)\s*(months?|years?|tháng|năm)/gi)) {
    const amount = Number(match[1].replace(",", "."))
    values.push(/month|tháng/i.test(match[2]) ? amount / 12 : amount)
    evidence.push(match[0])
  }
  if (/fresh graduates?|no experience required|sinh viên mới tốt nghiệp/i.test(value)) {
    values.push(0)
    evidence.push("fresh graduate/no experience accepted")
  }
  return {
    min: values.length ? Math.min(...values) : null,
    max: values.length ? Math.max(...values) : null,
    evidence,
  }
}

function skillsFrom(value: string): string[] {
  return ["Java", "ReactJS", "React", "VueJS", "Vue", "Spring", "JavaScript", "TypeScript"]
    .filter((skill) => new RegExp(`\\b${skill}\\b`, "i").test(value))
}

export function parseDetailPage(
  html: string,
  rawUrl: string,
  checkedAt = new Date().toISOString(),
): DetailResult {
  if (challengePage(html)) throw new PortalError("ITviec returned a challenge page", "RESTRICTED")
  const url = new URL(rawUrl)
  url.hash = ""
  url.search = ""
  const canonicalUrl = url.toString()
  const id = canonicalUrl.match(/-(\d{3,})\/?$/)?.[1] ?? null
  const closed = /(job is no longer available|job has expired|việc làm đã hết hạn)/i.test(html)
  const structured = structuredJob(html)
  const description =
    typeof structured?.description === "string" ? text(structured.description) : ""
  const range = experienceRange(description)
  const title = typeof structured?.title === "string" ? structured.title : null
  const sufficient = description.length >= 30
  const status: DetailResult["status"] = closed
    ? "expired"
    : sufficient
      ? "verified_active"
      : "insufficient_detail"
  const combined = `${title ?? ""} ${description}`
  const workMode = /hybrid/i.test(combined)
    ? "hybrid"
    : /\bremote\b/i.test(combined)
      ? "remote"
      : /\boffice\b|on-?site/i.test(combined)
        ? "onsite"
        : null
  return {
    contractVersion: "1",
    source: "itviec",
    status,
    job: {
      id,
      source: "itviec",
      url: canonicalUrl,
      title,
      company: organizationName(structured?.hiringOrganization),
      location: locationName(structured?.jobLocation),
      descriptionSummary: description ? description.slice(0, 1_500) : null,
      skills: skillsFrom(description),
      experienceEvidence: range.evidence.map((value) => ({
        value,
        evidenceKind: "detail_text",
      })),
      minExperienceYears: range.min,
      maxExperienceYears: range.max,
      seniority: /fresher/i.test(combined)
        ? "fresher"
        : /junior/i.test(combined)
          ? "junior"
          : null,
      employmentType:
        typeof structured?.employmentType === "string" ? structured.employmentType : null,
      workMode,
      postedAt:
        typeof structured?.datePosted === "string" ? structured.datePosted.slice(0, 10) : null,
      deadline: null,
      salary: null,
      applyUrl: null,
      language: "en",
      activeEvidence: status === "verified_active" ? ["accessible sufficient public detail"] : [],
      closedEvidence: closed ? ["explicit expired marker"] : [],
      verifiedAt: checkedAt,
    },
    verification: {
      status,
      detailAccessible: true,
      active: status === "verified_active" ? true : status === "expired" ? false : null,
      descriptionSufficient: sufficient,
      experienceDecisionSupported: range.evidence.length > 0,
      checkedAt,
      reasonCodes:
        status === "verified_active"
          ? []
          : status === "expired"
            ? ["CLOSED_MARKER"]
            : ["NO_REQUIREMENTS"],
    },
    warnings: ["Salary and apply actions may require sign-in and remain unknown."],
  }
}

export function parseSearchPage(
  html: string,
  discoveredAt = new Date().toISOString(),
): JobCard[] {
  if (challengePage(html)) throw new PortalError("ITviec returned a challenge page", "RESTRICTED")
  if (/(no jobs found|no-jobs|không tìm thấy)/i.test(html)) return []
  const cards: JobCard[] = []
  const seen = new Set<string>()
  for (const match of html.matchAll(/<article\b([^>]*)>([\s\S]*?)<\/article>/gi)) {
    const attrs = match[1]
    const body = match[2]
    const anchor = body.match(
      /<a\b[^>]*href=["']([^"']*\/it-jobs\/[^"']*-(\d{3,})[^"']*)["'][^>]*>([\s\S]*?)<\/a>/i,
    )
    const id = attribute(attrs, "data-job-id") ?? anchor?.[2] ?? null
    if (!id || !anchor || seen.has(id)) continue
    seen.add(id)
    cards.push({
      id,
      source: "itviec",
      title: text(anchor[3]) || null,
      company: attribute(attrs, "data-company"),
      location: attribute(attrs, "data-location"),
      date: body.match(/<time\b[^>]*datetime=["']([^"']+)["']/i)?.[1] ?? null,
      deadline: null,
      url: new URL(anchor[1], "https://itviec.com").toString().split(/[?#]/)[0],
      discoveredAt,
      verificationStatus: "unverified_lead",
      requiresVerification: true,
    })
  }
  if (cards.length === 0) {
    for (const match of html.matchAll(
      /<a\b[^>]*href=["']([^"']*\/it-jobs\/[^"']*-(\d{3,})[^"']*)["'][^>]*>([\s\S]*?)<\/a>/gi,
    )) {
      if (seen.has(match[2])) continue
      const title = text(match[3])
      if (!title) continue
      seen.add(match[2])
      cards.push({
        id: match[2],
        source: "itviec",
        title,
        company: null,
        location: null,
        date: null,
        deadline: null,
        url: new URL(match[1], "https://itviec.com").toString().split(/[?#]/)[0],
        discoveredAt,
        verificationStatus: "unverified_lead",
        requiresVerification: true,
      })
    }
  }
  if (cards.length === 0 && !/(it-jobs|jobs-list|search-results)/i.test(html)) {
    throw new PortalError("Unrecognized ITviec search page", "PARSE_FAILED")
  }
  return cards
}

export async function publicFetch(url: string, attempts = 2): Promise<string> {
  let lastError: unknown
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    try {
      const response = await fetch(url, {
        headers: {
          accept: "text/html,application/xhtml+xml",
          "user-agent": "Mozilla/5.0 (compatible; personal-job-search/1.0)",
        },
        signal: AbortSignal.timeout(12_000),
        redirect: "follow",
      })
      if (response.status === 401 || response.status === 403) {
        throw new PortalError(`ITviec returned HTTP ${response.status}`, "RESTRICTED")
      }
      if (response.status === 404) throw new PortalError("ITviec job not found", "NOT_FOUND")
      if (response.status === 429 || response.status >= 500) {
        lastError = new PortalError(`ITviec returned HTTP ${response.status}`, "UNAVAILABLE")
        if (attempt + 1 < attempts) {
          await Bun.sleep(150 * 2 ** attempt + Math.floor(Math.random() * 50))
          continue
        }
        throw lastError
      }
      if (!response.ok) throw new PortalError(`ITviec returned HTTP ${response.status}`, "SEARCH_FAILED")
      const html = await response.text()
      if (challengePage(html)) throw new PortalError("ITviec returned a challenge page", "RESTRICTED")
      return html
    } catch (error) {
      if (error instanceof PortalError) throw error
      lastError = error
      if (attempt + 1 < attempts) {
        await Bun.sleep(150 * 2 ** attempt)
        continue
      }
    }
  }
  throw new PortalError(
    lastError instanceof Error ? lastError.message : "ITviec request failed",
    "UNAVAILABLE",
  )
}

export function writeError(message: string, code: string): void {
  process.stderr.write(JSON.stringify({ error: message, code, source: "itviec" }) + "\n")
}
