import { writeOutput } from "../render.js"
import { runFreelanceSearch, type SearchOptions } from "../search.js"
import { attachExistingStatuses } from "../records.js"
import type { OutputFormat } from "../models.js"

export async function runSearch(opts: SearchOptions & { format: OutputFormat }): Promise<number> {
  const output = runFreelanceSearch(opts)
  writeOutput({ ...output, opportunities: attachExistingStatuses(output.opportunities) }, opts.format)
  return 0
}
