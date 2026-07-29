import type { ParsedFlags } from "../cli.js"
import {
  CONTRACT_VERSION,
  type AdapterSearchEnvelope,
  type PortalRunResult,
  type PortalRunStatus,
  type PortalSearchRun,
  type PortalSource,
  type SearchCriteria,
  type SourceCoverageSummary,
} from "../contracts.js"
import { deriveCoverage } from "../coverage.js"
import { deduplicateListings } from "../deduplicate.js"
import { evaluateFit } from "../fit.js"
import { attachHistory, loadSeenJobs, loadTracker } from "../history.js"
import {
  failedRun,
  normalizeAdapterDetail,
  normalizeAdapterSearch,
} from "../normalize.js"
import { findPortal, PORTAL_REGISTRY } from "../portal-registry.js"
import { parseProcessJson, runPortalProcess } from "../run-portal.js"
import { renderRun } from "../render.js"
import {
  DEFAULT_SEEN_PATH,
  DEFAULT_TRACKER_PATH,
  REPOSITORY_ROOT,
  defaultSessionPath,
  saveSession,
  seenRecordsFromJobs,
  upsertSeenJobs,
} from "../storage.js"
import { verifyListing } from "../verify.js"
import { resolve } from "node:path"

export interface SearchCommandOptions {
  flags: ParsedFlags
  criteria: SearchCriteria
}

export type PortalExecutor = (
  source: PortalSource,
  args: string[],
) => Promise<unknown>

class PortalExecutionError extends Error {
  constructor(
    message: string,
    public readonly payload: string,
    public readonly timedOut: boolean,
  ) {
    super(message)
  }
}

function list(value: string | string[] | boolean | undefined): string[] {
  if (Array.isArray(value)) return value
  if (typeof value === "string") return [value]
  return []
}

export function buildAdapterArgs(
  criteria: SearchCriteria,
  _source: PortalSource,
): string[] {
  const args = ["search"]
  if (criteria.queryTerms.length || criteria.skills.length) {
    args.push(
      "--query",
      [...criteria.queryTerms, ...criteria.skills]
        .filter((value, index, all) => value && all.indexOf(value) === index)
        .join(" "),
    )
  }
  for (const location of criteria.locations) args.push("--location", location)
  if (criteria.maxExperienceYears != null) {
    args.push("--max-experience", String(criteria.maxExperienceYears))
  }
  for (const seniority of criteria.seniority) args.push("--seniority", seniority)
  if (criteria.postedWithinDays != null) {
    args.push("--jobage", String(criteria.postedWithinDays))
  }
  for (const mode of criteria.workModes) args.push("--work-mode", mode)
  for (const type of criteria.employmentTypes) args.push("--employment-type", type)
  args.push("--limit", String(criteria.limitPerSource), "--format", "json")
  return args
}

async function defaultExecutor(source: PortalSource, args: string[]): Promise<unknown> {
  const result = await runPortalProcess(source, args)
  const parsed = parseProcessJson(result)
  if (result.timedOut || result.exitCode !== 0) {
    const adapterMessage =
      parsed && typeof parsed === "object" && typeof (parsed as { error?: unknown }).error === "string"
        ? (parsed as { error: string }).error
        : null
    throw new PortalExecutionError(
      result.timedOut
        ? `${source.displayName} timed out`
        : adapterMessage ?? `${source.displayName} exited ${result.exitCode}`,
      result.stderr || result.stdout,
      result.timedOut,
    )
  }
  return parsed
}

function requestedConstraintNames(criteria: SearchCriteria): string[] {
  return [
    ...((criteria.queryTerms.length > 0 || criteria.skills.length > 0) ? ["query"] : []),
    ...(criteria.locations.length > 0 ? ["locations"] : []),
    ...(criteria.maxExperienceYears != null ? ["maxExperience"] : []),
    ...(criteria.seniority.length > 0 ? ["seniority"] : []),
    ...(criteria.postedWithinDays != null ? ["recency"] : []),
    ...(criteria.workModes.length > 0 ? ["workMode"] : []),
    ...(criteria.employmentTypes.length > 0 ? ["employmentType"] : []),
  ]
}

export function classifyProcessFailure(
  payload: string,
  timedOut: boolean,
): PortalRunStatus {
  if (timedOut) return "unavailable"
  try {
    const code = String((JSON.parse(payload) as { code?: unknown }).code ?? "")
    if (["RESTRICTED", "HTTP_403", "CHALLENGE_PAGE", "LOGIN_REQUIRED"].includes(code)) {
      return "restricted"
    }
    if (["UNAVAILABLE", "TIMEOUT", "NETWORK_ERROR"].includes(code)) {
      return "unavailable"
    }
  } catch {
    // Unstructured process errors remain source-local failures.
  }
  return "failed"
}

export async function executeDiscovery(
  criteria: SearchCriteria,
  sources: PortalSource[],
  executor: PortalExecutor = defaultExecutor,
): Promise<PortalRunResult[]> {
  return Promise.all(
    sources.map(async (source) => {
      const startedAt = new Date().toISOString()
      try {
        const output = await executor(source, buildAdapterArgs(criteria, source))
        return normalizeAdapterSearch(
          source.sourceId,
          output,
          startedAt,
          new Date().toISOString(),
        )
      } catch (error) {
        const processError =
          error instanceof PortalExecutionError
            ? error
            : new PortalExecutionError(
                error instanceof Error ? error.message : String(error),
                "",
                false,
              )
        const status = classifyProcessFailure(processError.payload, processError.timedOut)
        const run = failedRun(
          source.sourceId,
          startedAt,
          new Date().toISOString(),
          processError.timedOut ? "TIMEOUT" : status.toUpperCase(),
          processError.message,
          status,
        )
        run.unsupportedConstraints = requestedConstraintNames(criteria)
        return run
      }
    }),
  )
}

async function enrichRun(
  result: PortalRunResult,
  criteria: SearchCriteria,
): Promise<PortalRunResult> {
  const portal = findPortal(result.sourceId)
  if (!portal || portal.detailCapability !== "automated_public" || result.listings.length === 0) {
    return result
  }
  const listings = await Promise.all(
    result.listings.map(async (listing) => {
      const processResult = await runPortalProcess(portal, [
        "detail",
        listing.sourceUrl,
        "--format",
        "json",
      ])
      if (processResult.timedOut || processResult.exitCode !== 0) {
        const status = classifyProcessFailure(
          processResult.stderr || processResult.stdout,
          processResult.timedOut,
        )
        return {
          ...listing,
          verification: {
            status:
              status === "restricted"
                ? ("restricted" as const)
                : ("failed" as const),
            detailAccessible: false,
            active: null,
            descriptionSufficient: false,
            experienceDecisionSupported: false,
            checkedAt: new Date().toISOString(),
            reasonCodes: [status.toUpperCase()],
          },
        }
      }
      const detailed = normalizeAdapterDetail(listing, parseProcessJson(processResult))
      return { ...detailed, verification: verifyListing(detailed) }
    }),
  )
  const verifiedActiveCount = listings.filter(
    (listing) => listing.verification.status === "verified_active",
  ).length
  const expiredOrRemovedCount = listings.filter((listing) =>
    ["expired", "removed"].includes(listing.verification.status),
  ).length
  const actionableCount = listings.filter(
    (listing) => evaluateFit(listing, criteria).recommendation !== null,
  ).length
  return {
    ...result,
    listings,
    verifiedActiveCount,
    expiredOrRemovedCount,
    inaccessibleLeadCount: listings.length - verifiedActiveCount,
    actionableCount,
  }
}

function initialCoverage(results: PortalRunResult[]): SourceCoverageSummary {
  const statuses: PortalRunStatus[] = [
    "searched",
    "no_matches",
    "manual_required",
    "restricted",
    "unavailable",
    "failed",
  ]
  const statusCounts = Object.fromEntries(statuses.map((status) => [status, 0])) as Record<
    PortalRunStatus,
    number
  >
  for (const result of results) statusCounts[result.status] += 1
  return {
    totalSources: results.length,
    statusCounts,
    rawDiscoveries: results.reduce((sum, result) => sum + result.rawDiscoveryCount, 0),
    verifiedActive: 0,
    duplicates: 0,
    expiredOrRemoved: 0,
    inaccessibleLeads: results.reduce(
      (sum, result) => sum + result.inaccessibleLeadCount,
      0,
    ),
    actionableMatches: 0,
    sourceRows: results.map((result) => ({
      source: result.sourceId,
      status: result.status,
      appliedConstraints: Object.keys(result.appliedConstraints),
      unsupportedConstraints: result.unsupportedConstraints,
      rawDiscoveries: result.rawDiscoveryCount,
      verifiedActive: result.verifiedActiveCount,
      duplicates: result.duplicateCount,
      expiredOrRemoved: result.expiredOrRemovedCount,
      inaccessibleLeads: result.inaccessibleLeadCount,
      actionable: result.actionableCount,
      reason: result.message,
      manualReviewUrl: result.manualReviewUrl,
    })),
  }
}

function renderDiscovery(run: PortalSearchRun, format: string): string {
  if (format === "json") return JSON.stringify(run, null, 2)
  const sourceLines = run.sourceStatuses.map((source) => {
    const counts = `raw=${source.rawDiscoveryCount} leads=${source.inaccessibleLeadCount}`
    const manual = source.manualReviewUrl ? `\n  review: ${source.manualReviewUrl}` : ""
    return `${source.sourceId.padEnd(14)} ${source.status.padEnd(15)} ${counts}${manual}`
  })
  const leadLines = run.unverifiedLeads.map(
    (lead) =>
      `${lead.title ?? "Unknown title"}\n  ${lead.employer ?? "Unknown employer"} · ${lead.location ?? "Unknown location"}\n  ${lead.canonicalUrl}`,
  )
  return [
    "Portal coverage",
    ...sourceLines,
    "",
    "Unverified discovery leads (no fit score)",
    ...(leadLines.length ? leadLines : ["None"]),
  ].join("\n")
}

export async function runSearch(options: SearchCommandOptions): Promise<number> {
  const allowed = new Set(list(options.flags.source))
  const sources = PORTAL_REGISTRY.filter(
    (source) => source.enabledByDefault && (allowed.size === 0 || allowed.has(source.sourceId)),
  )
  const startedAt = new Date().toISOString()
  const discovered = await executeDiscovery(options.criteria, sources)
  let sourceStatuses = await Promise.all(
    discovered.map((result) => enrichRun(result, options.criteria)),
  )
  const completedAt = new Date().toISOString()
  const allListings = sourceStatuses.flatMap((source) => source.listings)
  const canonicalJobs = deduplicateListings(allListings, options.criteria)
  const duplicateRefs = new Set(
    canonicalJobs
      .filter((job) => job.sourceListings.length > 1)
      .flatMap((job) =>
        job.sourceListings.map(
          (listing) =>
            `${listing.sourceId}|${listing.sourceJobId ?? ""}|${listing.canonicalUrl}`,
        ),
      ),
  )
  sourceStatuses = sourceStatuses.map((source) => ({
    ...source,
    duplicateCount: source.listings.filter((listing) =>
      duplicateRefs.has(
        `${listing.sourceId}|${listing.sourceJobId ?? ""}|${listing.canonicalUrl}`,
      ),
    ).length,
  }))
  const seenPath =
    typeof options.flags["seen-path"] === "string"
      ? resolve(REPOSITORY_ROOT, options.flags["seen-path"])
      : DEFAULT_SEEN_PATH
  const trackerPath =
    typeof options.flags["tracker-path"] === "string"
      ? resolve(REPOSITORY_ROOT, options.flags["tracker-path"])
      : DEFAULT_TRACKER_PATH
  const jobsWithHistory = attachHistory(
    canonicalJobs,
    await loadSeenJobs(seenPath),
    await loadTracker(trackerPath),
  )
  const jobs = jobsWithHistory.filter((job) => job.fitEvidence?.recommendation != null)
  const unverifiedLeads = allListings.filter(
    (listing) =>
      listing.verification.status !== "verified_active" ||
      evaluateFit(listing, options.criteria).recommendation === null,
  )
  const run: PortalSearchRun = {
    contractVersion: CONTRACT_VERSION,
    runId: `${completedAt.replace(/\D/g, "").slice(0, 14)}-${crypto.randomUUID().slice(0, 8)}`,
    startedAt,
    completedAt,
    criteria: options.criteria,
    sourceStatuses,
    jobs,
    unverifiedLeads,
    coverage: deriveCoverage(sourceStatuses),
    warnings: [
      "Discovery leads are not fit-scored until a permitted public detail is verified.",
    ],
    sessionPath: null,
  }
  if (options.flags["save-session"] === true) {
    const sessionPath = defaultSessionPath(run.runId)
    run.sessionPath = sessionPath
    await upsertSeenJobs(
      seenPath,
      seenRecordsFromJobs(jobsWithHistory, completedAt),
    )
    await saveSession(run, sessionPath)
  }
  const format =
    options.flags.format === "table" || options.flags.format === "plain"
      ? options.flags.format
      : "json"
  process.stdout.write(renderRun(run, String(format)) + "\n")
  return 0
}

export function adapterEnvelopeForTest(
  source: string,
  status: AdapterSearchEnvelope["status"],
): AdapterSearchEnvelope {
  return {
    contractVersion: "1",
    source,
    accessMode: status === "searched" ? "enabled_public" : "restricted",
    status,
    query: {},
    appliedConstraints: {},
    unsupportedConstraints: [],
    manualReviewUrl: null,
    meta: { count: 0, page: 1, total: 0 },
    results: [],
    warnings: [],
  }
}
