# Quickstart: Job Search Suite Validation

This guide describes validation scenarios for the planned feature. It is intentionally high-level; implementation tasks will define exact commands and file names.

## Prerequisites

- Repository setup completed from `README.md` and `SETUP.md`.
- Existing validation commands available:
  - `python tools/lint_skills.py`
  - `python tools/security_guards.py`
  - `python -m pytest tests`
- Portal CLI dependencies installed where a source CLI has tests or typechecking.
- A default candidate profile with at least one main skill and one target role.
- Fixture or live-accessible job postings for at least three supported source categories.

## Scenario 1: Profile-Driven Multi-Source Search

1. Load the default profile.
2. Enable at least three supported sources.
3. Run the profile-driven search workflow.
4. Confirm output includes:
   - Per-source status.
   - Normalized job listings.
   - Deduplicated jobs with source references.
   - Ranking or fit recommendation tied to profile skills.
   - Existing application status when a job is already tracked.

**Expected Outcome**: A ranked shortlist is produced within 5 minutes of user effort, unavailable sources are reported clearly, and no job is presented without source attribution.

## Scenario 2: Bilingual CV Drafts

1. Select one high-fit job from the shortlist.
2. Generate English and Vietnamese CV drafts.
3. Confirm both drafts use the same verified profile facts.
4. Compile and inspect generated PDFs before final delivery.

**Expected Outcome**: English and Vietnamese CV drafts are generated without unsupported candidate claims.

## Scenario 3: ATS Keyword Review

1. Use the selected job description and one generated CV draft.
2. Run ATS keyword review.
3. Confirm output includes matched, missing, weak, overused, and unsupported terms.
4. Confirm each recommendation is tied to profile evidence or marked unsupported.

**Expected Outcome**: The user receives actionable and truthful keyword guidance in under 2 minutes of user effort.

## Scenario 4: Save and Update Application History

1. Save the selected job as an application record.
2. Attach generated material references.
3. Update status from `interested` to `preparing` to `applied`.
4. Re-run search and confirm the same job shows existing application status.

**Expected Outcome**: Application history preserves job identity, source references, material references, dates, and status without duplicating the same job.

## Scenario 5: Interview Prep From JD and Related Posts

1. Select a tracked application with a job description.
2. Provide related public-post context or use accessible configured sources.
3. Generate interview preparation output in English or Vietnamese.
4. Confirm output includes at least 10 questions when enough context exists.
5. Confirm every question is tied to a job requirement, candidate experience, or clearly labeled related-post context.

**Expected Outcome**: The user receives a preparation packet that is useful for the role and does not present public-post claims as verified company facts.

## Regression Validation

After implementation, run:

```powershell
python tools\lint_skills.py
python tools\security_guards.py
python -m pytest tests
```

For each new or updated portal CLI, also run its typecheck and test command from the CLI directory.
