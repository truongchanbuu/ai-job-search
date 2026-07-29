import { mkdir, readFile, writeFile } from "node:fs/promises"
import { dirname, resolve } from "node:path"
import type {
  CanonicalJobListing,
  PortalSearchRun,
  SeenJobState,
} from "./contracts.js"

export const REPOSITORY_ROOT = resolve(import.meta.dir, "../../../../..")
export const DEFAULT_SEEN_PATH = resolve(REPOSITORY_ROOT, "job_scraper/seen_jobs.json")
export const DEFAULT_TRACKER_PATH = resolve(REPOSITORY_ROOT, "job_search_tracker.csv")

export function defaultSessionPath(runId: string): string {
  return resolve(REPOSITORY_ROOT, `job_scraper/sessions/${runId}.json`)
}

export async function saveSession(run: PortalSearchRun, path: string): Promise<void> {
  await mkdir(dirname(path), { recursive: true })
  await writeFile(path, JSON.stringify(run, null, 2) + "\n", "utf8")
}

async function existingSeen(path: string): Promise<SeenJobState[]> {
  try {
    const parsed = JSON.parse(await readFile(path, "utf8")) as unknown
    if (Array.isArray(parsed)) return parsed as SeenJobState[]
    if (parsed && typeof parsed === "object") {
      const records = (parsed as { records?: unknown; items?: unknown }).records ??
        (parsed as { items?: unknown }).items
      return Array.isArray(records) ? (records as SeenJobState[]) : []
    }
    return []
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return []
    throw error
  }
}

export async function upsertSeenJobs(
  path: string,
  updates: SeenJobState[],
): Promise<void> {
  const records = await existingSeen(path)
  const byId = new Map(records.map((record) => [record.jobId, record]))
  for (const update of updates) {
    const prior = byId.get(update.jobId)
    byId.set(update.jobId, {
      ...prior,
      ...update,
      firstSeenAt: prior?.firstSeenAt ?? update.firstSeenAt,
      sourceRefs: [
        ...new Map(
          [...(prior?.sourceRefs ?? []), ...update.sourceRefs].map((reference) => [
            `${reference.source}|${reference.id ?? ""}|${reference.url}`,
            reference,
          ]),
        ).values(),
      ],
    })
  }
  await mkdir(dirname(path), { recursive: true })
  await writeFile(
    path,
    JSON.stringify({ contractVersion: "1", records: [...byId.values()] }, null, 2) + "\n",
    "utf8",
  )
}

export function seenRecordsFromJobs(
  jobs: CanonicalJobListing[],
  observedAt: string,
): SeenJobState[] {
  return jobs.map((job) => ({
    jobId: job.jobId,
    canonicalKey: job.canonicalKey,
    sourceRefs: job.sourceListings.map((listing) => ({
      source: listing.sourceId,
      id: listing.sourceJobId,
      url: listing.canonicalUrl,
    })),
    state:
      job.existingSeenState === "dismissed" ||
      job.existingSeenState === "shortlisted" ||
      job.existingSeenState === "seen"
        ? job.existingSeenState
        : "seen",
    firstSeenAt: observedAt,
    lastSeenAt: observedAt,
    notes: null,
  }))
}
