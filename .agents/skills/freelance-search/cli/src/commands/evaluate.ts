import { readJson } from "../storage.js"
import { resolveWorkspacePath } from "../paths.js"
import { loadProfile } from "../profile.js"
import { loadReadiness } from "../readiness-store.js"
import { createFitMatch } from "../fit.js"
import { reliabilityForOpportunity } from "../reliability.js"
import { writeOutput } from "../render.js"
import type { FreelanceOpportunity, OutputFormat } from "../models.js"

export async function runEvaluate(opts: { opportunity?: string; profile?: string; format: OutputFormat }): Promise<number> {
  if (!opts.opportunity) throw new Error("--opportunity is required")
  const opportunity = readJson<FreelanceOpportunity>(resolveWorkspacePath(opts.opportunity), null as any)
  const profile = loadProfile(opts.profile)
  const fit = createFitMatch(opportunity, profile, loadReadiness())
  writeOutput({ opportunityId: opportunity.opportunityId, fit, reliability: reliabilityForOpportunity(opportunity) }, opts.format)
  return 0
}
