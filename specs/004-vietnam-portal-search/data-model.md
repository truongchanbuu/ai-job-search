# Data Model: Vietnam Job Portal Search Coverage

## Overview

The model separates persistent portal capabilities, one search run's source
outcomes, source-specific listing evidence, canonical cross-source jobs, and
fit results. Unknown facts remain `null`; conflicts remain attached to their
sources.

## PortalSource

Static registry entry for one adapter.

**Fields**:

- `sourceId`: Stable enum value:
  `careerviet | vieclam24h | topcv | itviec | vietnamworks` plus registered
  existing sources.
- `displayName`: User-facing portal name.
- `baseUrl`: Official portal URL.
- `accessMode`: `enabled_public | manual_only | restricted`.
- `enabledByDefault`: Whether a normal Vietnam search attempts the source.
- `searchCapability`: `automated | official_link | manual`.
- `detailCapability`: `automated_public | user_supplied_public | pasted_text`.
- `supportedConstraints`: Set of `query | location | experience | seniority |
  recency | workMode | employmentType | salary`.
- `policyNotes`: Concise non-bypass and content-retention rules.
- `accessReviewedAt`: Date current access behavior/terms were last reviewed.
- `requestTimeoutMs`: Bounded source execution timeout.

**Validation**:

- Every named portal must have exactly one registry entry.
- `manual_only` and `restricted` sources cannot execute unattended search-page
  retrieval.
- A source must not advertise a filter it cannot map.

## SearchCriteria

Normalized user/profile request supplied to the orchestrator.

**Fields**:

- `queryTerms`: Ordered role and free-text terms.
- `skills`: Technology names and aliases, preserving original spelling.
- `locations`: Requested locations.
- `maxExperienceYears`: Maximum accepted required experience, or `null`.
- `seniority`: Requested levels.
- `postedWithinDays`: Recency limit, or `null`.
- `workModes`: Requested remote/hybrid/onsite modes.
- `employmentTypes`: Requested employment types.
- `exclusions`: Explicit forbidden roles, skills, locations, or conditions.
- `languageTerms`: Original Vietnamese/English mixed terms.
- `limitPerSource`: Discovery cap.

**Validation**:

- Technology terms are not semantically translated or weakened.
- Missing profile constraints remain absent rather than inferred.

## PortalSearchRun

One invocation across one or more sources.

**Fields**:

- `runId`: Stable timestamp/random identifier.
- `startedAt`, `completedAt`: ISO timestamps.
- `criteria`: `SearchCriteria`.
- `enabledSources`: Ordered source IDs.
- `sourceRuns`: Array of `PortalRunResult`.
- `canonicalJobs`: Array of `CanonicalJobListing`.
- `coverage`: `SourceCoverageSummary`.
- `warnings`: Run-level warnings.
- `sessionPath`: Private persisted report path when saved.

**Validation**:

- Every enabled source has exactly one `PortalRunResult`, including on timeout.
- One source failure cannot prevent other source results from appearing.

## PortalRunResult

Dynamic outcome for one source in one run.

**Fields**:

- `sourceId`: Portal registry reference.
- `status`: `searched | no_matches | manual_required | restricted |
  unavailable | failed`.
- `startedAt`, `completedAt`: ISO timestamps.
- `appliedConstraints`: Constraint/value pairs actually sent to the portal.
- `unsupportedConstraints`: Requested constraints not applied by the portal.
- `manualReviewUrl`: Official search/review URL when applicable.
- `rawDiscoveryCount`: Valid search cards before verification.
- `verifiedActiveCount`: Active, sufficient details retrieved.
- `duplicateCount`: Source records merged into another canonical job.
- `expiredOrRemovedCount`: Closed/stale records.
- `inaccessibleLeadCount`: Title/snippet/manual leads not verified.
- `actionableCount`: Canonical source contributions eligible for fit.
- `listings`: Array of `SourceListing`.
- `reasonCode`: Structured reason for non-success.
- `message`: Safe user-facing explanation.
- `warnings`: Parser/filter/access caveats.

**Validation**:

- A 401/403/login/CAPTCHA/challenge response cannot map to `no_matches`.
- `no_matches` requires a recognized successful result page with zero valid
  cards.
- Counts must be internally consistent and non-negative.

## SourceListing

One portal's representation of a vacancy.

**Identity and provenance**:

- `sourceId`
- `sourceJobId`: Portal identifier or `null`.
- `sourceUrl`: Original public reference.
- `canonicalUrl`: Tracking-free normalized reference.
- `discoveredAt`
- `verifiedAt`: ISO timestamp or `null`.

**Observed fields**:

- `title`
- `employer`
- `location`
- `descriptionSummary`: Bounded normalized evidence, not a full page mirror.
- `skills`
- `experienceEvidence`: Array of source excerpts/facts relevant to experience.
- `minExperienceYears`, `maxExperienceYears`: Parsed values or `null`.
- `seniority`
- `employmentType`
- `workMode`
- `postedAt`
- `deadline`
- `salary`
- `applyUrl`
- `language`
- `activeEvidence`
- `closedEvidence`

**Quality fields**:

- `discoveryKind`: `search_result | indexed_lead | user_supplied_url |
  pasted_description`.
- `verification`: `ListingVerification`.
- `rawFieldEvidence`: Array of `FieldEvidence`.
- `warnings`

**Validation**:

- `sourceId`, `sourceUrl` or user-supplied identifier, and `discoveredAt` are
  required.
- Unknown values are `null`, never inferred placeholders.
- Full raw HTML and full copied descriptions are not persisted.

## ListingVerification

Decision gate between discovery and fit screening.

**Fields**:

- `status`: `unverified_lead | verified_active | insufficient_detail |
  expired | removed | restricted | failed`.
- `detailAccessible`: Boolean.
- `active`: `true | false | null`.
- `descriptionSufficient`: Boolean.
- `experienceDecisionSupported`: Boolean.
- `checkedAt`: ISO timestamp.
- `reasonCodes`: Array such as `TITLE_ONLY`, `LOGIN_REQUIRED`, `HTTP_403`,
  `CHALLENGE_PAGE`, `NOT_FOUND`, `CLOSED_MARKER`, `NO_REQUIREMENTS`,
  `EXPERIENCE_UNKNOWN`.

**Transitions**:

```text
unverified_lead
  -> verified_active
  -> insufficient_detail
  -> expired
  -> removed
  -> restricted
  -> failed

verified_active -> expired | removed | restricted
```

Only `verified_active` may enter fit screening. If a max-experience constraint
exists, `experienceDecisionSupported` must also be true.

## FieldEvidence

Source-backed value for one decision-critical field.

**Fields**:

- `field`: Field name.
- `value`: Normalized value.
- `sourceId`
- `sourceUrl`
- `evidenceKind`: `detail_text | structured_data | explicit_label |
  user_supplied_text`.
- `observedAt`
- `confidence`: `direct | parsed | ambiguous`.

**Validation**:

- `direct` requires an explicit source value.
- Search-category labels alone cannot be direct experience evidence.

## CanonicalJobListing

Conservative merge of one or more `SourceListing` records.

**Fields**:

- `jobId`: Workspace-generated stable ID.
- `canonicalKey`: Stable normalized identity key.
- `sourceListings`: Non-empty array of source listing references.
- `title`, `employer`, `location`: Selected display values with evidence.
- `skills`, `experience`, `seniority`, `employmentType`, `workMode`,
  `postedAt`, `deadline`, `salary`: Evidence-backed consolidated facts.
- `conflicts`: Array of `FieldConflict`.
- `verificationStatus`: Derived canonical verification status.
- `existingSeenState`: `new | seen | dismissed | shortlisted | null`.
- `existingApplicationStatus`: Tracker status or `null`.
- `fitEvidence`: `FitEvidence | null`.

**Deduplication layers**, evaluated in order:

1. Exact normalized source URL or source ID.
2. Existing seen/application source reference.
3. Strong normalized employer + title + location identity.
4. Strong employer/title match plus supporting description/evidence similarity.

**Validation**:

- Similar titles alone never merge.
- Merging unions source references rather than discarding a copy.
- Canonical verification is actionable only when at least one source copy is
  `verified_active`.

## FieldConflict

Preserved disagreement between source copies.

**Fields**:

- `field`
- `values`: Array of `{value, sourceId, sourceUrl, observedAt}`.
- `preferredValue`: Value selected for display or `null`.
- `preferenceReason`: Evidence-based reason or `UNRESOLVED`.

**Validation**:

- All conflicting values remain available.
- The system does not silently select a value without a reason.

## FitEvidence

Source-backed screen against the candidate/search constraints.

**Fields**:

- `eligibility`: `eligible | ineligible | insufficient_evidence`.
- `matchedSkills`
- `missingSkills`
- `experienceDecision`: `within_limit | exceeds_limit | explicit_exception |
  unknown`.
- `experienceEvidenceRefs`
- `locationDecision`
- `workModeDecision`
- `employmentTypeDecision`
- `hardConstraintViolations`
- `recommendation`: `strong | possible | weak | skip | null`.
- `rationale`

**Validation**:

- A recommendation requires canonical `verified_active` status.
- If `maxExperienceYears` is present, experience evidence refs are required.
- Explicit job-description requirements override title/category labels.
- Missing facts cannot become matches.

## SeenJobState

Record in private `job_scraper/seen_jobs.json`.

**Fields**:

- `jobId`
- `canonicalKey`
- `sourceRefs`
- `state`: `seen | dismissed | shortlisted`.
- `firstSeenAt`, `lastSeenAt`
- `notes`

Readers accept either a raw array or `{ "records": [...] }` for migration
tolerance. Upserts preserve unrelated records and `firstSeenAt`.

## ApplicationHistoryLink

Read-only normalized view of a matching `job_search_tracker.csv` row.

**Fields**:

- `applicationId`
- `status`
- `sourceRefs`
- `employer`
- `title`
- `location`
- `appliedAt`

The feature does not define or migrate unrelated tracker columns. Missing files
are normal and produce no matches.

## SourceCoverageSummary

User-facing aggregate derived from `PortalRunResult`.

**Fields**:

- `totalSources`
- `statusCounts`
- `rawDiscoveries`
- `verifiedActive`
- `duplicates`
- `expiredOrRemoved`
- `inaccessibleLeads`
- `actionableMatches`
- `sourceRows`: Source, status, filters applied/unapplied, counts, and reason.

**Validation**:

- Totals derive from source runs rather than being independently mutated.
- The summary distinguishes zero matches from restriction/failure.
