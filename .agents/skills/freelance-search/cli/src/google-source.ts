import { normalizeOpportunity } from "./normalize.js"
import type { FreelanceOpportunity, FreelanceProfileExtension } from "./models.js"

export function googleDiscovery(profile: FreelanceProfileExtension, query?: string): FreelanceOpportunity[] {
  const term = query || profile.target_services[0] || profile.main_skills?.[0] || "freelance developer"
  const url = `https://www.google.com/search?q=${encodeURIComponent(`${term} freelance project client request`)}`
  return [
    normalizeOpportunity({
      title: `Google discovery: ${term}`,
      clientOrSource: "Google public-page discovery",
      platform: "google",
      url,
      description: `Public-page discovery lead for ${term}. Verify original posting before action.`,
      requiresVerification: true,
      source: "google",
    }, "google"),
  ]
}
