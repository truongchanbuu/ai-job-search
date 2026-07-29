import { describe, expect, test } from "bun:test"
import { join } from "node:path"
import { PortalError, parseDetailPage } from "../src/helpers.js"

const url =
  "https://itviec.com/it-jobs/software-developer-fresher-junior-example-3413"

describe("ITviec detail", () => {
  test("extracts public evidence while leaving login-only salary and deadline null", async () => {
    const html = await Bun.file(join(import.meta.dir, "fixtures/detail/active.html")).text()
    const detail = parseDetailPage(html, url, "2026-07-28T00:00:00Z")
    expect(detail.status).toBe("verified_active")
    expect(detail.job).toMatchObject({
      id: "3413",
      company: "Example Technology",
      location: "Ho Chi Minh City",
      maxExperienceYears: 1,
      workMode: "hybrid",
      salary: null,
      deadline: null,
    })
    expect(detail.job.skills).toEqual(expect.arrayContaining(["Java", "VueJS"]))
  })

  test("rejects challenge and 403-style pages", () => {
    expect(() => parseDetailPage("<html>Access denied CAPTCHA</html>", url)).toThrow(
      PortalError,
    )
  })
})
