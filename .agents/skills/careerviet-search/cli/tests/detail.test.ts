import { describe, expect, test } from "bun:test"
import { buildDetailEnvelope } from "../src/helpers.js"

describe("CareerViet manual detail", () => {
  test("normalizes a supplied reference without claiming retrieval", () => {
    const detail = buildDetailEnvelope(
      "https://careerviet.vn/vi/tim-viec-lam/java-developer.35C7D09E.html?utm_source=x",
    )
    expect(detail.status).toBe("insufficient_detail")
    expect(detail.job.url).not.toContain("utm_source")
    expect(detail.verification.detailAccessible).toBe(false)
  })

  test("rejects references outside CareerViet", () => {
    expect(() => buildDetailEnvelope("https://example.com/job/1")).toThrow()
  })
})
