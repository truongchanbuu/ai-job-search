import { describe, expect, test } from "bun:test"
import { buildSearchEnvelope } from "../src/helpers.js"

describe("VietnamWorks restricted search", () => {
  test("returns an official review link without inferred job IDs", () => {
    const envelope = buildSearchEnvelope({ query: "Java fresher" })
    expect(envelope.status).toBe("manual_required")
    expect(envelope.manualReviewUrl).toContain("vietnamworks.com")
    expect(envelope.results).toEqual([])
    expect(envelope.warnings.join(" ")).toContain("no private/mobile endpoint")
  })
})
