import { describe, expect, test } from "bun:test"
import { mkdtemp, writeFile } from "node:fs/promises"
import { join } from "node:path"
import { tmpdir } from "node:os"
import {
  attachHistory,
  loadSeenJobs,
  loadTracker,
} from "../src/history.js"
import type { CanonicalJobListing, SeenJobState } from "../src/contracts.js"

const job: CanonicalJobListing = {
  jobId: "job_1",
  canonicalKey: "example|java|ha-noi",
  sourceListings: [],
  title: "Java Developer",
  employer: "Example",
  location: "Ha Noi",
  skills: ["Java"],
  experience: null,
  seniority: null,
  employmentType: null,
  workMode: null,
  postedAt: null,
  deadline: null,
  salary: null,
  conflicts: [],
  verificationStatus: "verified_active",
  existingSeenState: null,
  existingApplicationStatus: null,
  fitEvidence: null,
}

describe("history reconciliation", () => {
  test("treats missing files as empty", async () => {
    const root = await mkdtemp(join(tmpdir(), "job-history-"))
    expect(await loadSeenJobs(join(root, "missing.json"))).toEqual([])
    expect(await loadTracker(join(root, "missing.csv"))).toEqual([])
  })

  test("accepts raw arrays and envelopes", async () => {
    const root = await mkdtemp(join(tmpdir(), "job-history-"))
    const record: SeenJobState = {
      jobId: "job_1",
      canonicalKey: job.canonicalKey,
      sourceRefs: [],
      state: "shortlisted",
      firstSeenAt: "2026-07-20T00:00:00Z",
      lastSeenAt: "2026-07-28T00:00:00Z",
      notes: null,
    }
    const rawPath = join(root, "raw.json")
    const envelopePath = join(root, "envelope.json")
    await writeFile(rawPath, JSON.stringify([record]))
    await writeFile(envelopePath, JSON.stringify({ records: [record] }))
    expect(await loadSeenJobs(rawPath)).toHaveLength(1)
    expect(await loadSeenJobs(envelopePath)).toHaveLength(1)
  })

  test("attaches seen and tracker status without writing the tracker", async () => {
    const root = await mkdtemp(join(tmpdir(), "job-history-"))
    const trackerPath = join(root, "tracker.csv")
    await writeFile(
      trackerPath,
      "application_id,status,employer,title,location,source_url\napp_1,applied,Example,Java Developer,Ha Noi,https://topcv.example/1\n",
    )
    const tracker = await loadTracker(trackerPath)
    const seen: SeenJobState[] = [
      {
        jobId: job.jobId,
        canonicalKey: job.canonicalKey,
        sourceRefs: [],
        state: "shortlisted",
        firstSeenAt: "2026-07-20T00:00:00Z",
        lastSeenAt: "2026-07-28T00:00:00Z",
        notes: null,
      },
    ]
    const [attached] = attachHistory([job], seen, tracker)
    expect(attached.existingSeenState).toBe("shortlisted")
    expect(attached.existingApplicationStatus).toBe("applied")
  })
})
