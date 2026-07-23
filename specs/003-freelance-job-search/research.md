# Research: Freelance Job Search

## Decision: Extend the current skill-based workspace instead of creating a new app

**Rationale**: The existing job-search suite is already built around Codex skills, source-specific search helpers, shared workflow guidance, file-backed personal outputs, and tracker-style records. Freelance search is a related workflow: discover opportunities, evaluate fit, prepare truthful response material, and track outcomes. Keeping this in the same workspace avoids duplicating profile, factuality, history, and source-attribution rules.

**Alternatives considered**:

- New web application: rejected because the current user workflow is agent/CLI/document driven.
- Separate freelance repository: rejected because the feature depends on the same default profile and application history concepts.
- Prompt-only workflow: rejected because deduplication, readiness checks, ranking, and history require stable records.

## Decision: Use source-specific adapters plus a cross-source freelance workflow

**Rationale**: Fiverr, Upwork, Google-discovered pages, social posts, and user-supplied opportunities have different access patterns and reliability. A shared normalized freelance opportunity record lets ranking, deduplication, proposal drafting, and history operate consistently while each source reports limitations independently.

**Alternatives considered**:

- One monolithic freelance scraper: rejected because platform setup, login, access limits, and source fields vary widely.
- Treat all results as unstructured notes: rejected because source attribution, deduplication, ranking, and existing-record matching need stable fields.

## Decision: Treat Upwork as an account-sensitive source with Connects and readiness checks

**Rationale**: Upwork proposals can require Connects, and Upwork documents that Connects are used to submit proposals, boost proposals, and show availability; required Connects vary by job and can change while a job is posted. The system therefore must not simply recommend "apply" without checking user-recorded account readiness, Connects/bidding constraints when known, verification status, and profile completeness.

**Source**:

- Upwork Help, "Understanding and using Connects": https://support.upwork.com/hc/en-us/articles/211062898-Understanding-and-using-Connects

**Alternatives considered**:

- Treat Upwork like a generic public job board: rejected because actionability depends on platform account state and proposal cost.
- Store Upwork credentials for automation: rejected because the spec excludes platform passwords, private messages, and payment credentials from generated outputs.
- Require complete Upwork automation in v1: rejected because source availability and account permissions can vary; manual/user-supplied fallback is safer.

## Decision: Treat Fiverr as seller/gig readiness plus public/manual discovery first

**Rationale**: Fiverr is seller-offer oriented: freelancers typically prepare seller profiles and gigs so buyers can purchase services or contact them. Planning should prioritize account readiness, gig/service alignment, portfolio quality, and discovery of relevant buyer requests or public opportunities when available. Because no supported public access path is assumed for private account data, Fiverr automation should degrade to readiness tracking and user-supplied/manual discovery unless a compliant source path is confirmed during implementation.

**Source**:

- Fiverr public marketplace context: https://www.fiverr.com/

**Alternatives considered**:

- Scrape private seller dashboards or messages: rejected because private/restricted content must remain user-controlled.
- Model Fiverr as only bid-based job applications: rejected because the platform is commonly seller/gig driven rather than only job-post driven.
- Exclude Fiverr from v1: rejected because the user explicitly requested Fiverr account support.

## Decision: Keep Google and social media as verification-required discovery sources

**Rationale**: The current Google search behavior already treats Google as supplemental public-page discovery rather than a Google Jobs UI. Social posts and Google results may be stale, partial, private, or noisy. For freelance work, these sources should find candidate URLs or posts, then require original-source verification before ranking as actionable, drafting proposals, or saving as a serious opportunity.

**Alternatives considered**:

- Treat Google/social hits as final opportunities: rejected because source reliability and context are too variable.
- Scrape private social groups by default: rejected because access and privacy expectations differ by group.
- Ignore social and Google sources: rejected because the user explicitly requested broader internet discovery.

## Decision: Add freelance-specific profile preferences instead of creating a second profile

**Rationale**: The user already has one default candidate profile for skills and verified experience. Freelance search needs extra fields such as target services, rate preferences, availability, portfolio links, work type, platform profile URLs, and exclusions. Adding these as an extension avoids conflicting candidate facts while allowing freelance-specific ranking.

**Alternatives considered**:

- Separate freelance profile: rejected for v1 because it risks divergence from verified candidate facts.
- Ad hoc prompt-only preferences: rejected because repeat searches and proposal validation need persistent preferences.

## Decision: Proposal drafting must be evidence-aware and source-bounded

**Rationale**: Freelance proposals often mention platform metrics, project understanding, prior results, or client context. The repository safety rules prohibit invented candidate or company facts, and this feature extends that to client facts, project details, earnings, platform ratings, and freelancer metrics. Proposal material should use only verified profile facts and verified opportunity details, and flag unsupported claims.

**Alternatives considered**:

- Automatically create persuasive claims from the listing: rejected because it can fabricate experience or project understanding.
- Only produce generic templates: rejected because the user needs tailored proposal support to act on high-fit opportunities.

## Decision: Extend application history with freelance-specific states

**Rationale**: Freelance work has statuses that differ from full-time applications, such as proposal drafted, active contract, won, lost, scam, ignored, and archived. Extending the existing tracker concept keeps one outcome history while supporting freelance-specific lifecycle states and source references.

**Alternatives considered**:

- Separate tracker per platform: rejected because the user needs a unified view across Fiverr, Upwork, social, Google, and supplied leads.
- No history for freelance leads: rejected because duplicate prevention, follow-up, and outcome learning depend on saved records.
