import { describe, expect, test } from "bun:test"
import { PORTAL_REGISTRY, validatePortalRegistry } from "../src/portal-registry.js"

describe("portal registry", () => {
  test("contains each named source exactly once", () => {
    expect(PORTAL_REGISTRY.map((source) => source.sourceId)).toEqual([
      "careerviet",
      "vieclam24h",
      "topcv",
      "itviec",
      "vietnamworks",
    ])
    expect(validatePortalRegistry(PORTAL_REGISTRY)).toEqual([])
  })

  test("prevents unattended search for manual and restricted portals", () => {
    for (const source of PORTAL_REGISTRY.filter(
      (item) => item.accessMode !== "enabled_public",
    )) {
      expect(source.searchCapability).not.toBe("automated")
      expect(source.policyNotes.length).toBeGreaterThan(10)
      expect(source.accessReviewedAt).toBe("2026-07-28")
    }
  })

  test("uses bounded source timeouts", () => {
    expect(
      PORTAL_REGISTRY.every(
        (source) => source.requestTimeoutMs >= 1_000 && source.requestTimeoutMs <= 30_000,
      ),
    ).toBe(true)
  })
})
