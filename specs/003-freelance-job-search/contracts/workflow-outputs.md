# Contract: Freelance Workflow Output Files

This contract defines expected workspace files created or updated by the freelance job search feature.

## Personal Output Locations

All files containing personal profile data, account readiness, proposal drafts, opportunity history, copied listings, notes, or outcomes must be stored in gitignored personal-output paths.

Recommended locations:

- `documents/applications/freelance/` for proposal drafts, response outlines, gig notes, and archived opportunity material.
- `job_scraper/` for seen opportunity state, deduplication state, and scrape notes.
- Existing tracker files or a freelance-specific tracker file for opportunity status and outcomes.

## Freelance Profile Extension

**Purpose**: Store repeatable freelance search preferences without duplicating the default profile.

**Required content**:

- Target services.
- Rate and budget preferences.
- Availability.
- Portfolio references.
- Work type preferences.
- Platform profile references.
- Exclusions and constraints.

**Rules**:

- Must reference verified profile facts rather than duplicating unverified claims.
- Must not store passwords, payment credentials, private messages, or secret tokens.

## Platform Account Readiness Notes

**Purpose**: Track whether Fiverr and Upwork are actionable.

**Required content**:

- Platform.
- Account status.
- Profile completeness.
- Service categories.
- Verification status.
- Known blockers.
- Last update date.

**Rules**:

- Unknown readiness must be represented explicitly.
- Setup blockers must be visible in search and proposal recommendations.

## Opportunity History

**Purpose**: Preserve source references, status, materials, and outcomes for freelance leads.

**Required content**:

- Stable record ID.
- Opportunity identity.
- Source references.
- Current status.
- Saved and updated dates.
- Platform.
- Budget details when available.
- Material references.
- Notes and outcome when known.

**Rules**:

- Duplicate opportunities must point to the existing record.
- Status updates must not erase original source context.
- Scam/ignored/archive decisions must remain available for future duplicate prevention.

## Proposal Materials

**Purpose**: Store drafted proposals, response outlines, gig notes, or follow-ups.

**Required content**:

- Linked opportunity ID.
- Linked profile ID.
- Material type.
- Language.
- Evidence references.
- Unsupported claims list.
- Verification status.

**Rules**:

- Drafts must not include unsupported freelancer claims, platform ratings, earnings, client facts, project requirements, or availability.
- Drafts based on discovery leads must remain draft/needs-confirmation until original content is verified.
