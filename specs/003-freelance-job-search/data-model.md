# Data Model: Freelance Job Search

## FreelanceProfileExtension

Freelance-specific additions to the existing default candidate profile.

**Fields**:

- `profile_id`: Link to the default profile.
- `target_services`: Services the user wants to sell, such as mobile app development, Flutter fixes, API integration, or automation.
- `portfolio_links`: Public examples, case studies, repositories, screenshots, or documents the user can safely share.
- `rate_preferences`: Hourly, fixed-price, minimum budget, preferred currency, and negotiation notes.
- `availability`: Weekly hours, start date, timezone, and response-time expectations.
- `work_type_preferences`: Fixed-price, hourly, retainer, short gig, long contract, maintenance, consultation, or invite-only work.
- `languages`: Preferred proposal and client communication languages.
- `platform_profiles`: User-visible Fiverr, Upwork, portfolio, or social profile references.
- `exclusions`: Work types, industries, budgets, platforms, or client conditions to avoid.
- `updated_at`: Last update timestamp.

**Validation Rules**:

- `target_services` must contain at least one service before profile-driven freelance search.
- Rate and availability preferences should be present before budget/timeline scoring is treated as high confidence.
- Portfolio and platform profile links must be user-provided or verified before proposal material references them.

## FreelanceSource

A configured source for discovering freelance opportunities or public context.

**Fields**:

- `source_id`: Stable source key.
- `source_type`: One of `fiverr`, `upwork`, `google`, `social`, `community`, `marketplace`, or `user_supplied`.
- `display_name`: User-facing source name.
- `enabled`: Whether the source participates in searches.
- `scope`: Search scope such as platform, query, domain list, page URL, group URL, community URL, or user-supplied folder.
- `access_mode`: `public`, `account_required`, `manual`, `user_supplied`, or `blocked`.
- `status`: `available`, `needs_setup`, `unavailable`, `blocked`, or `skipped`.
- `last_checked_at`: Last source check timestamp.
- `notes`: Source limitations or setup notes.

**Validation Rules**:

- Disabled sources must not block search completion.
- Sources that require login, verification, credits, or user-supplied content must return a source-level status.
- Private social or platform content must be skipped unless the user supplies it.

## PlatformAccountReadiness

Readiness state for Fiverr, Upwork, or future freelance platforms.

**Fields**:

- `platform`: Platform key such as `fiverr` or `upwork`.
- `account_status`: `not_configured`, `draft`, `ready`, `limited`, `blocked`, or `unknown`.
- `profile_completeness`: Checklist of profile fields, portfolio links, service categories, description, skills, and verification.
- `service_categories`: Services or categories enabled on the platform.
- `portfolio_refs`: Platform-specific portfolio or gig references.
- `verification_status`: `not_started`, `pending`, `verified`, `failed`, or `unknown`.
- `action_currency`: Platform-specific action resource such as Upwork Connects, seller approval, or setup blocker when known.
- `known_blockers`: Missing setup, verification, billing/payment, connects, seller approval, or policy limitations.
- `last_updated_at`: Last update timestamp.

**Validation Rules**:

- No platform password, private message, payment credential, or secret token may be stored.
- A platform-specific opportunity cannot receive an `apply_now` or `optimize_now` recommendation unless account readiness is known enough to support that action.
- Unknown readiness must be surfaced as a user-facing caveat.

## FreelanceOpportunity

A discovered freelance project, gig lead, client request, or marketplace opportunity.

**Fields**:

- `opportunity_id`: Workspace-generated stable identifier.
- `canonical_key`: Deduplication key derived from platform/source, title, client/source, budget, location/remote clue, and URL when available.
- `title`: Opportunity title, service need, or post headline.
- `client_or_source`: Client name, company, buyer handle, page/group, or source label when available.
- `platform`: Source platform or channel.
- `url_or_identifier`: URL or source-specific identifier.
- `budget`: Fixed price, hourly rate, range, or unknown.
- `timeline`: Deadline, duration, urgency, or unknown.
- `description`: Verified opportunity text when available.
- `skills`: Skills, tools, responsibilities, or service terms extracted from verified content.
- `language`: `en`, `vi`, or mixed/unknown.
- `source_refs`: One or more source references.
- `risk_flags`: Spam, scam, unpaid, low-budget, vague scope, private-only, expired, duplicate, or policy risk flags.
- `discovered_at`: Discovery timestamp.
- `posted_at`: Posted date when available.
- `source_status`: Per-source retrieval status.

**Validation Rules**:

- An opportunity must preserve at least one source reference.
- Discovery leads must not be treated as actionable until original content is verified or user-supplied.
- Opportunities without enough details may be shown but must carry limited confidence.

## FreelanceSourceReference

Reference to where a freelance opportunity or related post was found.

**Fields**:

- `source_id`: Linked FreelanceSource key.
- `url_or_identifier`: URL, platform ID, post link, or user-supplied file reference.
- `retrieved_at`: Retrieval timestamp.
- `raw_title`: Source-provided title when available.
- `raw_client_or_source`: Source-provided client/source label when available.
- `raw_budget`: Source-provided budget/rate when available.
- `status`: `retrieved`, `partial`, `discovery_lead`, `unavailable`, `blocked`, or `expired`.
- `reliability_note_id`: Optional linked SourceReliabilityNote.

## FreelanceFitMatch

Assessment of a freelance opportunity against the profile and platform readiness.

**Fields**:

- `fit_id`: Stable identifier.
- `opportunity_id`: Linked FreelanceOpportunity.
- `profile_id`: Linked default profile.
- `matched_skills`: Skills present in both profile and opportunity.
- `missing_requirements`: Important requirements not supported by profile evidence.
- `budget_fit`: `strong`, `acceptable`, `low`, `unknown`, or `skip`.
- `timeline_fit`: `available`, `tight`, `unavailable`, or `unknown`.
- `platform_readiness`: `ready`, `setup_required`, `limited`, `blocked`, or `unknown`.
- `risk_flags`: Risks inherited or added during evaluation.
- `confidence`: `high`, `medium`, or `limited`.
- `recommendation`: `proposal_worthy`, `review`, `setup_first`, `watch`, or `skip`.
- `next_action`: Recommended user action.
- `created_at`: Assessment timestamp.

**Validation Rules**:

- A fit match cannot claim a skill match that is not in the verified profile.
- Unknown account readiness or unverified source content must reduce confidence.
- High-risk or blocked opportunities cannot be recommended as proposal-worthy.

## ProposalMaterial

Draft proposal, response outline, or gig optimization note tied to one opportunity.

**Fields**:

- `material_id`: Stable identifier.
- `opportunity_id`: Linked FreelanceOpportunity.
- `profile_id`: Linked default profile.
- `material_type`: `proposal`, `response_outline`, `gig_note`, or `follow_up`.
- `language`: `en`, `vi`, or mixed.
- `file_path`: Generated file path.
- `evidence_refs`: Verified profile facts and verified opportunity details used.
- `unsupported_claims`: Claims requiring user confirmation.
- `created_at`: Creation timestamp.
- `verification_status`: `draft`, `needs_confirmation`, `verified`, or `ready`.

**Validation Rules**:

- Proposal material must not include unsupported freelancer claims, client facts, project details, platform ratings, earnings, or availability.
- No material may be marked `ready` while unsupported claims remain.
- Generated material must preserve source attribution for opportunity facts.

## FreelanceOpportunityRecord

Persistent history entry for one freelance opportunity.

**Fields**:

- `record_id`: Stable identifier.
- `opportunity_id`: Linked FreelanceOpportunity.
- `status`: Current status.
- `saved_at`: Date saved to history.
- `updated_at`: Last update date.
- `actioned_at`: Proposal/application/action date when applicable.
- `source_refs`: Preserved source references.
- `material_refs`: Proposal, response, or gig-note references.
- `platform`: Fiverr, Upwork, Google, social, community, or user-supplied.
- `budget`: Budget or rate information when available.
- `notes`: User notes.
- `outcome`: Outcome summary when known.

**States**:

- `discovered`
- `reviewing`
- `proposal_drafted`
- `applied`
- `interviewing`
- `active`
- `won`
- `lost`
- `ignored`
- `scam`
- `archived`

**Validation Rules**:

- A saved record must include opportunity identity, source reference, status, and saved date.
- Status updates must not rewrite unrelated records.
- Search results matching saved records must surface the existing status.

## SourceReliabilityNote

Caveat attached to social, Google-discovered, or user-supplied content.

**Fields**:

- `note_id`: Stable identifier.
- `source_id`: Linked FreelanceSource key.
- `reliability`: `verified`, `partial`, `discovery_lead`, `stale`, `private`, `risky`, or `unknown`.
- `reason`: Short explanation of the limitation.
- `checked_at`: Last verification timestamp.
- `recommended_handling`: `verify_original`, `request_user_context`, `skip`, or `safe_to_use`.

**Validation Rules**:

- Private or restricted content must not be treated as verified unless the user supplies it.
- Risky or stale content must be surfaced before ranking or proposal drafting.
