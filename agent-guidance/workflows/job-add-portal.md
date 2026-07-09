# Job Add Portal Workflow

Canonical workflow for generating a new job portal search skill.

## Steps

1. Interview for market, portal URL, role categories, and language.
2. Investigate search URL patterns, result markup or APIs, pagination, detail pages, and access constraints.
3. Scaffold a new `.agents/skills/<portal>-search/` skill with a CLI.
4. Test a live query.
5. Add the portal to search guidance when requested.

## Rules

- Respect portal access rules.
- The generated skill must support search and detail lookup when feasible.
