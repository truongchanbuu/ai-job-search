import { describe, expect, test } from "bun:test"
import { criteriaFromFlags, parseFlags } from "../src/cli.js"
import { buildAdapterArgs } from "../src/commands/search.js"
import { PORTAL_REGISTRY } from "../src/portal-registry.js"

describe("criteria preservation", () => {
  test("keeps technology names, Vietnamese text, aliases, and exclusions intact", () => {
    const criteria = criteriaFromFlags(
      parseFlags([
        "search",
        "--query",
        "Java backend lập trình viên ReactJS VueJS",
        "--skill",
        "ReactJS",
        "--exclude",
        "senior",
        "--max-experience",
        "1",
      ]),
    )
    expect(criteria.queryTerms[0]).toBe(
      "Java backend lập trình viên ReactJS VueJS",
    )
    expect(criteria.skills).toEqual(["ReactJS"])
    expect(criteria.exclusions).toEqual(["senior"])
    const args = buildAdapterArgs(criteria, PORTAL_REGISTRY[0])
    expect(args.join(" ")).toContain("ReactJS")
  })
})
