import { buildQueryLinks, renderTable } from "../helpers.js"

export interface QueryOpts {
  query?: string
  location?: string
  jobage?: number
  sites?: string[]
  format: "json" | "table" | "plain"
}

export async function runQueries(opts: QueryOpts): Promise<number> {
  const links = buildQueryLinks(opts)

  if (opts.format === "table") {
    process.stdout.write(renderTable(links) + "\n")
  } else if (opts.format === "plain") {
    process.stdout.write(links.map((l) => `${l.query}\n  ${l.url}`).join("\n\n") + "\n")
  } else {
    process.stdout.write(JSON.stringify({
      meta: {
        count: links.length,
        source: "google",
        scope: "public_page_discovery",
        requiresVerification: true,
        googleJobsUi: false,
      },
      results: links,
    }, null, 2) + "\n")
  }
  return 0
}
