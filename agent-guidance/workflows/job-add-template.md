# Job Add Template Workflow

Canonical workflow for registering a custom CV or cover-letter template.

## Steps

1. Determine whether the template is for CVs or cover letters.
2. Inspect the template source and infer compile engine, fonts, layout, page limits, and pitfalls.
3. Store template files under `templates/`.
4. Run a compile smoke test.
5. Add an active-template managed block to the relevant guidance file.

## Rules

- Do not activate a template that has not compiled.
- Keep activation reversible.
