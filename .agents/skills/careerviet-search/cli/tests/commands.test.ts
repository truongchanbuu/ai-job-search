import { describe, expect, test } from "bun:test"
import { buildSearchEnvelope } from "../src/helpers.js"

describe("CareerViet manual search", () => {
  test("returns an official criteria-preserving URL without fetching", () => {
    const envelope = buildSearchEnvelope({
      query: "Java fresher",
      locations: ["Ho Chi Minh"],
      maxExperience: 1,
    })
    expect(envelope.status).toBe("manual_required")
    expect(envelope.results).toEqual([])
    expect(envelope.manualReviewUrl).toContain("careerviet.vn/viec-lam/")
    expect(decodeURIComponent(envelope.manualReviewUrl ?? "")).toContain("java-fresher")
  })
})
