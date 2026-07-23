import { dirname, join, normalize } from "node:path"
import { existsSync, mkdirSync } from "node:fs"

const ROOT_MARKER = ".specify"

export function repoRoot(): string {
  let cwd = process.cwd()
  while (cwd.length > 3) {
    if (existsSync(join(cwd, ROOT_MARKER))) return cwd
    const parent = dirname(cwd)
    if (parent === cwd) break
    cwd = parent
  }
  return process.cwd()
}

export function resolveWorkspacePath(path: string): string {
  const normalized = normalize(path)
  if (/^[a-zA-Z]:\\/.test(normalized) || normalized.startsWith("/")) return normalized
  return join(repoRoot(), normalized)
}

export function ensureDir(path: string): void {
  mkdirSync(path, { recursive: true })
}

export function defaultFreelanceDir(): string {
  return resolveWorkspacePath("documents/applications/freelance")
}

export function defaultStateDir(): string {
  return resolveWorkspacePath("job_scraper/freelance")
}

export function defaultProfilePath(): string {
  return resolveWorkspacePath("documents/freelance-profile.json")
}

export function defaultReadinessPath(): string {
  return resolveWorkspacePath("documents/applications/freelance/account-readiness.json")
}

export function defaultRecordsPath(): string {
  return resolveWorkspacePath("documents/applications/freelance/opportunity-records.json")
}
