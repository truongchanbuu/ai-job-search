import {
  PortalError,
  buildSearchUrl,
  parseSearchPage,
  publicFetch,
  writeError,
  type SearchOptions,
} from "../helpers.js"

export interface CommandOptions extends SearchOptions {
  format: "json" | "table" | "plain"
}

export async function runSearch(options: CommandOptions): Promise<number> {
  try {
    let results = parseSearchPage(await publicFetch(buildSearchUrl(options)))
    if (options.limit != null) results = results.slice(0, Math.max(0, options.limit))
    const envelope = {
      contractVersion: "1",
      source: "topcv",
      accessMode: "enabled_public",
      status: results.length ? "searched" : "no_matches",
      query: options,
      appliedConstraints: {
        query: options.query ?? null,
        location: [],
        experience: null,
        seniority: [],
        workMode: [],
      },
      unsupportedConstraints: [
        ...((options.locations?.length ?? 0) > 0 ? ["location"] : []),
        ...(options.maxExperience != null ? ["experience"] : []),
        ...((options.seniority?.length ?? 0) > 0 ? ["seniority"] : []),
        ...((options.workModes?.length ?? 0) > 0 ? ["workMode"] : []),
        ...(options.jobage != null ? ["recency"] : []),
      ],
      manualReviewUrl: null,
      meta: { count: results.length, page: 1, total: results.length },
      results,
      warnings: [],
    }
    if (options.format === "json") {
      process.stdout.write(JSON.stringify(envelope, null, 2) + "\n")
    } else {
      process.stdout.write(
        results.length
          ? results.map((result) => `${result.title ?? "Unknown"}\n  ${result.url}`).join("\n\n") + "\n"
          : "No results.\n",
      )
    }
    return 0
  } catch (error) {
    const code = error instanceof PortalError ? error.code : "SEARCH_FAILED"
    writeError(error instanceof Error ? error.message : String(error), code)
    return 1
  }
}
