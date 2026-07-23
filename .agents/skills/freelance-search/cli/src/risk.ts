import type { FreelanceOpportunity } from "./models.js"

export function detectRiskFlags(opportunity: Pick<FreelanceOpportunity, "title" | "description" | "budget" | "requiresVerification">): string[] {
  const text = `${opportunity.title} ${opportunity.description ?? ""} ${opportunity.budget ?? ""}`.toLowerCase()
  const flags = new Set<string>()
  if (opportunity.requiresVerification) flags.add("verification_required")
  if (/\bfree\b|unpaid|exposure/.test(text)) flags.add("unpaid_or_speculative")
  if (/\btelegram\b|\bwhatsapp\b|crypto|wire transfer|deposit/.test(text)) flags.add("scam_risk")
  if (/\burgent\b|asap|today/.test(text)) flags.add("urgent")
  if (/\bfull[- ]?time\b|internship|employee/.test(text)) flags.add("not_freelance")
  if (/\$?\s?([1-9]|[1-4][0-9])\b/.test(opportunity.budget ?? "")) flags.add("low_budget")
  return [...flags]
}
