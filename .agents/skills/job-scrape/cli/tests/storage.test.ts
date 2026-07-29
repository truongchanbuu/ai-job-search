import { describe, expect, test } from "bun:test"
import { mkdtemp, readFile, writeFile } from "node:fs/promises"
import { join } from "node:path"
import { tmpdir } from "node:os"
import { saveSession, upsertSeenJobs } from "../src/storage.js"
import type { PortalSearchRun, SeenJobState } from "../src/contracts.js"

describe("private persistence", () => {
  test("saves a session only at the requested path", async () => {
    const root = await mkdtemp(join(tmpdir(), "job-storage-"))
    const path = join(root, "sessions", "run.json")
    const run = {
      contractVersion: "1",
      runId: "run",
      startedAt: "2026-07-28T00:00:00Z",
      completedAt: "2026-07-28T00:00:01Z",
      criteria: {
        queryTerms: [],
        skills: [],
        locations: [],
        maxExperienceYears: null,
        seniority: [],
        postedWithinDays: null,
        workModes: [],
        employmentTypes: [],
        exclusions: [],
        languageTerms: [],
        limitPerSource: 10,
      },
      sourceStatuses: [],
      jobs: [],
      unverifiedLeads: [],
      coverage: {
        totalSources: 0,
        statusCounts: {
          searched: 0,
          no_matches: 0,
          manual_required: 0,
          restricted: 0,
          unavailable: 0,
          failed: 0,
        },
        rawDiscoveries: 0,
        verifiedActive: 0,
        duplicates: 0,
        expiredOrRemoved: 0,
        inaccessibleLeads: 0,
        actionableMatches: 0,
        sourceRows: [],
      },
      warnings: [],
    } satisfies PortalSearchRun
    await saveSession(run, path)
    expect(JSON.parse(await readFile(path, "utf8")).runId).toBe("run")
  })

  test("upserts seen records while preserving unrelated records and firstSeenAt", async () => {
    const root = await mkdtemp(join(tmpdir(), "job-storage-"))
    const path = join(root, "seen.json")
    const existing: SeenJobState[] = [
      {
        jobId: "job_1",
        canonicalKey: "one",
        sourceRefs: [],
        state: "seen",
        firstSeenAt: "2026-07-01T00:00:00Z",
        lastSeenAt: "2026-07-01T00:00:00Z",
        notes: null,
      },
      {
        jobId: "job_other",
        canonicalKey: "other",
        sourceRefs: [],
        state: "dismissed",
        firstSeenAt: "2026-07-02T00:00:00Z",
        lastSeenAt: "2026-07-02T00:00:00Z",
        notes: null,
      },
    ]
    await writeFile(path, JSON.stringify({ records: existing }))
    await upsertSeenJobs(path, [
      {
        ...existing[0],
        state: "shortlisted",
        firstSeenAt: "2026-07-28T00:00:00Z",
        lastSeenAt: "2026-07-28T00:00:00Z",
      },
    ])
    const records = JSON.parse(await readFile(path, "utf8")).records as SeenJobState[]
    expect(records).toHaveLength(2)
    expect(records.find((record) => record.jobId === "job_1")?.firstSeenAt).toBe(
      "2026-07-01T00:00:00Z",
    )
    expect(records.find((record) => record.jobId === "job_other")?.state).toBe("dismissed")
  })
})
