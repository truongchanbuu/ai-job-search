import { describe, expect, test } from "bun:test"
import { join } from "node:path"
import { buildSearchUrl, parseSearchPage } from "../src/helpers.js"

describe("TopCV search", () => {
  test("parses canonical IDs, URLs, Unicode, and nullable fields", async () => {
    const html = await Bun.file(join(import.meta.dir, "fixtures/search/results.html")).text()
    const results = parseSearchPage(html, "2026-07-28T00:00:00Z")
    expect(results).toHaveLength(1)
    expect(results[0]).toMatchObject({
      id: "1954519",
      title: "Fresher Java Developer",
      company: "Công ty Ví dụ",
      location: "Hà Nội",
      verificationStatus: "unverified_lead",
      requiresVerification: true,
    })
    expect(results[0].url).toBe(
      "https://www.topcv.vn/viec-lam/fresher-java-developer/1954519.html",
    )
    expect(results[0].deadline).toBeNull()
  })

  test("recognizes a valid empty page and rejects generic markup", async () => {
    const empty = await Bun.file(join(import.meta.dir, "fixtures/search/empty.html")).text()
    expect(parseSearchPage(empty)).toEqual([])
    expect(() => parseSearchPage("<html><body>generic</body></html>")).toThrow()
  })

  test("maps mixed-language keywords to the official SEO search URL", () => {
    expect(buildSearchUrl({ query: "Java lập trình viên ReactJS" })).toBe(
      "https://www.topcv.vn/tim-viec-lam-java-lap-trinh-vien-reactjs",
    )
  })
})
