import { expect, test } from "bun:test"
import { buildLead, upworkUrl } from "../src/cli.js"

test("upworkUrl builds an Upwork job search URL", () => {
  expect(upworkUrl("Flutter Firebase", "Remote")).toContain("upwork.com/nx/search/jobs/")
  expect(upworkUrl("Flutter Firebase", "Remote")).toContain("Flutter%20Firebase%20Remote")
})

test("buildLead marks Upwork output as verification-required", () => {
  expect(buildLead("Flutter")).toMatchObject({
    source: "upwork",
    resultType: "manual_discovery_query",
    requiresVerification: true,
  })
})
