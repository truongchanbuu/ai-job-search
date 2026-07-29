import { readFile } from "node:fs/promises"
import type {
  CanonicalJobListing,
  SeenJobState,
  SourceReference,
} from "./contracts.js"
import { canonicalizeUrl, normalizeIdentityText } from "./normalize.js"

export interface ApplicationHistoryLink {
  applicationId: string
  status: string
  sourceRefs: SourceReference[]
  employer: string | null
  title: string | null
  location: string | null
  appliedAt: string | null
}

async function readable(path: string): Promise<string | null> {
  try {
    return await readFile(path, "utf8")
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return null
    throw error
  }
}

export async function loadSeenJobs(path: string): Promise<SeenJobState[]> {
  const text = await readable(path)
  if (!text) return []
  const parsed = JSON.parse(text) as unknown
  const records = Array.isArray(parsed)
    ? parsed
    : parsed && typeof parsed === "object"
      ? (parsed as { records?: unknown; items?: unknown }).records ??
        (parsed as { items?: unknown }).items ??
        []
      : []
  return Array.isArray(records) ? (records as SeenJobState[]) : []
}

function csvRows(text: string): string[][] {
  const rows: string[][] = []
  let row: string[] = []
  let cell = ""
  let quoted = false
  for (let index = 0; index < text.length; index += 1) {
    const char = text[index]
    if (char === '"') {
      if (quoted && text[index + 1] === '"') {
        cell += '"'
        index += 1
      } else quoted = !quoted
    } else if (char === "," && !quoted) {
      row.push(cell)
      cell = ""
    } else if ((char === "\n" || char === "\r") && !quoted) {
      if (char === "\r" && text[index + 1] === "\n") index += 1
      row.push(cell)
      if (row.some((value) => value.length > 0)) rows.push(row)
      row = []
      cell = ""
    } else cell += char
  }
  if (cell || row.length) {
    row.push(cell)
    rows.push(row)
  }
  return rows
}

export async function loadTracker(path: string): Promise<ApplicationHistoryLink[]> {
  const text = await readable(path)
  if (!text) return []
  const rows = csvRows(text)
  if (rows.length < 2) return []
  const headers = rows[0].map((header) =>
    header.trim().toLowerCase().replace(/[\s-]+/g, "_"),
  )
  return rows.slice(1).map((row, index) => {
    const record = Object.fromEntries(headers.map((header, column) => [header, row[column] ?? ""]))
    const sourceUrl = record.source_url || record.url || record.job_url
    return {
      applicationId: record.application_id || record.id || `row_${index + 2}`,
      status: record.status || "unknown",
      sourceRefs: sourceUrl
        ? [{ source: record.source || "tracker", id: null, url: canonicalizeUrl(sourceUrl) }]
        : [],
      employer: record.employer || record.company || null,
      title: record.title || record.role || null,
      location: record.location || null,
      appliedAt: record.applied_at || record.date || null,
    }
  })
}

function identity(
  employer: string | null,
  title: string | null,
  location: string | null,
): string {
  return [
    normalizeIdentityText(employer),
    normalizeIdentityText(title),
    normalizeIdentityText(location),
  ].join("|")
}

export function attachHistory(
  jobs: CanonicalJobListing[],
  seen: SeenJobState[],
  tracker: ApplicationHistoryLink[],
): CanonicalJobListing[] {
  return jobs.map((job) => {
    const urls = new Set(job.sourceListings.map((listing) => canonicalizeUrl(listing.canonicalUrl)))
    const seenMatch = seen.find(
      (record) =>
        record.jobId === job.jobId ||
        record.canonicalKey === job.canonicalKey ||
        record.sourceRefs.some((reference) => urls.has(canonicalizeUrl(reference.url))),
    )
    const jobIdentity = identity(job.employer, job.title, job.location)
    const trackerMatch = tracker.find(
      (record) =>
        record.sourceRefs.some((reference) => urls.has(canonicalizeUrl(reference.url))) ||
        identity(record.employer, record.title, record.location) === jobIdentity,
    )
    return {
      ...job,
      existingSeenState: seenMatch?.state ?? null,
      existingApplicationStatus: trackerMatch?.status ?? null,
    }
  })
}
