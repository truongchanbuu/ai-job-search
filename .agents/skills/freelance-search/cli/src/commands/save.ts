import { readJson } from "../storage.js"
import { resolveWorkspacePath } from "../paths.js"
import { saveRecord } from "../records.js"
import { writeOutput } from "../render.js"
import type { FreelanceOpportunity, FreelanceOpportunityRecord, OutputFormat } from "../models.js"

export async function runSave(opts: { opportunity?: string; status?: FreelanceOpportunityRecord["status"]; materialRefs?: string[]; notes?: string; format: OutputFormat }): Promise<number> {
  if (!opts.opportunity) throw new Error("--opportunity is required")
  if (!opts.status) throw new Error("--status is required")
  const opportunity = readJson<FreelanceOpportunity>(resolveWorkspacePath(opts.opportunity), null as any)
  writeOutput(saveRecord(opportunity, opts.status, opts.materialRefs ?? [], opts.notes), opts.format)
  return 0
}
