import { defaultReadinessPath, resolveWorkspacePath } from "./paths.js"
import { readJsonArray, upsertBy, writeJson } from "./storage.js"
import type { PlatformAccountReadiness } from "./models.js"

export function loadReadiness(path?: string): PlatformAccountReadiness[] {
  return readJsonArray<PlatformAccountReadiness>(path ? resolveWorkspacePath(path) : defaultReadinessPath())
}

export function saveReadiness(readiness: PlatformAccountReadiness, path?: string): PlatformAccountReadiness[] {
  const target = path ? resolveWorkspacePath(path) : defaultReadinessPath()
  const all = upsertBy(loadReadiness(target), readiness, (item) => item.platform.toLowerCase() === readiness.platform.toLowerCase())
  writeJson(target, all)
  return all
}

export function readinessFor(platform: string, all: PlatformAccountReadiness[]): PlatformAccountReadiness | undefined {
  return all.find((item) => item.platform.toLowerCase() === platform.toLowerCase())
}
