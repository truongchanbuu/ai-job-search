import { createReadiness, parseAccountStatus, recommendedReadinessAction } from "../readiness-model.js"
import { saveReadiness } from "../readiness-store.js"
import { writeOutput } from "../render.js"
import type { OutputFormat } from "../models.js"

export async function runReadiness(opts: {
  platform?: string
  accountStatus?: string
  serviceCategories?: string[]
  portfolioRefs?: string[]
  knownBlockers?: string[]
  verificationStatus?: string
  format: OutputFormat
}): Promise<number> {
  if (!opts.platform) throw new Error("--platform is required")
  const readiness = createReadiness({
    platform: opts.platform,
    accountStatus: parseAccountStatus(opts.accountStatus),
    serviceCategories: opts.serviceCategories ?? [],
    portfolioRefs: opts.portfolioRefs ?? [],
    knownBlockers: opts.knownBlockers ?? [],
    verificationStatus: (opts.verificationStatus as any) ?? "unknown",
  })
  saveReadiness(readiness)
  writeOutput({ ...readiness, recommendedAction: recommendedReadinessAction(readiness) }, opts.format)
  return 0
}
