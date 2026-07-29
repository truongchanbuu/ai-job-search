import { PortalError, parseDetailPage, publicFetch, writeError } from "../helpers.js"

export interface DetailOptions {
  input: string
  format: "json" | "plain"
}

export async function runDetail(options: DetailOptions): Promise<number> {
  if (!/^https:\/\/(www\.)?itviec\.com\/it-jobs\//i.test(options.input)) {
    writeError("detail requires a public ITviec job URL", "BAD_ID")
    return 1
  }
  try {
    const detail = parseDetailPage(await publicFetch(options.input), options.input)
    process.stdout.write(
      options.format === "json"
        ? JSON.stringify(detail, null, 2) + "\n"
        : `${String(detail.job.title ?? "Unknown")}\n${options.input}\nStatus: ${detail.status}\n`,
    )
    return 0
  } catch (error) {
    const code = error instanceof PortalError ? error.code : "DETAIL_FAILED"
    writeError(error instanceof Error ? error.message : String(error), code)
    return 1
  }
}
