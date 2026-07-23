import { expect, test } from "bun:test"
import { buildLead, fiverrUrl } from "../src/cli.js"

test("fiverrUrl builds a Fiverr gig search URL", () => {
  expect(fiverrUrl("Flutter app", "mobile development")).toContain("fiverr.com/search/gigs")
  expect(fiverrUrl("Flutter app", "mobile development")).toContain("Flutter%20app%20mobile%20development")
})

test("buildLead marks Fiverr output as verification-required", () => {
  expect(buildLead("Flutter app")).toMatchObject({
    source: "fiverr",
    resultType: "manual_discovery_query",
    requiresVerification: true,
  })
})
