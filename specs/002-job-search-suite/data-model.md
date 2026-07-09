# Data Model: Job Search Suite

## CandidateProfile

Represents the single default job-seeker profile used to drive search, CV tailoring, ATS checks, and interview preparation.

**Fields**:

- `profile_id`: Stable identifier for the default profile.
- `display_name`: Candidate display name.
- `main_skills`: Required list of primary skills.
- `secondary_skills`: Optional supporting skills.
- `target_roles`: Desired titles or role families.
- `preferred_locations`: Locations or remote preferences.
- `languages`: Supported application languages, initially English and Vietnamese.
- `experience_highlights`: Verified experience facts available for CV/interview use.
- `education_highlights`: Verified education facts.
- `constraints`: Application constraints such as seniority, salary, work mode, relocation, or excluded roles.
- `source_references`: Documents or profile files supporting stored facts.
- `updated_at`: Last profile update date.

**Validation Rules**:

- `main_skills` must contain at least one skill before profile-driven search.
- Generated candidate claims must trace to `experience_highlights`, `education_highlights`, or source references.
- Only one profile is active by default in v1.

## JobSource

Represents a configured source for job discovery or public-post context.

**Fields**:

- `source_id`: Stable source key.
- `source_type`: One of `linkedin`, `vietnamworks`, `topcv`, `itviec`, `vieclam24h`, `google`, `facebook`.
- `display_name`: User-facing source name.
- `enabled`: Whether the source participates in searches.
- `scope`: Search scope such as portal, domain list, page URL, group URL, or user-supplied content folder.
- `access_mode`: `public`, `credentialed`, `user_supplied`, or `manual`.
- `status`: `available`, `unavailable`, `blocked`, `needs_setup`, or `skipped`.
- `last_checked_at`: Last source check timestamp.
- `notes`: Source-specific limitations or setup notes.

**Validation Rules**:

- Disabled sources must not block search completion.
- Unavailable sources must return a source-level status rather than failing the whole workflow.
- Private/restricted Facebook content must be user-supplied or skipped.

## JobListing

Normalized job record from one or more sources.

**Fields**:

- `job_id`: Workspace-generated stable identifier.
- `canonical_key`: Deduplication key derived from employer, title, location, and source URL when available.
- `title`: Job title.
- `employer`: Employer name when available.
- `location`: Location or remote status.
- `language`: Inferred or declared posting language.
- `seniority_clues`: Seniority signals found in title or description.
- `description`: Job description text when available.
- `requirements`: Extracted requirements and qualifications.
- `skills`: Extracted skills and tools.
- `source_refs`: One or more source references.
- `discovered_at`: Discovery timestamp.
- `posted_at`: Posted date when available.
- `source_status`: Per-source retrieval status.

**Validation Rules**:

- A listing must preserve at least one source reference.
- Jobs without descriptions may be shown but must be marked as limited-detail.
- Duplicate listings should merge source references rather than overwrite prior records.

## SourceReference

Reference to where a job listing or public post was found.

**Fields**:

- `source_id`: Linked JobSource key.
- `url_or_identifier`: URL or source-specific identifier.
- `retrieved_at`: Retrieval timestamp.
- `raw_title`: Source-provided title when available.
- `raw_location`: Source-provided location when available.
- `raw_employer`: Source-provided employer when available.
- `status`: `retrieved`, `partial`, `unavailable`, `blocked`, or `expired`.

## FitMatch

Assessment of a job listing against the default profile.

**Fields**:

- `fit_id`: Stable identifier.
- `job_id`: Linked JobListing.
- `profile_id`: Linked CandidateProfile.
- `matched_skills`: Skills present in both profile and job.
- `missing_requirements`: Important job requirements not found in profile.
- `concerns`: Seniority, location, language, or constraint mismatches.
- `recommendation`: `strong`, `possible`, `weak`, or `skip`.
- `rationale`: Human-readable fit explanation.
- `created_at`: Assessment timestamp.

**Validation Rules**:

- A fit match cannot claim a skill match that is not in the profile.
- Jobs with inaccessible descriptions should receive a limited-confidence recommendation.

## CVMaterial

Generated CV draft for a specific job and language.

**Fields**:

- `material_id`: Stable identifier.
- `job_id`: Linked JobListing.
- `profile_id`: Linked CandidateProfile.
- `language`: `en` or `vi`.
- `file_path`: Generated file path.
- `template_id`: Template used.
- `evidence_refs`: Profile facts used in the material.
- `unsupported_claims`: Claims requiring user confirmation.
- `created_at`: Generation timestamp.
- `verification_status`: `draft`, `needs_confirmation`, `compiled`, `inspected`, or `ready`.

**Validation Rules**:

- No unsupported claim may be marked ready.
- English and Vietnamese drafts for the same job must use the same verified profile facts.

## ATSKeywordReview

Keyword comparison between a job description and CV material.

**Fields**:

- `review_id`: Stable identifier.
- `job_id`: Linked JobListing.
- `material_id`: Linked CVMaterial.
- `matched_terms`: Important job terms present in the CV.
- `missing_terms`: Important job terms absent from the CV.
- `weak_terms`: Terms present but weakly represented.
- `overused_terms`: Terms repeated excessively or generically.
- `unsupported_terms`: Terms that should not be added without profile evidence.
- `recommendations`: Evidence-aware revision suggestions.
- `created_at`: Review timestamp.

**Validation Rules**:

- Recommendations must distinguish truthful supported edits from unsupported additions.
- Generic filler terms should not dominate the review.

## ApplicationRecord

Persistent application history entry.

**Fields**:

- `application_id`: Stable identifier.
- `job_id`: Linked JobListing.
- `status`: Current application status.
- `saved_at`: Date saved to history.
- `applied_at`: Application date when applicable.
- `updated_at`: Last update date.
- `source_refs`: Preserved job source references.
- `material_refs`: Generated CV, cover-letter, ATS review, and prep files.
- `notes`: User notes.
- `outcome`: Outcome summary when known.

**States**:

- `interested`
- `preparing`
- `applied`
- `interviewing`
- `offer`
- `rejected`
- `withdrawn`
- `archived`

**Validation Rules**:

- A saved application must include job identity, source reference, status, and saved date.
- Search results matching an existing application must display the existing status.
- Status updates must not rewrite unrelated application records.

## RelatedPost

Public or user-supplied post used for interview-preparation context.

**Fields**:

- `post_id`: Stable identifier.
- `source_id`: Source reference.
- `url_or_identifier`: URL or user-supplied identifier.
- `title`: Post title when available.
- `author_or_context`: Page, group, or context label when available.
- `posted_at`: Post date when available.
- `content_excerpt`: Relevant excerpt or summary.
- `reliability_note`: Caveat about source quality.
- `retrieved_at`: Retrieval timestamp.

**Validation Rules**:

- Related posts must not be treated as verified company facts.
- Private or restricted post content must be user-supplied by the user.

## InterviewPrepPacket

Interview preparation output for a tracked job.

**Fields**:

- `packet_id`: Stable identifier.
- `application_id`: Linked ApplicationRecord.
- `job_id`: Linked JobListing.
- `language`: `en`, `vi`, or mixed.
- `question_groups`: Technical, behavioral, role-specific, company-context, and gap-handling groups.
- `experience_links`: Candidate experience facts tied to questions.
- `related_post_refs`: RelatedPost references used as context.
- `missing_context`: Needed job, company, or post context that was unavailable.
- `created_at`: Generation timestamp.

**Validation Rules**:

- Each question must tie to a job requirement, candidate experience, or labeled related-post context.
- The packet must include at least 10 questions for a selected tracked job when enough job context exists.
