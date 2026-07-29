import { describe, expect, test } from "bun:test"
import { buildDetailEnvelope } from "../src/helpers.js"

describe("VietnamWorks manual detail", () => {
  test("preserves a supplied reference without private retrieval", () => {
    const detail = buildDetailEnvelope("https://www.vietnamworks.com/viec-lam/example")
    expect(detail.status).toBe("restricted")
    expect(detail.job.url).toContain("vietnamworks.com")
    expect(detail.verification.descriptionSufficient).toBe(false)
  })
})
