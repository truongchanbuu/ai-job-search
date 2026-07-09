# Job Apply Workflow

Canonical workflow for applying to one job posting.

## Inputs

- Job posting URL or pasted text.
- Candidate profile and writing guidance.
- CV and cover-letter templates.
- Optional salary data from `salary_data.json`.

## Steps

1. Fetch or parse the job posting.
2. Evaluate fit before drafting.
3. Ask whether to proceed if the fit is weak or uncertain.
4. Draft a tailored CV in `cv/main_<company>.tex`.
5. Draft a tailored cover letter in `cover_letters/cover_<company>_<role>.tex`.
6. Review factual accuracy, targeting, tone, and company claims.
7. Compile the CV with `lualatex` and the cover letter with `xelatex`.
8. Visually inspect compiled PDFs.
9. Run ATS text extraction with `pdftotext` when available.
10. Present final files, verification status, and key tailoring decisions.

## Rules

- Never invent skills, experience, company facts, or role requirements.
- Verify company-specific claims before using them.
- Keep CV to exactly two pages unless a custom template says otherwise.
- Keep cover letter to exactly one page.
