# Contract: Vietnam Portal Adapter CLI

Each new portal skill exposes the same command surface even when its configured
access mode is manual or restricted.

## Commands

```text
bun run src/cli.ts search [flags]
bun run src/cli.ts detail <source-id|url> [flags]
```

### Search flags

- `--query, -q <text>`: Role/keyword expression.
- `--location, -l <text>`: Repeatable or portal-mapped location.
- `--max-experience <years>`: Maximum accepted required experience.
- `--seniority <value>`: Requested level.
- `--jobage <days>`: Maximum posting age.
- `--work-mode <remote|hybrid|onsite>`: Requested arrangement.
- `--employment-type <value>`: Requested employment type.
- `--limit, -n <number>`: Discovery cap.
- `--format <json|table|plain>`: Default `json`.

### Detail flags

- `--format <json|plain>`: Default `json`.

## Search JSON

```json
{
  "contractVersion": "1",
  "source": "itviec",
  "accessMode": "enabled_public",
  "status": "searched",
  "query": {
    "query": "Java React Vue",
    "location": ["Ho Chi Minh City"],
    "maxExperience": 1,
    "jobage": 30
  },
  "appliedConstraints": {
    "query": "Java React Vue",
    "location": ["ho-chi-minh-hcm"],
    "seniority": ["fresher", "junior"]
  },
  "unsupportedConstraints": ["maxExperience"],
  "manualReviewUrl": null,
  "meta": {
    "count": 1,
    "page": 1,
    "total": 1
  },
  "results": [
    {
      "id": "3413",
      "source": "itviec",
      "title": "Software Developer Fresher/Junior",
      "company": "Example Company",
      "location": "Ho Chi Minh City",
      "date": "2026-07-28",
      "deadline": null,
      "url": "https://itviec.com/it-jobs/example-company-3413",
      "discoveredAt": "2026-07-28T10:00:00+07:00",
      "verificationStatus": "unverified_lead",
      "requiresVerification": true
    }
  ],
  "warnings": []
}
```

Allowed search statuses:

- `searched`: Recognized successful result page with results.
- `no_matches`: Recognized successful result page with zero valid results.
- `manual_required`: Policy or source design requires user review/input.
- `restricted`: Access control such as 401/403/login/CAPTCHA blocked retrieval.
- `unavailable`: Portal/network is currently unavailable.
- `failed`: Request or parsing failed for another reason.

For `manual_required`, the adapter returns exit code `0`,
`manualReviewUrl`, zero results, and an explanatory warning. Expected access
limitations are not process failures.

## Detail JSON

```json
{
  "contractVersion": "1",
  "source": "topcv",
  "status": "verified_active",
  "job": {
    "id": "1954519",
    "source": "topcv",
    "url": "https://www.topcv.vn/viec-lam/example/1954519.html",
    "title": "Fresher Java Developer",
    "company": "Example Company",
    "location": "Ha Noi",
    "descriptionSummary": "Normalized decision-relevant facts.",
    "skills": ["Java"],
    "experienceEvidence": [
      {
        "value": "6 months to 1 year",
        "evidenceKind": "explicit_label"
      }
    ],
    "minExperienceYears": 0.5,
    "maxExperienceYears": 1,
    "seniority": "fresher",
    "employmentType": "full-time",
    "workMode": null,
    "postedAt": null,
    "deadline": "2026-08-15",
    "salary": "As stated by source",
    "applyUrl": null,
    "language": "vi",
    "activeEvidence": ["application deadline is in the future"],
    "closedEvidence": [],
    "verifiedAt": "2026-07-28T10:01:00+07:00"
  },
  "verification": {
    "status": "verified_active",
    "detailAccessible": true,
    "active": true,
    "descriptionSufficient": true,
    "experienceDecisionSupported": true,
    "reasonCodes": []
  },
  "warnings": []
}
```

Allowed detail statuses:

- `verified_active`
- `insufficient_detail`
- `expired`
- `removed`
- `restricted`
- `failed`

## Error contract

Invalid commands/arguments and unexpected adapter failures write one JSON object
to stderr and exit `1`:

```json
{
  "error": "--max-experience must be a non-negative number",
  "code": "BAD_ARG",
  "source": "topcv"
}
```

Expected codes include `BAD_ARG`, `BAD_CMD`, `NO_ID`, `BAD_ID`,
`NOT_FOUND`, `RESTRICTED`, `SEARCH_FAILED`, `DETAIL_FAILED`, and
`PARSE_FAILED`.

## Access behavior

- CareerViet `search` produces `manual_required` and an official search URL.
- Vieclam24h and VietnamWorks produce `manual_required` or `restricted`; they do
  not retry around 403/challenge behavior.
- TopCV and ITviec use low-volume public fetches while the registry policy
  remains current.
- `detail` may accept a user-supplied public URL or pasted-content path where
  the source's configured capability permits it.
- No adapter reads credentials, authenticated cookies, private APIs, or mobile
  traffic.

## Content and parsing rules

- Prefer public structured data/JSON-LD, then narrow source-specific parsing.
- Strip tracking parameters from canonical URLs.
- Missing values are `null`.
- Search cards remain `unverified_lead`.
- A 404/closed marker is not active.
- A 401/403/login/CAPTCHA/challenge is not zero results.
- Persist normalized facts/evidence and canonical attribution, not raw HTML or
  full mirrored descriptions.
- Explicit description requirements override title, category, and filter labels.

## Legacy compatibility

The `job-scrape` orchestrator also accepts current LinkedIn/FreeHire search
output (`{meta, results}`) and detail objects, then adds source/status fields at
its normalization boundary. This feature does not change those existing CLI
contracts.
