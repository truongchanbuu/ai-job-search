import { readJson } from "../storage.js"
import { resolveWorkspacePath } from "../paths.js"
import { loadProfile } from "../profile.js"
import { draftProposal } from "../proposal.js"
import { writeOutput } from "../render.js"
import type { FreelanceOpportunity, OutputFormat, ProposalMaterial } from "../models.js"
import { validateProposalMaterial } from "../proposal-model.js"

export async function runProposal(opts: { opportunity?: string; profile?: string; materialType?: ProposalMaterial["materialType"]; format: OutputFormat }): Promise<number> {
  if (!opts.opportunity) throw new Error("--opportunity is required")
  const opportunity = readJson<FreelanceOpportunity>(resolveWorkspacePath(opts.opportunity), null as any)
  const material = draftProposal(opportunity, loadProfile(opts.profile), opts.materialType ?? "proposal")
  validateProposalMaterial(material)
  writeOutput(material, opts.format)
  return 0
}
