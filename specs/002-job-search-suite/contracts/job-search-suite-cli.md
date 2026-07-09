# Contract: Job Search Suite CLI and Skill Interfaces

This contract describes the expected behavior of source-specific portal CLIs and cross-source job workflow commands. Exact command names may be implemented through Codex skills or CLI wrappers, but outputs must follow the normalized records in [data-model.md](../data-model.md).

## Source Adapter: Search

**Purpose**: Search one source for jobs using profile-derived or explicit query terms.

**Inputs**:

- `query`: Optional search text. Required if no profile is provided.
- `location`: Optional location filter.
- `remote`: Optional work-mode filter.
- `posted_within_days`: Optional recency filter.
- `limit`: Optional maximum result count.
- `profile_path`: Optional path to the default profile.
- `format`: `json`, `table`, or `plain`.

**Expected JSON Output**:

```json
{
  "source": "linkedin",
  "status": "available",
  "query": "backend engineer",
  "results": [
    {
      "sourceJobId": "source-specific-id",
      "title": "Backend Engineer",
      "employer": "Example Co",
      "location": "Ho Chi Minh City",
      "url": "https://example.com/jobs/1",
      "postedAt": "2026-07-01",
      "descriptionAvailable": false,
      "language": "en",
      "source": "linkedin"
    }
  ],
  "warnings": []
}
```

**Failure Output**:

```json
{
  "source": "facebook",
  "status": "needs_setup",
  "error": "Source requires user-supplied page/group content or accessible source configuration."
}
```

**Rules**:

- Source failure must not abort cross-source search.
- Source output must include enough information to preserve attribution.
- If a description is not retrieved, `descriptionAvailable` must be false.

## Source Adapter: Detail

**Purpose**: Fetch or parse details for one job result.

**Inputs**:

- `source_job_id` or `url`: Required.
- `format`: `json` or `plain`.

**Expected JSON Output**:

```json
{
  "source": "itviec",
  "status": "retrieved",
  "job": {
    "sourceJobId": "source-specific-id",
    "title": "Senior Developer",
    "employer": "Example Co",
    "location": "Da Nang",
    "url": "https://example.com/jobs/2",
    "description": "Full job description text",
    "requirements": ["Requirement 1"],
    "skills": ["TypeScript", "API design"],
    "language": "en"
  },
  "warnings": []
}
```

**Rules**:

- Detail fetch must return `partial` status when only some fields are available.
- Expired or inaccessible postings must be marked clearly.

## Cross-Source Workflow: Profile Search

**Purpose**: Search enabled sources using the default profile and produce a ranked shortlist.

**Inputs**:

- `profile_path`: Required path to default profile.
- `sources`: Optional source allowlist.
- `limit_per_source`: Optional source cap.
- `format`: `json`, `table`, or `plain`.

**Expected JSON Output**:

```json
{
  "profileId": "default",
  "sourceStatuses": [
    { "source": "linkedin", "status": "available", "count": 10 },
    { "source": "topcv", "status": "unavailable", "count": 0 }
  ],
  "jobs": [
    {
      "jobId": "generated-id",
      "title": "Backend Engineer",
      "employer": "Example Co",
      "location": "Ho Chi Minh City",
      "sources": ["linkedin", "google"],
      "matchedSkills": ["TypeScript", "Node.js"],
      "recommendation": "strong",
      "existingApplicationStatus": null
    }
  ],
  "warnings": []
}
```

**Rules**:

- The workflow must deduplicate results before ranking.
- Existing application history matches must be surfaced in output.
- Source statuses must be visible even when a source returns no jobs.

## Cross-Source Workflow: ATS Keyword Review

**Purpose**: Compare one CV draft with one job description.

**Inputs**:

- `job_path` or `job_id`: Required.
- `cv_path` or `material_id`: Required.
- `profile_path`: Required.
- `language`: Optional; inferred when omitted.

**Expected JSON Output**:

```json
{
  "jobId": "generated-id",
  "materialId": "cv-en",
  "matchedTerms": ["API design", "TypeScript"],
  "missingTerms": ["system design"],
  "weakTerms": ["testing"],
  "overusedTerms": [],
  "unsupportedTerms": ["Kubernetes"],
  "recommendations": [
    {
      "term": "system design",
      "action": "strengthen",
      "evidence": "profile experience highlight reference",
      "suggestedSection": "Experience"
    }
  ]
}
```

**Rules**:

- Unsupported terms must not be recommended as factual additions.
- Recommendations must identify whether the profile has evidence for the term.

## Cross-Source Workflow: Save Application

**Purpose**: Save or update an application history entry.

**Inputs**:

- `job_id` or `job_path`: Required.
- `status`: Required.
- `material_refs`: Optional generated material paths.
- `notes`: Optional.

**Expected JSON Output**:

```json
{
  "applicationId": "generated-id",
  "jobId": "generated-id",
  "status": "applied",
  "savedAt": "2026-07-09",
  "updatedAt": "2026-07-09",
  "materialRefs": ["cv/main_example_company.tex"],
  "duplicateOf": null
}
```

**Rules**:

- Updating one application must not rewrite unrelated records.
- If an application already exists for the job, output must identify the existing record.

## Cross-Source Workflow: Interview Prep

**Purpose**: Generate interview questions and prompts for a tracked application.

**Inputs**:

- `application_id` or `job_id`: Required.
- `profile_path`: Required.
- `related_posts_path`: Optional.
- `language`: Optional; supports English and Vietnamese.

**Expected JSON Output**:

```json
{
  "applicationId": "generated-id",
  "language": "en",
  "questionGroups": [
    {
      "group": "technical",
      "questions": [
        {
          "question": "How have you designed APIs for high-change product requirements?",
          "basis": "job_requirement",
          "sourceRef": "job-description"
        }
      ]
    }
  ],
  "missingContext": [],
  "warnings": []
}
```

**Rules**:

- At least 10 questions should be produced when enough job context exists.
- Related-post themes must be labeled as context, not verified fact.
