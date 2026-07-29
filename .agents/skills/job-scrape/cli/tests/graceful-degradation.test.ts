import { describe, expect, test } from "bun:test"
import { classifyProcessFailure } from "../src/commands/search.js"

describe("graceful degradation", () => {
  test("does not classify restriction or timeout as no matches", () => {
    expect(classifyProcessFailure('{"code":"RESTRICTED"}', false)).toBe("restricted")
    expect(classifyProcessFailure("", true)).toBe("unavailable")
    expect(classifyProcessFailure('{"code":"SEARCH_FAILED"}', false)).toBe("failed")
  })
})
