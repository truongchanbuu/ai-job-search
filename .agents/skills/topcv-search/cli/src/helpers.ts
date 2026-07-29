export interface SearchOptions {
  query?: string
  locations?: string[]
  maxExperience?: number | null
  seniority?: string[]
  workModes?: string[]
  jobage?: number | null
  limit?: number
}

export interface JobCard {
  id: string
  source: "topcv"
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
  source: "topcv"
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

export function decodeHtml(value: string): string {
  const entities: Record<string, string> = {
    amp: "&",
    quot: '"',
    apos: "'",
    lt: "<",
    gt: ">",
    nbsp: " ",
  }
  return value
    .replace(/&#(\d+);/g, (_, code: string) => String.fromCodePoint(Number(code)))
    .replace(/&#x([\da-f]+);/gi, (_, code: string) =>
      String.fromCodePoint(Number.parseInt(code, 16)),
    )
    .replace(/&([a-z]+);/gi, (full, name: string) => entities[name.toLowerCase()] ?? full)
}

export function text(value: string): string {
  return decodeHtml(value.replace(/<[^>]+>/g, " ").replace(/\s+/g, " ").trim())
}

function attribute(markup: string, name: string): string | null {
  const match = markup.match(new RegExp(`${name}=["']([^"']+)["']`, "i"))
  return match ? decodeHtml(match[1]) : null
}

export function buildSearchUrl(options: SearchOptions): string {
  const slug = options.query
    ?.normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .replace(/đ/gi, "d")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "")
  return slug
    ? `https://www.topcv.vn/tim-viec-lam-${encodeURIComponent(slug)}`
    : "https://www.topcv.vn/tim-viec-lam"
}

function challengePage(html: string): boolean {
  return /(captcha|cloudflare|access denied|verify you are human|đăng nhập để tiếp tục)/i.test(
    html,
  )
}

function canonicalDetailUrl(raw: string): string {
  const url = new URL(raw)
  url.hash = ""
  for (const key of [...url.searchParams.keys()]) {
    if (key.startsWith("utm_") || ["ref", "source", "tracking"].includes(key)) {
      url.searchParams.delete(key)
    }
  }
  return url.toString()
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
  if (type === "JobPosting" || (Array.isArray(type) && type.includes("JobPosting"))) {
    return record
  }
  for (const nested of Object.values(record)) {
    const found = findJobPosting(nested)
    if (found) return found
  }
  return null
}

function jobPostingFromHtml(html: string): Record<string, unknown> | null {
  for (const match of html.matchAll(
    /<script\b[^>]*type=["']application\/ld\+json["'][^>]*>([\s\S]*?)<\/script>/gi,
  )) {
    try {
      const found = findJobPosting(JSON.parse(match[1]))
      if (found) return found
    } catch {
      // Ignore unrelated malformed structured data and continue.
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
  const record = first as Record<string, unknown>
  const address = record.address
  if (typeof address === "string") return address
  if (address && typeof address === "object") {
    const addressRecord = address as Record<string, unknown>
    const parts = [addressRecord.addressLocality, addressRecord.addressRegion]
      .filter((part): part is string => typeof part === "string" && part.length > 0)
    return parts.length ? parts.join(", ") : null
  }
  return null
}

function experienceRange(value: string): {
  min: number | null
  max: number | null
  evidence: string[]
} {
  const values: number[] = []
  const evidence: string[] = []
  for (const match of value.matchAll(
    /(\d+(?:[.,]\d+)?)\s*(tháng|năm|months?|years?)/gi,
  )) {
    const amount = Number(match[1].replace(",", "."))
    const years = /tháng|month/i.test(match[2]) ? amount / 12 : amount
    values.push(years)
    evidence.push(match[0])
  }
  if (/fresh graduates?|sinh viên mới tốt nghiệp|không yêu cầu kinh nghiệm/i.test(value)) {
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
  const known = ["Java", "ReactJS", "React", "VueJS", "Vue", "Spring", "JavaScript", "TypeScript"]
  return known.filter((skill) => new RegExp(`\\b${skill}\\b`, "i").test(value))
}

export function parseDetailPage(
  html: string,
  rawUrl: string,
  checkedAt = new Date().toISOString(),
): DetailResult {
  if (challengePage(html)) throw new PortalError("TopCV returned a challenge page", "RESTRICTED")
  const canonicalUrl = canonicalDetailUrl(rawUrl)
  const id = canonicalUrl.match(/\/(\d+)\.html(?:$|\?)/)?.[1] ?? null
  const closed = /(việc làm đã hết hạn|tin tuyển dụng đã đóng|job has expired)/i.test(html)
  const structured = jobPostingFromHtml(html)
  if (closed && !structured) {
    return {
      contractVersion: "1",
      source: "topcv",
      status: "expired",
      job: {
        id,
        source: "topcv",
        url: canonicalUrl,
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
        language: "vi",
        activeEvidence: [],
        closedEvidence: ["explicit expired marker"],
        verifiedAt: checkedAt,
      },
      verification: {
        status: "expired",
        detailAccessible: true,
        active: false,
        descriptionSufficient: false,
        experienceDecisionSupported: false,
        checkedAt,
        reasonCodes: ["CLOSED_MARKER"],
      },
      warnings: [],
    }
  }
  const description = typeof structured?.description === "string" ? text(structured.description) : ""
  const range = experienceRange(description)
  const deadline = typeof structured?.validThrough === "string" ? structured.validThrough.slice(0, 10) : null
  const deadlineExpired =
    deadline != null && new Date(`${deadline}T23:59:59Z`).getTime() < new Date(checkedAt).getTime()
  const sufficient = description.length >= 30
  const status: DetailResult["status"] =
    closed || deadlineExpired ? "expired" : sufficient ? "verified_active" : "insufficient_detail"
  const title = typeof structured?.title === "string" ? structured.title : null
  const evidence = range.evidence.map((value) => ({
    value,
    evidenceKind: "detail_text",
  }))
  return {
    contractVersion: "1",
    source: "topcv",
    status,
    job: {
      id,
      source: "topcv",
      url: canonicalUrl,
      title,
      company: organizationName(structured?.hiringOrganization),
      location: locationName(structured?.jobLocation),
      descriptionSummary: description ? description.slice(0, 1_500) : null,
      skills: skillsFrom(description),
      experienceEvidence: evidence,
      minExperienceYears: range.min,
      maxExperienceYears: range.max,
      seniority: /fresher/i.test(`${title} ${description}`)
        ? "fresher"
        : /junior/i.test(`${title} ${description}`)
          ? "junior"
          : null,
      employmentType:
        typeof structured?.employmentType === "string" ? structured.employmentType : null,
      workMode: null,
      postedAt:
        typeof structured?.datePosted === "string" ? structured.datePosted.slice(0, 10) : null,
      deadline,
      salary: null,
      applyUrl: null,
      language: "vi",
      activeEvidence: status === "verified_active" ? ["accessible sufficient public detail"] : [],
      closedEvidence: status === "expired" ? ["expired deadline or closed marker"] : [],
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
    warnings: [],
  }
}

export function parseSearchPage(
  html: string,
  discoveredAt = new Date().toISOString(),
): JobCard[] {
  if (challengePage(html)) throw new PortalError("TopCV returned a challenge page", "RESTRICTED")
  if (/(không tìm thấy việc làm|no-result|no jobs found)/i.test(html)) return []
  const cards: JobCard[] = []
  const seen = new Set<string>()
  const articlePattern = /<article\b([^>]*)>([\s\S]*?)<\/article>/gi
  for (const match of html.matchAll(articlePattern)) {
    const attrs = match[1]
    const body = match[2]
    const anchor = body.match(
      /<a\b[^>]*href=["']([^"']*\/viec-lam\/[^"']*\/(\d+)\.html[^"']*)["'][^>]*>([\s\S]*?)<\/a>/i,
    )
    const id = attribute(attrs, "data-job-id") ?? anchor?.[2] ?? null
    if (!id || !anchor || seen.has(id)) continue
    seen.add(id)
    const href = new URL(anchor[1], "https://www.topcv.vn").toString()
    const time = body.match(/<time\b[^>]*datetime=["']([^"']+)["']/i)?.[1] ?? null
    cards.push({
      id,
      source: "topcv",
      title: text(anchor[3]) || null,
      company: attribute(attrs, "data-company"),
      location: attribute(attrs, "data-location"),
      date: time,
      deadline: null,
      url: href.split(/[?#]/)[0],
      discoveredAt,
      verificationStatus: "unverified_lead",
      requiresVerification: true,
    })
  }
  if (cards.length === 0) {
    const anchorPattern =
      /<a\b[^>]*href=["']([^"']*\/viec-lam\/[^"']*\/(\d+)\.html[^"']*)["'][^>]*>([\s\S]*?)<\/a>/gi
    for (const match of html.matchAll(anchorPattern)) {
      if (seen.has(match[2])) continue
      const title = text(match[3])
      if (!title) continue
      seen.add(match[2])
      cards.push({
        id: match[2],
        source: "topcv",
        title,
        company: null,
        location: null,
        date: null,
        deadline: null,
        url: new URL(match[1], "https://www.topcv.vn").toString().split(/[?#]/)[0],
        discoveredAt,
        verificationStatus: "unverified_lead",
        requiresVerification: true,
      })
    }
  }
  if (cards.length === 0 && !/(tim-viec-lam|job-list|search-job)/i.test(html)) {
    throw new PortalError("Unrecognized TopCV search page", "PARSE_FAILED")
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
        throw new PortalError(`TopCV returned HTTP ${response.status}`, "RESTRICTED")
      }
      if (response.status === 404) throw new PortalError("TopCV job not found", "NOT_FOUND")
      if (response.status === 429 || response.status >= 500) {
        lastError = new PortalError(`TopCV returned HTTP ${response.status}`, "UNAVAILABLE")
        if (attempt + 1 < attempts) {
          await Bun.sleep(150 * 2 ** attempt + Math.floor(Math.random() * 50))
          continue
        }
        throw lastError
      }
      if (!response.ok) throw new PortalError(`TopCV returned HTTP ${response.status}`, "SEARCH_FAILED")
      const html = await response.text()
      if (challengePage(html)) throw new PortalError("TopCV returned a challenge page", "RESTRICTED")
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
    lastError instanceof Error ? lastError.message : "TopCV request failed",
    "UNAVAILABLE",
  )
}

export function writeError(message: string, code: string): void {
  process.stderr.write(JSON.stringify({ error: message, code, source: "topcv" }) + "\n")
}
