import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs"
import { dirname } from "node:path"
import { assertNoSecrets } from "./safety.js"

export function readJson<T>(path: string, fallback: T): T {
  if (!existsSync(path)) return fallback
  const raw = readFileSync(path, "utf8").trim()
  if (!raw) return fallback
  return JSON.parse(raw) as T
}

export function writeJson<T>(path: string, value: T): void {
  assertNoSecrets(value)
  mkdirSync(dirname(path), { recursive: true })
  writeFileSync(path, JSON.stringify(value, null, 2) + "\n", "utf8")
}

export function readJsonArray<T>(path: string): T[] {
  const value = readJson<T[] | { records?: T[]; items?: T[] }>(path, [])
  if (Array.isArray(value)) return value
  return value.records ?? value.items ?? []
}

export function upsertBy<T>(items: T[], item: T, predicate: (existing: T) => boolean): T[] {
  const index = items.findIndex(predicate)
  if (index === -1) return [...items, item]
  const next = [...items]
  next[index] = item
  return next
}
