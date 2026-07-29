import { describe, expect, test } from "bun:test"
import { join } from "node:path"
import { PortalError, parseDetailPage } from "../src/helpers.js"

const url =
  "https://www.topcv.vn/viec-lam/fresher-java-developer/1954519.html?utm_source=test"

describe("TopCV detail", () => {
  test("extracts canonical evidence, experience, and nullable fields", async () => {
    const html = await Bun.file(join(import.meta.dir, "fixtures/detail/active.html")).text()
    const detail = parseDetailPage(html, url, "2026-07-28T00:00:00Z")
    expect(detail.status).toBe("verified_active")
    expect(detail.job).toMatchObject({
      id: "1954519",
      title: "Fresher Java Developer",
      company: "Công ty Ví dụ",
      location: "Hà Nội",
      minExperienceYears: 0.5,
      maxExperienceYears: 1,
      deadline: "2026-08-15",
      workMode: null,
      salary: null,
    })
    expect(detail.job.url).not.toContain("utm_source")
    expect(detail.verification.experienceDecisionSupported).toBe(true)
  })

  test("marks explicit closed pages expired", async () => {
    const html = await Bun.file(join(import.meta.dir, "fixtures/detail/closed.html")).text()
    expect(parseDetailPage(html, url).status).toBe("expired")
  })

  test("rejects challenge pages as restricted", () => {
    expect(() => parseDetailPage("<html>Verify you are human CAPTCHA</html>", url)).toThrow(
      PortalError,
    )
  })
})
