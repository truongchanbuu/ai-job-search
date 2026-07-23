# Feature Specification: Freelance Job Search

**Feature Branch**: `003-freelance-job-search`

**Created**: 2026-07-13

**Status**: Draft

**Input**: User description: "I want extend the current for finding freekancer jobs on Fiverr, upwork (add fiverr/upwork account) or any freelancer jobs on Internet (social media, gg search, ...)"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Find Freelance Opportunities Across Sources (Priority: P1)

A freelancer wants to search for freelance projects and gigs from Fiverr, Upwork, social media, Google-discovered public pages, and other internet sources using their existing profile and skills so they can find paid contract work without repeating manual searches.

**Why this priority**: Freelance discovery is the core value of the feature. Without relevant opportunity discovery, account readiness, proposals, and tracking do not help the user win work.

**Independent Test**: Can be fully tested by creating or loading a freelance-ready profile, running one freelance search, and confirming that results from available sources are ranked by fit to the user's skills, preferred work type, budget expectations, language, and availability.

**Acceptance Scenarios**:

1. **Given** a profile with main skills, target freelance services, preferred budget range, and availability, **When** the user starts a freelance search, **Then** the system returns relevant freelance opportunities from configured available sources.
2. **Given** a source is unavailable, requires account setup, blocks access, or returns no results, **When** the search completes, **Then** the system reports that source status and still returns results from other available sources.
3. **Given** the same opportunity appears from multiple discovery paths, **When** results are shown, **Then** the system consolidates it into one opportunity while preserving every source reference.

---

### User Story 2 - Manage Fiverr and Upwork Account Readiness (Priority: P1)

A freelancer wants to track whether Fiverr and Upwork accounts are ready for use, including profile completeness, service categories, portfolio links, verification status, and setup gaps, so platform opportunities can be evaluated realistically.

**Why this priority**: Fiverr and Upwork are explicitly requested sources, and incomplete account setup can prevent applying, bidding, or receiving client leads.

**Independent Test**: Can be fully tested by recording Fiverr and Upwork account readiness information and verifying that searches and recommendations flag any setup gaps before suggesting platform-specific action.

**Acceptance Scenarios**:

1. **Given** the user has not configured an account for a platform, **When** platform-specific opportunities are returned, **Then** the system marks them as requiring account setup before action.
2. **Given** the user records account readiness details, **When** opportunities are evaluated, **Then** the system reflects profile completeness, allowed service categories, and known platform constraints in the recommendation.
3. **Given** the user updates account readiness, **When** the same platform is searched later, **Then** recommendations use the latest readiness state.

---

### User Story 3 - Evaluate Freelance Fit and Proposal Priority (Priority: P2)

A freelancer wants to evaluate each opportunity by skill fit, client/project quality, budget, deadline, language, competition clues, and risk so they can decide which opportunities deserve a proposal or gig optimization effort.

**Why this priority**: Freelance search produces noisy results; prioritization helps the user avoid low-quality, underpaid, or risky work.

**Independent Test**: Can be fully tested by reviewing a shortlist of freelance opportunities and confirming that each has a clear recommendation, matched skills, concerns, and next action.

**Acceptance Scenarios**:

1. **Given** a freelance opportunity with project details, **When** the system evaluates fit, **Then** it identifies matched skills, missing requirements, budget fit, urgency, risk flags, and a recommendation.
2. **Given** an opportunity lacks budget, deadline, or client context, **When** it is evaluated, **Then** the system marks confidence as limited and avoids unsupported assumptions.
3. **Given** an opportunity is outside the user's constraints, **When** results are ranked, **Then** it is deprioritized or flagged as a skip.

---

### User Story 4 - Prepare Freelance Responses and Track Outcomes (Priority: P2)

A freelancer wants to draft truthful proposals, application notes, or gig improvement notes from verified profile facts and track opportunity statuses so they can manage outreach and follow-up across platforms.

**Why this priority**: Finding freelance work is only useful if the user can act on strong opportunities and remember what happened.

**Independent Test**: Can be fully tested by selecting one opportunity, generating a proposal outline or response draft, saving it to history, updating status, and retrieving the record later.

**Acceptance Scenarios**:

1. **Given** a selected opportunity and verified profile, **When** the user requests a proposal draft, **Then** the system produces a truthful response that does not invent experience, platform metrics, client facts, or project requirements.
2. **Given** the user saves an opportunity, **When** the record is viewed later, **Then** it includes source references, status, dates, proposal notes, platform, budget information when available, and outcome.
3. **Given** a saved opportunity appears again in search results, **When** results are displayed, **Then** the system identifies the existing record and avoids duplicate tracking.

---

### Edge Cases

- A platform requires login, verification, connects, seller approval, or other account setup before applying or receiving leads; the system should mark action as blocked or setup-required.
- A freelance listing is expired, hidden, invite-only, duplicated, scam-like, unpaid, speculative, or has unclear client identity; the system should flag the risk and avoid presenting it as a normal opportunity.
- Search results include full-time jobs, internships, agency hiring posts, or irrelevant service advertisements; the system should filter or flag them according to freelance intent.
- Social media posts contain personal data, unverifiable claims, referral-only offers, or private-group content; the system should use only public or user-supplied content and label reliability limitations.
- Google-discovered results are public-page discovery leads; the original posting or platform page must be verified before proposal drafting or tracking as an actionable opportunity.
- The user has no freelance services, rate preferences, portfolio links, or availability defined; the system should guide the user to fill those gaps before ranking opportunities.
- Opportunity details may be in English, Vietnamese, or mixed language; the system should preserve important source terms and support response guidance in the appropriate language.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support freelance opportunity discovery from Fiverr, Upwork, Google-discovered public pages, user-configured social media sources, and other user-supplied internet sources.
- **FR-002**: System MUST distinguish freelance opportunities from full-time employment, internships, training programs, agency hiring, and general advertisements.
- **FR-003**: System MUST allow each freelance source to be enabled, disabled, and reported independently during a search.
- **FR-004**: System MUST track Fiverr and Upwork account readiness, including setup status, profile completeness, service categories, portfolio links, platform constraints, and known blockers.
- **FR-005**: System MUST avoid storing or exposing platform passwords, private messages, payment credentials, or other sensitive account secrets in generated outputs.
- **FR-006**: Users MUST be able to run a freelance search based on the existing default profile plus freelance-specific preferences without re-entering the same skills every time.
- **FR-007**: System MUST rank freelance opportunities by relevance to main skills, target services, budget or rate fit, availability, language, work type, platform readiness, and explicit exclusions.
- **FR-008**: System MUST show enough opportunity details for screening, including title or service need, client or source when available, platform, source reference, budget or rate when available, deadline or urgency, language, skill matches, risk flags, and discovered date.
- **FR-009**: System MUST deduplicate opportunities that appear across multiple sources while preserving every source reference.
- **FR-010**: System MUST identify opportunities that require account setup, verification, bidding credits, invite access, seller approval, or user-supplied context before action.
- **FR-011**: System MUST evaluate each opportunity with matched skills, missing requirements, budget fit, timeline fit, risk flags, confidence level, and recommended next action.
- **FR-012**: System MUST treat social media and Google-discovered opportunities as discovery leads until the original post, platform page, or user-supplied content is verified.
- **FR-013**: System MUST support proposal drafts, response outlines, or gig improvement notes that use only verified profile facts and verified opportunity details.
- **FR-014**: System MUST flag unsupported claims instead of inventing freelancer experience, platform ratings, earnings, client facts, or project requirements.
- **FR-015**: System MUST allow users to save freelance opportunity records with source references, status, dates, proposal notes, platform, budget information when available, and outcome.
- **FR-016**: System MUST let users update freelance opportunity statuses such as discovered, reviewing, proposal drafted, applied, interviewing, active, won, lost, ignored, scam, or archived.
- **FR-017**: System MUST identify when a search result matches an existing freelance opportunity record.
- **FR-018**: System MUST support English, Vietnamese, or mixed-language freelance opportunities and response guidance.
- **FR-019**: System MUST handle source failures, missing details, inaccessible posts, account blockers, and empty results with clear user-facing status messages.
- **FR-020**: System MUST keep personal profile data, account readiness notes, proposal drafts, opportunity history, and source notes in designated personal-output locations that are not intended for public commits.

### Key Entities *(include if feature involves data)*

- **Freelance Profile Extension**: Freelance-specific additions to the default profile, including target services, portfolio links, rate preferences, availability, work type preferences, languages, and exclusions.
- **Freelance Source**: A configured source such as Fiverr, Upwork, Google Search, a social media page/group, public community, marketplace page, or user-supplied source, including availability and scope.
- **Platform Account Readiness**: Fiverr or Upwork setup state containing profile completeness, service categories, portfolio links, verification status, platform constraints, and blockers.
- **Freelance Opportunity**: A discovered project, gig lead, client request, or marketplace opportunity with title, client/source, platform, budget, timeline, description, skills, language, source references, and discovery metadata.
- **Freelance Fit Match**: Assessment of a freelance opportunity against the profile, including matched skills, missing requirements, budget fit, timeline fit, platform readiness, risk flags, confidence, and recommendation.
- **Proposal Material**: Draft proposal, response outline, or gig optimization note linked to a verified profile and opportunity, including evidence references and unsupported claims that need user confirmation.
- **Freelance Opportunity Record**: Saved history entry for a freelance opportunity, including status, dates, source references, proposal material references, notes, budget details when available, and outcome.
- **Source Reliability Note**: Caveat attached to a social, Google-discovered, or user-supplied source indicating whether the information is verified, partial, stale, private, or risky.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user with a complete freelance profile can run one search and receive relevant freelance opportunities from at least two available source categories within 5 minutes.
- **SC-002**: At least 80% of top-ranked opportunities in a sample review match one or more of the user's main skills or target freelance services.
- **SC-003**: 100% of Fiverr and Upwork opportunities show account readiness status or setup-required status before the user is advised to act.
- **SC-004**: Duplicate freelance opportunities across sources are consolidated with at least 90% accuracy in a representative sample of repeated listings.
- **SC-005**: Users can identify the top 5 proposal-worthy opportunities from a mixed-source result set in under 10 minutes using ranking, risk flags, and next-action recommendations.
- **SC-006**: Proposal drafts or response outlines contain no unsupported freelancer claims, platform metrics, client facts, or project details during validation against stored profile and opportunity context.
- **SC-007**: Users can save and later retrieve a complete freelance opportunity record, including status and source references, for 100% of saved opportunities.
- **SC-008**: Source-level failures or blockers do not prevent results from other available freelance sources from being shown.

## Assumptions

- This feature extends the existing single-user job-search workspace rather than replacing the current full-time job workflow.
- "Freelancer jobs" means freelance projects, contract gigs, client requests, marketplace opportunities, and paid service leads, not regular full-time employment.
- Fiverr and Upwork account support means tracking account readiness and setup blockers; private account credentials, payment credentials, and private messages are not stored by default.
- Social media support covers public or user-supplied posts from pages, groups, communities, and networks the user is allowed to access.
- Google Search remains supplemental public-page discovery and is not a Google Jobs-style UI or general result scraper.
- The existing default profile remains the source of verified candidate facts, with freelance-specific preferences added for services, rates, availability, and portfolio context.
- Proposal and response drafts must follow existing repository safety rules: no invented candidate facts, client facts, project requirements, platform ratings, earnings, or learning resources.
