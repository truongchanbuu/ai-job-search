export interface GoogleResult {
  id: string
  source: "google"
  resultType: "public_page_discovery"
  requiresVerification: true
  title: string
  snippet: string | null
  displayLink: string | null
  url: string
  date: string | null
}

export interface QueryLink {
  source: "google"
  resultType: "manual_public_page_query"
  requiresVerification: true
  query: string
  url: string
}

export function writeError(error: string, code: string): void {
  process.stderr.write(JSON.stringify({ error, code }) + "\n")
}

export function parseRepeatable(value: string | boolean | string[] | undefined): string[] {
  if (value === undefined || typeof value === "boolean") return []
  return Array.isArray(value) ? value : [value]
}

export function buildGoogleQuery(opts: {
  query?: string
  location?: string
  jobage?: number
  sites?: string[]
}): string {
  const parts: string[] = []
  if (opts.query) parts.push(opts.query)
  if (opts.location) parts.push(opts.location)
  parts.push("(job OR jobs OR career OR careers OR hiring OR apply)")
  if (opts.jobage && opts.jobage > 0 && opts.jobage < 9999) {
    parts.push(`posted within ${opts.jobage} days`)
  }
  for (const site of opts.sites ?? []) {
    const cleaned = site.replace(/^https?:\/\//, "").replace(/\/.*$/, "")
    if (cleaned) parts.push(`site:${cleaned}`)
  }
  return parts.join(" ").replace(/\s+/g, " ").trim()
}

export function buildQueryLinks(opts: {
  query?: string
  location?: string
  jobage?: number
  sites?: string[]
}): QueryLink[] {
  const baseSites = opts.sites && opts.sites.length > 0 ? opts.sites : [
    "linkedin.com/jobs",
    "itviec.com",
    "topdev.vn",
    "vietnamworks.com",
    "careers.smartrecruiters.com",
    "greenhouse.io",
    "lever.co",
  ]

  const broad = buildGoogleQuery({ ...opts, sites: [] })
  const links: QueryLink[] = [{
    source: "google",
    resultType: "manual_public_page_query",
    requiresVerification: true,
    query: broad,
    url: googleUrl(broad),
  }]

  for (const site of baseSites) {
    const query = buildGoogleQuery({ ...opts, sites: [site] })
    links.push({
      source: "google",
      resultType: "manual_public_page_query",
      requiresVerification: true,
      query,
      url: googleUrl(query),
    })
  }

  return links
}

export function googleUrl(query: string): string {
  return `https://www.google.com/search?q=${encodeURIComponent(query)}`
}

export function isGoogleJobsUiUrl(url: string): boolean {
  try {
    const parsed = new URL(url)
    const host = parsed.hostname.toLowerCase()
    const path = parsed.pathname.replace(/\/+$/, "")
    if (!/(^|\.)google\.[a-z.]+$/.test(host)) return false
    if (path !== "/search") return false
    return parsed.searchParams.get("ibp") === "htl;jobs" || parsed.search.toLowerCase().includes("htl%3bjobs")
  } catch {
    return false
  }
}

export function normalizeResult(item: {
  title?: string
  link?: string
  snippet?: string
  displayLink?: string
  pagemap?: { metatags?: Array<Record<string, string>> }
}, index: number): GoogleResult | null {
  if (!item.link || !item.title) return null
  if (isGoogleJobsUiUrl(item.link)) return null
  const meta = item.pagemap?.metatags?.[0] ?? {}
  const date =
    meta["article:published_time"] ??
    meta["date"] ??
    meta["publishdate"] ??
    meta["pubdate"] ??
    null

  return {
    id: String(index + 1),
    source: "google",
    resultType: "public_page_discovery",
    requiresVerification: true,
    title: item.title,
    snippet: item.snippet ?? null,
    displayLink: item.displayLink ?? null,
    url: item.link,
    date,
  }
}

export function renderTable(rows: Array<GoogleResult | QueryLink>): string {
  if (rows.length === 0) return "No results."
  if ("title" in rows[0]) {
    const header = "ID".padEnd(4) + " " + "TITLE".padEnd(48) + " " + "DOMAIN".padEnd(24) + " URL"
    const body = (rows as GoogleResult[]).map((r) => {
      return `${r.id.padEnd(4)} ${r.title.slice(0, 48).padEnd(48)} ${(r.displayLink ?? "-").slice(0, 24).padEnd(24)} ${r.url}`
    })
    return [header, "-".repeat(header.length), ...body].join("\n")
  }

  const header = "QUERY".padEnd(80) + " URL"
  const body = (rows as QueryLink[]).map((r) => `${r.query.slice(0, 80).padEnd(80)} ${r.url}`)
  return [header, "-".repeat(header.length), ...body].join("\n")
}
