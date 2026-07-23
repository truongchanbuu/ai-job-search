# Contract: Freelance Search CLI and Skill Interfaces

This contract describes expected behavior for source-specific freelance skills and cross-source freelance workflow commands. Exact command names may be implemented through Codex skills or CLI wrappers, but outputs must follow the normalized records in [data-model.md](../data-model.md).

## Source Adapter: Search

**Purpose**: Search one freelance source for opportunities using profile-derived or explicit query terms.

**Inputs**:

- `query`: Optional search text. Required if no profile or target service is provided.
- `service`: Optional target freelance service.
- `location`: Optional location or remote preference text.
- `budget_min`: Optional minimum budget/rate preference.
- `posted_within_days`: Optional recency filter.
- `limit`: Optional maximum result count.
- `profile_path`: Optional path to the default profile or freelance profile extension.
- `format`: `json`, `table`, or `plain`.

**Expected JSON Output**:

```json
{
  "source": "upwork",
  "status": "available",
  "query": "flutter mobile app bug fix",
  "results": [
    {
      "sourceOpportunityId": "source-specific-id",
      "title": "Fix Flutter app checkout issue",
      "clientOrSource": "Example Client",
      "platform": "upwork",
      "url": "https://example.com/opportunity/1",
      "budget": "$300 fixed",
      "timeline": "1 week",
      "descriptionAvailable": false,
      "language": "en",
      "source": "upwork",
      "requiresVerification": true,
      "riskFlags": []
    }
  ],
  "warnings": []
}
```

**Failure or Setup Output**:

```json
{
  "source": "fiverr",
  "status": "needs_setup",
  "error": "Fiverr account readiness is not configured. Add seller profile and gig readiness notes before platform-specific recommendations."
}
```

**Rules**:

- Source failure must not abort cross-source freelance search.
- Source output must preserve attribution and source-level status.
- If original content has not been verified, `requiresVerification` must be true.
- If account setup, verification, Connects, seller approval, or user-supplied content is needed, status must reflect the blocker.

## Source Adapter: Detail

**Purpose**: Fetch, parse, or accept details for one freelance opportunity.

**Inputs**:

- `source_opportunity_id` or `url`: Required unless the opportunity is user-supplied text.
- `user_supplied_path`: Optional file containing copied opportunity content.
- `format`: `json` or `plain`.

**Expected JSON Output**:

```json
{
  "source": "user_supplied",
  "status": "retrieved",
  "opportunity": {
    "sourceOpportunityId": "manual-2026-07-14-001",
    "title": "Build landing page and payment flow",
    "clientOrSource": "Copied Upwork post",
    "platform": "upwork",
    "url": null,
    "budget": "$500 fixed",
    "timeline": "2 weeks",
    "description": "Full user-supplied opportunity description",
    "requirements": ["Flutter", "Stripe", "API integration"],
    "skills": ["Flutter", "Stripe", "REST API"],
    "language": "en",
    "requiresVerification": false
  },
  "warnings": []
}
```

**Rules**:

- Detail fetch must return `partial` status when only some fields are available.
- Expired, inaccessible, private, or blocked content must be marked clearly.
- User-supplied content may be treated as verified for drafting only when the user provided enough source context.

## Cross-Source Workflow: Freelance Profile Search

**Purpose**: Search enabled freelance sources using the default profile and freelance profile extension, then produce a ranked shortlist.

**Inputs**:

- `profile_path`: Required path to the default profile.
- `freelance_profile_path`: Required path or default location for freelance-specific preferences.
- `sources`: Optional source allowlist.
- `limit_per_source`: Optional source cap.
- `format`: `json`, `table`, or `plain`.

**Expected JSON Output**:

```json
{
  "profileId": "default",
  "sourceStatuses": [
    { "source": "upwork", "status": "available", "count": 8 },
    { "source": "fiverr", "status": "needs_setup", "count": 0 },
    { "source": "google", "status": "available", "count": 5 }
  ],
  "opportunities": [
    {
      "opportunityId": "freelance-generated-id",
      "title": "Flutter bug fix and release support",
      "clientOrSource": "Example Client",
      "platform": "upwork",
      "sources": ["upwork", "google"],
      "matchedSkills": ["Flutter", "Firebase"],
      "budgetFit": "acceptable",
      "platformReadiness": "ready",
      "riskFlags": [],
      "recommendation": "proposal_worthy",
      "existingRecordStatus": null
    }
  ],
  "warnings": []
}
```

**Rules**:

- The workflow must deduplicate opportunities before ranking.
- Existing freelance opportunity records must be surfaced in output.
- Source statuses must be visible even when a source returns no opportunities.
- Google/social results must remain discovery leads until original content is verified.

## Cross-Source Workflow: Account Readiness

**Purpose**: Record or inspect Fiverr and Upwork readiness before recommending platform-specific actions.

**Inputs**:

- `platform`: Required, such as `fiverr` or `upwork`.
- `profile_url`: Optional public platform profile URL.
- `service_categories`: Optional list of services/categories.
- `portfolio_refs`: Optional list of portfolio references.
- `verification_status`: Optional readiness value.
- `known_blockers`: Optional list of blockers.
- `format`: `json`, `table`, or `plain`.

**Expected JSON Output**:

```json
{
  "platform": "upwork",
  "accountStatus": "limited",
  "profileCompleteness": ["title", "overview", "skills", "portfolio"],
  "serviceCategories": ["Mobile App Development"],
  "verificationStatus": "verified",
  "knownBlockers": ["Connects balance unknown"],
  "recommendedAction": "Check Connects before sending proposals."
}
```

**Rules**:

- Passwords, payment credentials, private messages, and secret tokens must never be accepted or written.
- Unknown readiness must be treated as a caveat, not as ready.
- Recommendations must distinguish setup tasks from proposal actions.

## Cross-Source Workflow: Proposal Material

**Purpose**: Produce a truthful proposal draft, response outline, or gig improvement note from a verified opportunity and profile facts.

**Inputs**:

- `opportunity_id` or `opportunity_path`: Required.
- `profile_path`: Required.
- `freelance_profile_path`: Required.
- `material_type`: `proposal`, `response_outline`, `gig_note`, or `follow_up`.
- `language`: Optional; inferred when omitted.
- `format`: `json` or `plain`.

**Expected JSON Output**:

```json
{
  "opportunityId": "freelance-generated-id",
  "materialType": "proposal",
  "language": "en",
  "draftPath": "documents/applications/freelance/example-proposal.md",
  "evidenceRefs": ["profile:experience:flutter-release", "opportunity:requirements:flutter"],
  "unsupportedClaims": [],
  "warnings": []
}
```

**Rules**:

- Unsupported claims must be listed instead of inserted as facts.
- Drafts must not include invented platform ratings, earnings, client facts, project details, or availability.
- Discovery leads must be verified before proposal material can be marked ready.

## Cross-Source Workflow: Save Freelance Opportunity

**Purpose**: Save or update a freelance opportunity history entry.

**Inputs**:

- `opportunity_id` or `opportunity_path`: Required.
- `status`: Required.
- `material_refs`: Optional proposal or response references.
- `notes`: Optional.

**Expected JSON Output**:

```json
{
  "recordId": "freelance-record-id",
  "opportunityId": "freelance-generated-id",
  "status": "proposal_drafted",
  "savedAt": "2026-07-14",
  "updatedAt": "2026-07-14",
  "materialRefs": ["documents/applications/freelance/example-proposal.md"],
  "duplicateOf": null
}
```

**Rules**:

- Updating one record must not rewrite unrelated records.
- If an opportunity already exists in history, output must identify the existing record.
- Scam, ignored, and archived states must remain searchable for duplicate prevention.
