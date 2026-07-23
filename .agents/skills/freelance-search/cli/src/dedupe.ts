import type { FreelanceOpportunity } from "./models.js"

export function dedupeOpportunities(items: FreelanceOpportunity[]): FreelanceOpportunity[] {
  const byKey = new Map<string, FreelanceOpportunity>()
  for (const item of items) {
    const key = item.canonicalKey || item.url || item.opportunityId
    const existing = byKey.get(key)
    if (!existing) {
      byKey.set(key, item)
      continue
    }
    byKey.set(key, {
      ...existing,
      sourceRefs: [...existing.sourceRefs, ...item.sourceRefs],
      riskFlags: Array.from(new Set([...existing.riskFlags, ...item.riskFlags])),
      skills: Array.from(new Set([...existing.skills, ...item.skills])),
      requiresVerification: existing.requiresVerification && item.requiresVerification,
    })
  }
  return [...byKey.values()]
}
