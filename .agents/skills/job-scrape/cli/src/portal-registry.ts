import { resolve } from "node:path"
import type { PortalSource } from "./contracts.js"

const REPOSITORY_ROOT = resolve(import.meta.dir, "../../../../..")

function cli(source: string): string {
  return resolve(REPOSITORY_ROOT, `.agents/skills/${source}-search/cli/src/cli.ts`)
}

export const PORTAL_REGISTRY: PortalSource[] = [
  {
    sourceId: "careerviet",
    displayName: "CareerViet",
    baseUrl: "https://careerviet.vn/",
    cliPath: cli("careerviet"),
    accessMode: "manual_only",
    enabledByDefault: true,
    searchCapability: "official_link",
    detailCapability: "user_supplied_public",
    supportedConstraints: ["query", "location", "experience", "seniority", "recency", "employmentType", "salary"],
    policyNotes: "Generate official browser links only; do not perform unattended crawler/robot search.",
    accessReviewedAt: "2026-07-28",
    requestTimeoutMs: 5_000,
  },
  {
    sourceId: "vieclam24h",
    displayName: "Vieclam24h",
    baseUrl: "https://vieclam24h.vn/",
    cliPath: cli("vieclam24h"),
    accessMode: "restricted",
    enabledByDefault: true,
    searchCapability: "manual",
    detailCapability: "pasted_text",
    supportedConstraints: ["query", "location", "experience", "seniority", "salary"],
    policyNotes: "Current ordinary requests return 403; do not reuse tokens, private APIs, or bypass challenges.",
    accessReviewedAt: "2026-07-28",
    requestTimeoutMs: 5_000,
  },
  {
    sourceId: "topcv",
    displayName: "TopCV",
    baseUrl: "https://www.topcv.vn/",
    cliPath: cli("topcv"),
    accessMode: "enabled_public",
    enabledByDefault: true,
    searchCapability: "automated",
    detailCapability: "automated_public",
    supportedConstraints: ["query", "location", "experience", "seniority", "workMode", "salary"],
    policyNotes: "Low-volume, link-first public access; retain normalized facts rather than mirrored content.",
    accessReviewedAt: "2026-07-28",
    requestTimeoutMs: 15_000,
  },
  {
    sourceId: "itviec",
    displayName: "ITviec",
    baseUrl: "https://itviec.com/",
    cliPath: cli("itviec"),
    accessMode: "enabled_public",
    enabledByDefault: true,
    searchCapability: "automated",
    detailCapability: "automated_public",
    supportedConstraints: ["query", "location", "seniority", "workMode", "salary"],
    policyNotes: "Reasonable low-volume public access only; no login, bulk archive, or authenticated actions.",
    accessReviewedAt: "2026-07-28",
    requestTimeoutMs: 15_000,
  },
  {
    sourceId: "vietnamworks",
    displayName: "VietnamWorks",
    baseUrl: "https://www.vietnamworks.com/",
    cliPath: cli("vietnamworks"),
    accessMode: "restricted",
    enabledByDefault: true,
    searchCapability: "manual",
    detailCapability: "pasted_text",
    supportedConstraints: ["query", "location", "seniority", "salary"],
    policyNotes: "Current ordinary requests return 403; do not infer IDs or call private/mobile interfaces.",
    accessReviewedAt: "2026-07-28",
    requestTimeoutMs: 5_000,
  },
]

export function validatePortalRegistry(registry: PortalSource[]): string[] {
  const errors: string[] = []
  const ids = new Set<string>()
  for (const source of registry) {
    if (ids.has(source.sourceId)) errors.push(`duplicate source: ${source.sourceId}`)
    ids.add(source.sourceId)
    if (
      source.accessMode !== "enabled_public" &&
      source.searchCapability === "automated"
    ) {
      errors.push(`${source.sourceId}: protected access cannot be automated`)
    }
    if (source.requestTimeoutMs < 1_000 || source.requestTimeoutMs > 30_000) {
      errors.push(`${source.sourceId}: timeout must be between 1000 and 30000 ms`)
    }
    if (!source.accessReviewedAt || !source.policyNotes) {
      errors.push(`${source.sourceId}: missing access policy review`)
    }
  }
  return errors
}

const registryErrors = validatePortalRegistry(PORTAL_REGISTRY)
if (registryErrors.length) {
  throw new Error(`Invalid portal registry: ${registryErrors.join("; ")}`)
}

export function findPortal(sourceId: string): PortalSource | undefined {
  return PORTAL_REGISTRY.find((source) => source.sourceId === sourceId)
}
