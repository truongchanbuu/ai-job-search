import { buildDetailEnvelope, writeError } from "../helpers.js"

export async function runDetail(options: {
  input: string
  format: "json" | "plain"
}): Promise<number> {
  try {
    const detail = buildDetailEnvelope(options.input)
    process.stdout.write(
      options.format === "json"
        ? JSON.stringify(detail, null, 2) + "\n"
        : `CareerViet reference\n${detail.job.url}\nStatus: ${detail.status}\n`,
    )
    return 0
  } catch (error) {
    writeError(error instanceof Error ? error.message : String(error), "BAD_ID")
    return 1
  }
}
