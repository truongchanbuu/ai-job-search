import { buildGoogleQuery, normalizeResult, renderTable, writeError, type GoogleResult } from "../helpers.js"

export interface SearchOpts {
  query?: string
  location?: string
  jobage?: number
  sites?: string[]
  limit?: number
  format: "json" | "table" | "plain"
}

interface GoogleApiResponse {
  items?: Array<{
    title?: string
    link?: string
    snippet?: string
    displayLink?: string
    pagemap?: { metatags?: Array<Record<string, string>> }
  }>
  error?: { message?: string }
}

function apiUrl(opts: SearchOpts): string {
  const key = process.env.GOOGLE_API_KEY
  const cx = process.env.GOOGLE_CSE_ID
  if (!key || !cx) {
    throw new Error("GOOGLE_API_KEY and GOOGLE_CSE_ID are required for live Google API search. Use the queries command without credentials.")
  }

  const params = new URLSearchParams()
  params.set("key", key)
  params.set("cx", cx)
  params.set("q", buildGoogleQuery(opts))
  params.set("num", String(Math.min(Math.max(opts.limit ?? 10, 1), 10)))
  if (opts.jobage && opts.jobage > 0 && opts.jobage < 9999) {
    params.set("dateRestrict", `d${opts.jobage}`)
  }
  return `https://www.googleapis.com/customsearch/v1?${params.toString()}`
}

export async function runSearch(opts: SearchOpts): Promise<number> {
  try {
    const response = await fetch(apiUrl(opts), {
      headers: {
        Accept: "application/json",
        "User-Agent": "ai-job-search-google-skill/1.0",
      },
    })
    const body = (await response.json()) as GoogleApiResponse
    if (!response.ok) {
      throw new Error(body.error?.message ?? `Request failed: ${response.status} ${response.statusText}`)
    }

    let results = (body.items ?? [])
      .map((item, index) => normalizeResult(item, index))
      .filter((item): item is GoogleResult => item !== null)
    if (opts.limit !== undefined && opts.limit >= 0) results = results.slice(0, opts.limit)

    if (opts.format === "table") {
      process.stdout.write(renderTable(results) + "\n")
    } else if (opts.format === "plain") {
      process.stdout.write(
        results.map((r) => `${r.title}\n  ${r.displayLink ?? "-"} · ${r.date ?? "-"}\n  ${r.url}\n  ${r.snippet ?? ""}`).join("\n\n") + "\n",
      )
    } else {
      process.stdout.write(JSON.stringify({
        meta: {
          count: results.length,
          source: "google",
          scope: "public_page_discovery",
          requiresVerification: true,
          googleJobsUi: false,
        },
        results,
      }, null, 2) + "\n")
    }
    return 0
  } catch (e) {
    writeError(e instanceof Error ? e.message : String(e), "SEARCH_FAILED")
    return 1
  }
}
