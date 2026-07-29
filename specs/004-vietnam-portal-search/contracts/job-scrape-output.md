# Contract: Cross-Source Job Scrape Output

## Command

```text
bun run .agents/skills/job-scrape/cli/src/cli.ts search [flags]
```

Flags:

- All normalized criteria from the portal adapter contract.
- `--source <id>`: Repeatable allowlist; default uses configured sources.
- `--profile-path <path>`: Optional candidate profile input.
- `--seen-path <path>`: Default `job_scraper/seen_jobs.json`.
- `--tracker-path <path>`: Default `job_search_tracker.csv`.
- `--save-session`: Persist the private JSON report.
- `--format <json|table|plain>`: Default `json`.

## JSON output

```json
{
  "contractVersion": "1",
  "runId": "20260728T100000-abc123",
  "startedAt": "2026-07-28T10:00:00+07:00",
  "completedAt": "2026-07-28T10:00:12+07:00",
  "criteria": {
    "queryTerms": ["Java backend", "ReactJS", "VueJS"],
    "locations": ["Vietnam"],
    "maxExperienceYears": 1,
    "postedWithinDays": 30,
    "workModes": [],
    "employmentTypes": [],
    "exclusions": []
  },
  "sourceStatuses": [
    {
      "source": "itviec",
      "status": "searched",
      "appliedConstraints": ["query", "location", "seniority"],
      "unsupportedConstraints": ["maxExperience"],
      "rawDiscoveries": 10,
      "verifiedActive": 3,
      "duplicates": 1,
      "expiredOrRemoved": 1,
      "inaccessibleLeads": 5,
      "actionable": 2,
      "reason": null,
      "manualReviewUrl": null
    },
    {
      "source": "careerviet",
      "status": "manual_required",
      "appliedConstraints": [],
      "unsupportedConstraints": ["automatedSearch"],
      "rawDiscoveries": 0,
      "verifiedActive": 0,
      "duplicates": 0,
      "expiredOrRemoved": 0,
      "inaccessibleLeads": 0,
      "actionable": 0,
      "reason": "Published terms require ordinary browser/manual search.",
      "manualReviewUrl": "https://careerviet.vn/viec-lam/..."
    }
  ],
  "jobs": [
    {
      "jobId": "job_abc123",
      "title": "Junior Java Developer",
      "employer": "Example Company",
      "location": "Ho Chi Minh City",
      "verificationStatus": "verified_active",
      "sources": [
        {
          "source": "itviec",
          "id": "1234",
          "url": "https://itviec.com/it-jobs/example-1234"
        },
        {
          "source": "topcv",
          "id": "1954519",
          "url": "https://www.topcv.vn/viec-lam/example/1954519.html"
        }
      ],
      "conflicts": [],
      "fit": {
        "eligibility": "eligible",
        "experienceDecision": "within_limit",
        "experienceEvidenceRefs": ["itviec:1234"],
        "matchedSkills": ["Java"],
        "missingSkills": [],
        "hardConstraintViolations": [],
        "recommendation": "strong",
        "rationale": "Verified source evidence supports the stated limit."
      },
      "existingSeenState": null,
      "existingApplicationStatus": null
    }
  ],
  "unverifiedLeads": [],
  "coverage": {
    "totalSources": 5,
    "statusCounts": {
      "searched": 2,
      "no_matches": 0,
      "manual_required": 3,
      "restricted": 0,
      "unavailable": 0,
      "failed": 0
    },
    "rawDiscoveries": 20,
    "verifiedActive": 6,
    "duplicates": 2,
    "expiredOrRemoved": 2,
    "inaccessibleLeads": 10,
    "actionableMatches": 4
  },
  "warnings": []
}
```

## Orchestration rules

1. Start enabled public adapters independently and preferably concurrently.
2. Apply a bounded timeout per source.
3. Convert child errors into that source's result; never abort sibling sources.
4. Preserve requested, applied, and unsupported constraints.
5. Fetch detail only for promising discoveries within the source cap.
6. Verify active status and sufficient requirements before fit screening.
7. Deduplicate before ranking while preserving every source copy.
8. Attach seen/application state when private files exist.
9. Produce exactly one `sourceStatuses` entry per enabled source.
10. Save only when `--save-session` is present.

## Verification and fit contract

- Search cards and indexed snippets go to `unverifiedLeads`.
- Only `verified_active` jobs may have non-null fit recommendations.
- Experience-constrained recommendations require source-backed experience
  evidence.
- Explicit description requirements override title/category labels.
- Missing salary, deadline, work mode, employer, or location stays `null`.
- Expired, removed, restricted, and insufficient-detail records never appear as
  actionable matches.

## Deduplication contract

- Exact URL/source IDs merge first.
- Existing state source references merge before fuzzy identity checks.
- Employer/title/location matching must be strong and may use description
  evidence.
- Title similarity alone never merges.
- Conflicting source values remain under `conflicts`.
- A preferred display value includes an evidence-based reason or stays
  unresolved.

## Private output files

When `--save-session` is set:

```text
job_scraper/sessions/<run-id>.json
```

`job_scraper/seen_jobs.json` schema:

```json
{
  "contractVersion": "1",
  "records": [
    {
      "jobId": "job_abc123",
      "canonicalKey": "example-company|junior-java-developer|ho-chi-minh-city",
      "sourceRefs": [
        {
          "source": "itviec",
          "id": "1234",
          "url": "https://itviec.com/it-jobs/example-1234"
        }
      ],
      "state": "seen",
      "firstSeenAt": "2026-07-28T10:00:12+07:00",
      "lastSeenAt": "2026-07-28T10:00:12+07:00",
      "notes": null
    }
  ]
}
```

Readers tolerate a missing seen/tracker file. Upserts preserve unrelated
records and the original `firstSeenAt`. Search does not rewrite
`job_search_tracker.csv`.

## Table/plain output

Human formats include:

- A portal coverage section with status, filters, counts, and reason.
- An actionable shortlist with source links, evidence-backed fit, and existing
  state.
- A separate unverified-lead section with no score.
- Warnings for conflicts, missing facts, and access limitations.
