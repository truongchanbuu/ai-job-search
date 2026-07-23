import { readFileSync } from "node:fs"
import { resolveWorkspacePath } from "./paths.js"
import { normalizeOpportunity } from "./normalize.js"
import type { FreelanceOpportunity } from "./models.js"

export function parseUserSuppliedOpportunity(path: string): FreelanceOpportunity {
  const text = readFileSync(resolveWorkspacePath(path), "utf8")
  const firstLine = text.split(/\r?\n/).find((line) => line.trim())?.trim()
  const budget = text.match(/(?:budget|rate)[:\s]+([^\n]+)/i)?.[1]?.trim() ?? null
  const timeline = text.match(/(?:timeline|deadline|duration)[:\s]+([^\n]+)/i)?.[1]?.trim() ?? null
  return normalizeOpportunity({
    title: firstLine || "User-supplied freelance opportunity",
    clientOrSource: "User supplied",
    platform: "user_supplied",
    url: path,
    budget,
    timeline,
    description: text,
    requiresVerification: false,
    source: "user_supplied",
  }, "user_supplied")
}
