import { defaultProfilePath, resolveWorkspacePath } from "./paths.js"
import { readJson } from "./storage.js"
import type { FreelanceProfileExtension } from "./models.js"

export function defaultProfile(): FreelanceProfileExtension {
  return {
    profile_id: "default",
    target_services: [],
    portfolio_links: [],
    work_type_preferences: [],
    languages: ["en", "vi"],
    platform_profiles: {},
    exclusions: [],
    main_skills: [],
  }
}

export function loadProfile(path?: string): FreelanceProfileExtension {
  const profile = readJson<FreelanceProfileExtension>(path ? resolveWorkspacePath(path) : defaultProfilePath(), defaultProfile())
  validateProfile(profile)
  return profile
}

export function validateProfile(profile: FreelanceProfileExtension): void {
  if (!profile.profile_id) throw new Error("Freelance profile requires profile_id")
  if (!Array.isArray(profile.target_services)) throw new Error("Freelance profile target_services must be a list")
}

export function profileKeywords(profile: FreelanceProfileExtension): string[] {
  return Array.from(new Set([...(profile.target_services ?? []), ...(profile.main_skills ?? [])]
    .map((item) => item.toLowerCase().trim())
    .filter(Boolean)))
}
