import { expect, test } from "bun:test"
import { dedupeOpportunities } from "../src/dedupe.js"
import { normalizeOpportunity } from "../src/normalize.js"
import { detectRiskFlags } from "../src/risk.js"
import { createFitMatch } from "../src/rank.js"

test("normalizeOpportunity marks discovery leads and extracts skills", () => {
  const opportunity = normalizeOpportunity({
    title: "Flutter Firebase bug fix",
    platform: "upwork",
    description: "Need Flutter and Firebase support",
  }, "upwork")

  expect(opportunity.requiresVerification).toBe(true)
  expect(opportunity.skills).toContain("Flutter")
  expect(opportunity.sourceRefs[0].status).toBe("discovery_lead")
})

test("dedupeOpportunities merges source references", () => {
  const first = normalizeOpportunity({ title: "Flutter fix", platform: "upwork", url: "https://example.com/1" }, "upwork")
  const second = normalizeOpportunity({ title: "Flutter fix", platform: "upwork", url: "https://example.com/1" }, "google")
  const deduped = dedupeOpportunities([first, second])

  expect(deduped).toHaveLength(1)
  expect(deduped[0].sourceRefs).toHaveLength(2)
})

test("risk and fit avoid proposal-worthy recommendation for unverified risky leads", () => {
  const opportunity = normalizeOpportunity({
    title: "Urgent unpaid Flutter job",
    platform: "social",
    budget: "$20",
    description: "Contact Telegram today",
  }, "social")

  expect(detectRiskFlags(opportunity)).toContain("scam_risk")
  const fit = createFitMatch(opportunity, { profile_id: "default", target_services: ["Flutter"], main_skills: ["Flutter"] })
  expect(fit.recommendation).toBe("skip")
})
