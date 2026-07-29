import { describe, expect, test } from "bun:test"
import { buildDetailEnvelope } from "../src/helpers.js"

describe("Vieclam24h manual detail", () => {
  test("preserves a supplied reference with restricted verification", () => {
    const detail = buildDetailEnvelope("https://vieclam24h.vn/viec-lam/example")
    expect(detail.status).toBe("restricted")
    expect(detail.job.url).toContain("vieclam24h.vn")
    expect(detail.verification.descriptionSufficient).toBe(false)
  })
})
