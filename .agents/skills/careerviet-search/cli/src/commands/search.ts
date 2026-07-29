import { buildSearchEnvelope, type ManualSearchOptions } from "../helpers.js"

export interface SearchOptions extends ManualSearchOptions {
  format: "json" | "table" | "plain"
}

export async function runSearch(options: SearchOptions): Promise<number> {
  const envelope = buildSearchEnvelope(options)
  if (options.format === "json") {
    process.stdout.write(JSON.stringify(envelope, null, 2) + "\n")
  } else {
    process.stdout.write(
      `CareerViet: manual review required\n${envelope.manualReviewUrl}\n${envelope.warnings[0]}\n`,
    )
  }
  return 0
}
