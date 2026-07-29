import { describe, expect, test } from "bun:test"
import { join } from "node:path"
import { buildSearchUrl, parseSearchPage } from "../src/helpers.js"

describe("ITviec search", () => {
  test("parses canonical ID, Unicode facts, and unverified status", async () => {
    const html = await Bun.file(join(import.meta.dir, "fixtures/search/results.html")).text()
    const results = parseSearchPage(html, "2026-07-28T00:00:00Z")
    expect(results).toHaveLength(1)
    expect(results[0]).toMatchObject({
      id: "3413",
      company: "Công ty Ví dụ",
      location: "Ho Chi Minh City",
      verificationStatus: "unverified_lead",
    })
    expect(results[0].url).toBe(
      "https://itviec.com/it-jobs/software-developer-fresher-junior-example-3413",
    )
  })

  test("distinguishes valid empty results from malformed pages", async () => {
    const empty = await Bun.file(join(import.meta.dir, "fixtures/search/empty.html")).text()
    expect(parseSearchPage(empty)).toEqual([])
    expect(() => parseSearchPage("<html>generic</html>")).toThrow()
  })

  test("maps skill, fresher, city, and work-mode hints", () => {
    const url = buildSearchUrl({
      query: "Java",
      locations: ["Ho Chi Minh"],
      seniority: ["fresher"],
      workModes: ["hybrid"],
    })
    expect(url).toContain("/it-jobs/java/ho-chi-minh-hcm")
  })
})
