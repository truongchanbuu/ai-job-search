import { expect, test } from "bun:test"
import { buildGoogleQuery, buildQueryLinks, googleUrl, isGoogleJobsUiUrl, normalizeResult } from "../src/helpers.js"

test("buildGoogleQuery combines role, location, recency, and site filters", () => {
  expect(
    buildGoogleQuery({
      query: "Backend Engineer Node.js",
      location: "Ho Chi Minh City",
      jobage: 30,
      sites: ["https://linkedin.com/jobs/search"],
    }),
  ).toBe("Backend Engineer Node.js Ho Chi Minh City (job OR jobs OR career OR careers OR hiring OR apply) posted within 30 days site:linkedin.com")
})

test("buildQueryLinks includes broad and site-specific links", () => {
  const links = buildQueryLinks({ query: "NestJS", location: "Remote Vietnam", sites: ["itviec.com"] })
  expect(links).toHaveLength(2)
  expect(links[0]).toMatchObject({
    source: "google",
    resultType: "manual_public_page_query",
    requiresVerification: true,
  })
  expect(links[0].query).not.toContain("site:")
  expect(links[1].query).toContain("site:itviec.com")
  expect(links[1].url).toStartWith("https://www.google.com/search?q=")
})

test("googleUrl encodes query", () => {
  expect(googleUrl("C# developer Ho Chi Minh")).toContain("C%23%20developer")
})

test("normalizeResult maps Google API item", () => {
  expect(
    normalizeResult(
      {
        title: "Backend Engineer",
        link: "https://example.com/jobs/1",
        snippet: "Node.js role",
        displayLink: "example.com",
        pagemap: { metatags: [{ date: "2026-07-10" }] },
      },
      0,
    ),
  ).toEqual({
    id: "1",
    source: "google",
    resultType: "public_page_discovery",
    requiresVerification: true,
    title: "Backend Engineer",
    snippet: "Node.js role",
    displayLink: "example.com",
    url: "https://example.com/jobs/1",
    date: "2026-07-10",
  })
})

test("normalizeResult skips Google Jobs UI links", () => {
  expect(isGoogleJobsUiUrl("https://www.google.com/search?q=flutter&ibp=htl;jobs")).toBe(true)
  expect(isGoogleJobsUiUrl("https://careers.google.com/jobs/results/123")).toBe(false)
  expect(
    normalizeResult(
      {
        title: "Flutter Developer",
        link: "https://www.google.com/search?q=flutter&ibp=htl%3Bjobs",
        snippet: "Google Jobs UI result",
        displayLink: "google.com",
      },
      0,
    ),
  ).toBeNull()
})
