# Contract: Workflow Output Files

This contract defines the expected workspace files created or updated by the job search suite.

## Default Profile

**Location**: A gitignored personal-profile path under `documents/`.

**Required Content**:

- Main skills.
- Target roles.
- Preferred locations or remote preference.
- Languages.
- Verified experience highlights.
- Source references for profile facts.

**Rules**:

- Profile facts must be user-provided or source-backed.
- The default profile must be loaded before profile-driven search.

## Search Session Output

**Location**: Gitignored scrape/search output under `job_scraper/` or `documents/applications/` when tied to a specific application.

**Required Content**:

- Search timestamp.
- Enabled sources and per-source statuses.
- Normalized job listings.
- Deduplication notes.
- Fit matches and recommendations.
- Existing application matches.

**Rules**:

- Results must preserve source references.
- Failed or skipped sources must be listed.

## CV Materials

**Location**:

- English CV drafts under `cv/main_*.tex`.
- Vietnamese CV drafts under `cv/main_*.tex` with a language-distinguishing filename or archive path.
- Compiled PDFs remain gitignored.

**Required Content**:

- Candidate facts supported by the default profile.
- Job-specific emphasis.
- Language marker or filename convention.
- Verification status.

**Rules**:

- CVs must compile and be inspected before final delivery.
- Unsupported claims must be removed or explicitly confirmed by the user.

## ATS Keyword Review

**Location**: Application archive under `documents/applications/<company>_<role>/`.

**Required Content**:

- Job reference.
- CV material reference.
- Matched terms.
- Missing terms.
- Weak terms.
- Unsupported terms.
- Evidence-aware revision suggestions.

**Rules**:

- Review output must not encourage adding unsupported claims.
- Suggestions should preserve natural CV language in English or Vietnamese.

## Application History

**Location**: `job_search_tracker.csv` plus per-application archive under `documents/applications/<company>_<role>/`.

**Required Content**:

- Job identity.
- Source references.
- Current status.
- Important dates.
- Material references.
- Notes and outcome when available.

**Rules**:

- Updates must preserve unrelated rows.
- Duplicate applications must be detected from job identity and source references.

## Interview Prep Packet

**Location**: Application archive under `documents/applications/<company>_<role>/`.

**Required Content**:

- Job and application references.
- Question groups.
- Candidate experience talking points.
- Related-post context with source labels.
- Missing context and caveats.

**Rules**:

- Related posts must not be presented as verified company facts.
- Questions should be available in English or Vietnamese according to user preference and source language.
