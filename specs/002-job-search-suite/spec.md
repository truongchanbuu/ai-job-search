# Feature Specification: Job Search Suite

**Feature Branch**: `002-job-search-suite`

**Created**: 2026-07-09

**Status**: Draft

**Input**: User description: "I wanna have some more features: support LinkedIn, VietnamWorks, TopCV, ITviec, Vieclam24h, Google Search, Facebook pages/groups; English and Vietnamese CVs; one default profile and job search related to main skills; ATS keyword checker; save application history; prepare interview questions from JD and experience from related posts."

## Clarifications

### Session 2026-07-13

- Q: Should Google Search behave like the Google Jobs UI shown in the screenshot or stay as supplemental discovery for public job posting pages? → A: Google Search remains supplemental public-page discovery; main feature work should not focus on UI.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Find Relevant Jobs From Multiple Sources (Priority: P1)

A job seeker wants to search across major job sources and social channels from one workflow, using a default profile so results are related to their main skills instead of requiring repeated manual queries.

**Why this priority**: Job discovery is the entry point for the rest of the workflow; without relevant jobs, CV tailoring, ATS checks, application history, and interview preparation cannot deliver value.

**Independent Test**: Can be fully tested by creating or loading a default profile, running one search, and confirming that matched jobs from supported sources are ranked by relevance to the profile's main skills.

**Acceptance Scenarios**:

1. **Given** a default profile with main skills and target role preferences, **When** the user starts a job search, **Then** the system returns job matches related to those skills across configured supported sources.
2. **Given** the same job appears from multiple sources, **When** search results are shown, **Then** the system presents one deduplicated job record with source references preserved.
3. **Given** a supported source is unavailable or returns no results, **When** the search completes, **Then** the user receives available results from other sources and a clear source-level status.

---

### User Story 2 - Generate English and Vietnamese CV Materials (Priority: P1)

A job seeker wants application materials in both English and Vietnamese so they can apply appropriately to local, international, and bilingual job postings.

**Why this priority**: Bilingual CV support is core to applying in the Vietnamese market and to international roles that require English materials.

**Independent Test**: Can be fully tested by selecting one matched job and producing English and Vietnamese CV drafts that reflect the same verified profile facts and job-specific emphasis.

**Acceptance Scenarios**:

1. **Given** a job posting and a verified profile, **When** the user requests tailored CV materials, **Then** the system produces English and Vietnamese CV drafts without adding unsupported candidate facts.
2. **Given** a job posting language can be inferred, **When** CV materials are prepared, **Then** the system recommends the most appropriate language while keeping both language versions available.
3. **Given** a profile fact is missing for a requested claim, **When** CV content is generated, **Then** the system omits the claim or flags it for user confirmation instead of inventing it.

---

### User Story 3 - Check ATS Keywords Before Applying (Priority: P2)

A job seeker wants to compare their tailored CV against a job description to identify missing or weakly represented keywords before submitting an application.

**Why this priority**: ATS screening can block otherwise strong applications; keyword feedback improves application quality before submission.

**Independent Test**: Can be fully tested by comparing one CV draft to one job description and reviewing a keyword match score, missing keyword list, and concrete revision suggestions.

**Acceptance Scenarios**:

1. **Given** a job description and CV draft, **When** the user runs the ATS keyword checker, **Then** the system reports matched, missing, and overused keywords relevant to the job.
2. **Given** the checker finds important missing keywords, **When** recommendations are shown, **Then** each recommendation identifies where the keyword could be truthfully reflected or states that no supported profile evidence exists.
3. **Given** the job description contains generic or misleading terms, **When** the checker analyzes keywords, **Then** it prioritizes role-specific skills, tools, responsibilities, and qualifications over filler language.

---

### User Story 4 - Track Application History (Priority: P2)

A job seeker wants a persistent history of jobs, application materials, statuses, dates, and outcomes so they can manage follow-ups and avoid duplicate applications.

**Why this priority**: Application tracking turns one-off job search into an ongoing workflow and supports interview prep and outcome learning.

**Independent Test**: Can be fully tested by saving an application from a matched job, updating its status, and retrieving the complete history later.

**Acceptance Scenarios**:

1. **Given** a matched job and generated materials, **When** the user saves an application, **Then** the history records the job, source, date, material references, status, and relevant notes.
2. **Given** a previously saved job appears again in search results, **When** results are displayed, **Then** the system indicates the existing application status and prevents accidental duplicate tracking.
3. **Given** an application outcome changes, **When** the user updates the record, **Then** the application history keeps the current status and preserves enough context for later review.

---

### User Story 5 - Prepare Interview Questions From Job and Related Experience (Priority: P3)

A job seeker wants likely interview questions and answer prompts derived from the job description, their own experience, and relevant public discussion posts so they can prepare for company- and role-specific interviews.

**Why this priority**: Interview preparation is valuable after applications are submitted, but it depends on reliable job and profile context gathered earlier.

**Independent Test**: Can be fully tested by selecting a tracked job and generating interview questions grouped by technical skills, behavioral themes, role responsibilities, and profile-specific talking points.

**Acceptance Scenarios**:

1. **Given** a tracked job with a job description and profile history, **When** the user requests interview preparation, **Then** the system generates questions tied to job requirements and candidate experience.
2. **Given** relevant public posts are available, **When** interview prep is generated, **Then** the system includes themes from those posts with source context and avoids presenting unverified claims as facts.
3. **Given** limited job or post context is available, **When** interview prep is generated, **Then** the system still provides clearly labeled general role questions and identifies missing context.

---

### Edge Cases

- A source requires login, blocks access, changes content availability, or cannot be searched; the system should mark the source unavailable and continue with other configured sources.
- A job has incomplete or stale information; the system should display available details, source date where known, and avoid unsupported inferences.
- Search results include spam, duplicate reposts, unpaid internships, unrelated locations, or mismatched seniority; the system should filter or flag these according to profile preferences.
- The default profile has no main skills yet; the system should guide the user to add skills before running profile-driven search.
- A job description is in Vietnamese, English, or mixed language; the system should handle both languages and preserve important terms in their original language where useful.
- Public posts contain personal data, rumors, or unverifiable claims; the system should use them only as preparation context and not as factual application claims.
- Application history contains sensitive personal materials; the system should keep generated outputs in the repository's designated personal-output locations.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support job discovery from LinkedIn, VietnamWorks, TopCV, ITviec, Vieclam24h, Google Search, and user-configured Facebook pages or groups.
- **FR-002**: System MUST allow each supported source to be enabled, disabled, and reported independently during a search.
- **FR-003**: System MUST maintain one default candidate profile containing main skills, target roles, preferred locations, language preferences, experience highlights, and application constraints.
- **FR-004**: Users MUST be able to run a job search based on the default profile without re-entering the same skills and preferences for every search.
- **FR-005**: System MUST rank job results by relevance to the default profile's main skills, target roles, preferred locations, and explicit exclusions.
- **FR-006**: System MUST deduplicate jobs that appear across multiple sources while preserving every source where the job was found.
- **FR-007**: System MUST show enough job details for fit screening, including title, employer when available, location, source, posting link or reference, language, seniority clues, relevant skill matches, and discovered date.
- **FR-008**: System MUST support tailored CV outputs in English and Vietnamese from the same verified candidate profile.
- **FR-009**: System MUST prevent invented candidate facts in generated CV content and must flag unsupported claims for user confirmation.
- **FR-010**: System MUST compare a CV draft against a job description and produce an ATS keyword review with matched terms, missing high-value terms, weakly represented terms, and overused terms.
- **FR-011**: System MUST explain ATS keyword recommendations in terms of truthful profile evidence and job relevance.
- **FR-012**: System MUST allow users to save a job application record with job details, source references, generated material references, application status, dates, notes, and outcome.
- **FR-013**: System MUST let users update application history statuses such as interested, preparing, applied, interviewing, offer, rejected, withdrawn, or archived.
- **FR-014**: System MUST identify when a search result matches an existing application history record.
- **FR-015**: System MUST generate interview preparation questions from the selected job description and the candidate's verified experience.
- **FR-016**: System MUST include relevant themes from related public posts when available, while labeling them as preparation context rather than verified company facts.
- **FR-017**: System MUST support English and Vietnamese interview preparation content when source material or user preference calls for either language.
- **FR-018**: System MUST preserve source attribution for job listings and public posts used in fit evaluation, ATS checks, and interview preparation.
- **FR-019**: System MUST handle source failures, missing job descriptions, inaccessible posts, and empty results with clear user-facing status messages.
- **FR-020**: System MUST keep personal profile data, generated CVs, application history, and interview notes in designated personal-output locations that are not intended for public commits.

### Key Entities *(include if feature involves data)*

- **Candidate Profile**: Default job-seeker profile containing main skills, target roles, locations, languages, verified experience highlights, exclusions, and application preferences.
- **Job Source**: A configured source such as LinkedIn, VietnamWorks, TopCV, ITviec, Vieclam24h, Google Search, or a Facebook page/group, including availability status and search scope.
- **Job Listing**: A discovered role with title, employer, location, source references, link or identifier, description when available, skills, language, seniority clues, and discovery metadata.
- **Fit Match**: A profile-to-job assessment containing matched skills, missing requirements, relevance score or category, concerns, and recommendation.
- **CV Material**: English or Vietnamese CV draft linked to a profile version and job listing, containing generated content and evidence references.
- **ATS Keyword Review**: Analysis comparing a job description and CV material, including matched terms, missing terms, weak terms, overused terms, and revision guidance.
- **Application Record**: Saved history entry for a job application, including status, dates, source references, material references, notes, and outcome.
- **Interview Prep Packet**: Questions and answer prompts derived from the job description, candidate experience, and related public-post context with source attribution.
- **Related Post**: Public discussion or page/group content used only for interview preparation context, with source reference, date when available, and reliability notes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user with a complete default profile can run one search and receive relevant jobs from at least three available supported source categories within 5 minutes.
- **SC-002**: At least 80% of top-ranked results in a sample review match one or more of the candidate's main skills or target role terms.
- **SC-003**: Duplicate job postings across sources are consolidated with at least 90% accuracy in a representative sample of repeated listings.
- **SC-004**: Users can generate both English and Vietnamese CV drafts for a selected job in one workflow without re-entering profile facts.
- **SC-005**: ATS keyword review identifies matched and missing high-value job terms for a selected CV and job description in under 2 minutes of user effort.
- **SC-006**: Users can save and later retrieve a complete application history record, including status and material references, for 100% of saved applications.
- **SC-007**: Interview preparation output includes at least 10 relevant questions for a selected tracked job, with each question tied to job requirements, candidate experience, or clearly labeled related-post context.
- **SC-008**: No generated CV, application record, or interview prep output includes unsupported candidate claims during validation against the stored profile.

## Assumptions

- The primary user is one job seeker using this workspace for their own applications.
- "Facebook support" means user-configured pages or groups where the user is allowed to access public or available posts; private or restricted content is outside scope unless the user supplies the content.
- Google Search is used as a discovery source for publicly available job pages and related interview-preparation context, not as a replacement for dedicated source records or a Google Jobs-style UI.
- The default profile is required before profile-driven search, ATS checking, CV tailoring, and interview preparation can produce high-quality results.
- English and Vietnamese are the first supported CV and interview-prep languages for this feature.
- Application history should extend the existing job-search tracking concept rather than replace prior saved records.
- Public posts used for interview preparation may be noisy; the system should treat them as context signals, not verified facts.
- The feature must follow existing repository safety rules: no invented candidate facts, company facts, job requirements, or learning resources.
