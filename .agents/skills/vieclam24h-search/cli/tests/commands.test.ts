import { describe, expect, test } from "bun:test"
import { buildSearchEnvelope } from "../src/helpers.js"

describe("Vieclam24h restricted search", () => {
  test("returns manual_required and never claims zero matches", () => {
    const envelope = buildSearchEnvelope({ query: "Java fresher" })
    expect(envelope.status).toBe("manual_required")
    expect(envelope.manualReviewUrl).toContain("vieclam24h.vn")
    expect(envelope.warnings.join(" ")).toContain("403")
    expect(envelope.results).toEqual([])
  })
})
