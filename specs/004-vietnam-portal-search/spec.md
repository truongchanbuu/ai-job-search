# Feature Specification: Vietnam Job Portal Search Coverage

**Feature Branch**: _Not created (no branch hook configured)_

**Created**: 2026-07-28

**Status**: Draft

**Input**: User description: "The current project does not find jobs from CareerViet, Vieclam24h, TopCV, ITviec, or VietnamWorks."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Search Major Vietnam Job Portals (Priority: P1)

A job seeker wants one job-search request to check CareerViet, Vieclam24h, TopCV, ITviec, and VietnamWorks so relevant Vietnamese listings are not missed when broader international sources have sparse local coverage.

**Why this priority**: The feature exists to close the current discovery gap. Searching the five named portals is the minimum independently useful outcome.

**Independent Test**: Can be tested by running a search with a role, skills, experience level, and location, then confirming that every enabled named portal is attempted and returns either matching listings or a clear source status.

**Acceptance Scenarios**:

1. **Given** a candidate profile and search criteria, **When** the user starts a job search, **Then** the search attempts CareerViet, Vieclam24h, TopCV, ITviec, and VietnamWorks and identifies the source of every result.
2. **Given** a role query containing English or Vietnamese terms, **When** supported portals are searched, **Then** the results include relevant listings that use either language without changing the meaning of the user's required technologies or constraints.
3. **Given** a portal has no matching public listings, **When** the search completes, **Then** that portal is reported as successfully searched with no matches rather than silently omitted.
4. **Given** one portal is unavailable or access-restricted, **When** the search completes, **Then** results from the remaining portals are still returned and the affected portal has a clear status.

---

### User Story 2 - Verify Listings Before Fit Screening (Priority: P1)

A job seeker wants each recommended result to be backed by an accessible job description so they can trust experience, skill, location, deadline, and employment details rather than relying on a title or search snippet.

**Why this priority**: Unverified or expired listings waste application effort and can produce misleading fit recommendations.

**Independent Test**: Can be tested by reviewing a mixed result set containing accessible, expired, incomplete, and access-restricted listings and confirming that only sufficiently verified active listings receive a fit recommendation.

**Acceptance Scenarios**:

1. **Given** an accessible job listing, **When** it is prepared for screening, **Then** the result includes the original source reference and all available decision-critical details.
2. **Given** a listing title says "junior" or "fresher" but its description requires more experience than the user's limit, **When** the listing is screened, **Then** the explicit experience requirement takes precedence over the title.
3. **Given** a listing is expired, removed, or cannot be opened beyond a discovery snippet, **When** results are presented, **Then** it is excluded from actionable recommendations or clearly labeled as an unverified lead without a fit score.
4. **Given** a source does not state a salary, deadline, or work arrangement, **When** the listing is shown, **Then** the missing field remains unknown and is not inferred.

---

### User Story 3 - Receive One Deduplicated, Transparent Shortlist (Priority: P2)

A job seeker wants repeated postings across the five portals and existing search sources consolidated into one result while preserving source links and prior application history.

**Why this priority**: The same Vietnamese vacancy is often syndicated across multiple boards. Consolidation keeps the shortlist useful and prevents repeated review or duplicate applications.

**Independent Test**: Can be tested with repeated versions of the same vacancy from different portals and an existing tracked application, confirming that one canonical result is shown with all source and history references.

**Acceptance Scenarios**:

1. **Given** the same vacancy appears on multiple portals, **When** results are consolidated, **Then** one job record is shown with every discovered source preserved.
2. **Given** similar titles belong to different employers or locations, **When** results are consolidated, **Then** distinct vacancies remain separate.
3. **Given** a discovered vacancy already exists in seen-job state or application history, **When** results are presented, **Then** its prior state is visible and it is not treated as a new unreviewed job.
4. **Given** source copies disagree on a decision-critical field, **When** the consolidated result is shown, **Then** the disagreement is visible and the most reliable available source is identified without discarding the alternative evidence.

---

### User Story 4 - Understand Portal Coverage and Search Limitations (Priority: P2)

A job seeker wants a concise per-portal search summary so they know which sources were covered, which filters were applied, and why a source produced no actionable results.

**Why this priority**: Transparent coverage prevents an empty result set from being mistaken for proof that no jobs exist.

**Independent Test**: Can be tested by running a search where portals have different filter support and availability, then reviewing the source summary without inspecting internal logs.

**Acceptance Scenarios**:

1. **Given** portals support different combinations of recency, experience, location, and work-arrangement criteria, **When** a search completes, **Then** the user can see which constraints each portal applied or could not apply.
2. **Given** a portal requires login, presents a human-verification challenge, or blocks public detail access, **When** the search encounters it, **Then** the source is marked restricted and the system does not claim full coverage.
3. **Given** a source returns raw results but none survive verification or fit filtering, **When** the summary is shown, **Then** it distinguishes raw discoveries from actionable matches.

### Edge Cases

- A portal changes its page structure, search behavior, public availability, or listing identifiers.
- A portal returns a successful page with zero results, a generic category page, advertisements, or unrelated content.
- A listing is duplicated within one portal under different links or reposted with a new identifier.
- The same employer posts several roles with similar titles but different teams, locations, or experience levels.
- Search criteria contain technology aliases, spelling variations, Vietnamese diacritics, or mixed English and Vietnamese terms.
- A listing states an experience range whose upper bound exceeds the user's limit but explicitly accepts fresh graduates or internship experience.
- A listing has a future deadline but the source labels it closed, or it appears active while the original employer page is unavailable.
- Salary, location, work arrangement, employer, posting date, or deadline is missing or inconsistent across source copies.
- A portal rate-limits requests or temporarily fails after returning some results.
- A source requires authentication, a paid account, or a human-verification challenge; protected access must not be bypassed.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST support CareerViet, Vieclam24h, TopCV, ITviec, and VietnamWorks as independently reportable job-discovery sources.
- **FR-002**: Users MUST be able to search the supported portals using target role, technology or skill, location, seniority or experience limit, recency, and work-arrangement preferences when those criteria are provided.
- **FR-003**: The system MUST preserve the meaning of English, Vietnamese, and mixed-language search criteria, including technology names and explicit exclusions.
- **FR-004**: The system MUST report, for every enabled portal, whether it was searched successfully, returned no matches, was restricted, was unavailable, or failed.
- **FR-005**: The system MUST disclose when a requested search constraint could not be applied by a particular portal.
- **FR-006**: Every discovered listing MUST retain its portal name, original public reference, portal-specific identifier when available, and discovery date.
- **FR-007**: The system MUST capture available decision-critical listing details, including title, employer, location, description, skills, experience requirement, seniority, employment type, work arrangement, posting date, application deadline, and stated salary.
- **FR-008**: The system MUST distinguish an actionable verified listing from an unverified discovery lead.
- **FR-009**: The system MUST NOT assign a fit score or application recommendation to a listing that is expired, inaccessible beyond a title or snippet, or missing enough description content to evaluate the user's stated constraints.
- **FR-010**: Explicit requirements in a job description MUST take precedence over seniority labels, category tags, or search-result summaries.
- **FR-011**: The system MUST evaluate experience-limit matches using source evidence and MUST NOT infer compliance from words such as "junior" or "fresher" alone.
- **FR-012**: Missing listing facts MUST remain unknown unless another preserved source for the same vacancy supplies the fact.
- **FR-013**: The system MUST identify expired, removed, closed, or stale listings and prevent them from appearing as active recommendations.
- **FR-014**: The system MUST consolidate duplicate vacancies within and across the five named portals and existing job-discovery sources while preserving all source references.
- **FR-015**: Duplicate detection MUST consider employer, title, location, description, source identifiers, and application references sufficiently to avoid merging distinct roles that only have similar titles.
- **FR-016**: The system MUST compare discovered vacancies with existing seen-job state and application history when those records exist.
- **FR-017**: When source copies disagree, the system MUST retain the conflicting values, identify the evidence source for each, and avoid silently inventing a resolution.
- **FR-018**: Failure or restriction of one portal MUST NOT prevent available results from other portals from being returned.
- **FR-019**: The search summary MUST distinguish raw discoveries, verified active listings, duplicates, expired or removed listings, inaccessible leads, and final actionable matches.
- **FR-020**: The system MUST preserve source attribution in every shortlist, fit-screening result, and saved job record.
- **FR-021**: The system MUST respect portal access restrictions and MUST NOT bypass authentication, paid access, human-verification challenges, or other protected controls.
- **FR-022**: The system MUST avoid fabricating job requirements, employer facts, listing availability, or portal coverage.
- **FR-023**: Portal search state and personal job-search results MUST remain in the repository's designated non-public output locations.
- **FR-024**: Automated job application, account creation, credential storage, and authenticated portal actions are outside the scope of this feature.

### Key Entities

- **Portal Source**: One supported Vietnamese job portal, including its display name, availability state, publicly searchable scope, supported search constraints, and restriction status.
- **Portal Search Run**: One attempt to search enabled portals using a shared set of candidate and job criteria, including per-source outcomes and result counts.
- **Source Listing**: A vacancy as represented by one portal, including its source reference, source identifier, discovered facts, discovery date, and verification status.
- **Canonical Job Listing**: A deduplicated vacancy that consolidates one or more source listings while retaining source-specific evidence and conflicts.
- **Listing Verification**: Evidence that a listing is accessible, sufficiently detailed, and active enough for fit screening, including any reason it is not actionable.
- **Fit Evidence**: Source-backed matches and gaps between a canonical job listing and candidate preferences, including experience, skills, seniority, location, and work arrangement.
- **Source Coverage Summary**: User-facing account of which portals were attempted, their outcomes, supported or unapplied constraints, and counts at each filtering stage.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A single job-search request attempts all five enabled named portals and reports an explicit outcome for 100% of them.
- **SC-002**: In a representative validation set where each portal has at least one public matching listing, the search discovers at least one valid result from at least four of the five portals.
- **SC-003**: At least 95% of presented listings include a working source reference, title, employer or an explicit unknown-employer label, location or an explicit unknown-location label, and discovery date.
- **SC-004**: Zero inaccessible, expired, removed, or title-only listings receive a fit score or application recommendation during validation.
- **SC-005**: At least 90% of known cross-portal duplicate vacancies in a representative sample are consolidated correctly, while 100% of deliberately distinct same-title vacancies remain separate.
- **SC-006**: For experience-constrained searches, 100% of final recommended listings contain source evidence supporting the experience decision.
- **SC-007**: A failure or restriction affecting any one portal still allows the user to receive results and status information from every other available portal.
- **SC-008**: A user can understand which portals were covered, why listings were excluded, and which matches are actionable from one search report without consulting internal diagnostics.
- **SC-009**: At least 80% of the top ten actionable results in a representative review match a target role or primary skill and do not violate an explicit location, experience, or employment-type exclusion.
- **SC-010**: No reviewed output contains a fabricated job requirement, employer fact, listing status, or claim of portal coverage.

## Assumptions

- The primary user is an individual job seeker searching the Vietnamese employment market for public job listings.
- "Carrerviet" in the request refers to CareerViet, and "vietnamwork" refers to VietnamWorks.
- The five portals complement rather than replace existing LinkedIn, FreeHire, Google-discovery, and user-supplied source coverage.
- The existing candidate profile, search preferences, seen-job state, application tracker, fit-evaluation rules, and job-scrape workflow remain the authoritative shared context.
- Searches use saved profile preferences when available; missing location or work-authorization information is reported rather than inferred.
- Public source availability can vary over time, so complete coverage means every enabled portal is attempted and reported honestly, not that every portal must always return results.
- Search activity is low-volume and for personal job-seeking use.
- Publicly accessible listings may be used for discovery and verification; protected or authenticated content is outside scope unless the user supplies it directly.
- The feature produces search and screening results only; applying to jobs remains a separate user-authorized workflow.
