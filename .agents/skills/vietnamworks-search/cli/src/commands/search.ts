import { buildSearchEnvelope, type ManualSearchOptions } from "../helpers.js"

export async function runSearch(
  options: ManualSearchOptions & { format: "json" | "table" | "plain" },
): Promise<number> {
  const envelope = buildSearchEnvelope(options)
  process.stdout.write(
    options.format === "json"
      ? JSON.stringify(envelope, null, 2) + "\n"
      : `VietnamWorks: manual review required\n${envelope.manualReviewUrl}\n${envelope.warnings[0]}\n`,
  )
  return 0
}
