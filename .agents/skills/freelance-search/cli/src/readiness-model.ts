import type { AccountStatus, PlatformAccountReadiness } from "./models.js"
import { nowIso } from "./models.js"

export function createReadiness(input: Partial<PlatformAccountReadiness> & { platform: string }): PlatformAccountReadiness {
  return {
    platform: input.platform,
    accountStatus: input.accountStatus ?? "unknown",
    profileCompleteness: input.profileCompleteness ?? [],
    serviceCategories: input.serviceCategories ?? [],
    portfolioRefs: input.portfolioRefs ?? [],
    verificationStatus: input.verificationStatus ?? "unknown",
    actionCurrency: input.actionCurrency,
    knownBlockers: input.knownBlockers ?? [],
    lastUpdatedAt: input.lastUpdatedAt ?? nowIso(),
  }
}

export function readinessStatus(readiness: PlatformAccountReadiness | undefined): "ready" | "setup_required" | "limited" | "blocked" | "unknown" {
  if (!readiness) return "unknown"
  if (readiness.accountStatus === "ready" && readiness.knownBlockers.length === 0) return "ready"
  if (readiness.accountStatus === "blocked") return "blocked"
  if (readiness.accountStatus === "limited" || readiness.knownBlockers.length > 0) return "limited"
  if (readiness.accountStatus === "not_configured" || readiness.accountStatus === "draft") return "setup_required"
  return "unknown"
}

export function recommendedReadinessAction(readiness: PlatformAccountReadiness): string {
  const status = readinessStatus(readiness)
  if (status === "ready") return "Platform profile appears ready for opportunity action."
  if (status === "blocked") return "Resolve platform blocker before acting."
  if (status === "limited") return readiness.knownBlockers[0] ?? "Review platform limitations before acting."
  if (status === "setup_required") return "Complete account and service setup before acting."
  return "Record account readiness before platform-specific recommendations."
}

export function parseAccountStatus(value: string | undefined): AccountStatus {
  const allowed: AccountStatus[] = ["not_configured", "draft", "ready", "limited", "blocked", "unknown"]
  return allowed.includes(value as AccountStatus) ? value as AccountStatus : "unknown"
}
