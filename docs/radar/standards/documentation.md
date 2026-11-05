# Standard — how to edit the docs

Keep each file small and owned by one concern. Markdown is for **human-editable policy, facts and rationale**; YAML is for **validated runtime configuration**; the database is for **live overrides and history**.

## Names and IDs

- BRD: `RADAR-BRD`.
- Functional requirements: `FR-NNN`; business rules: `BR-NNN`; acceptance cases: `AC-NNN`.
- Do not reuse an existing ID. Update links when relocating a requirement.
- Write a short file for one domain; avoid a 1,000-line master spec.

## Change protocol

1. Identify owning doc from `docs/radar/README.md`.
2. Change rule/decision there; adjust config example only if the default changes.
3. Update or add a matching acceptance scenario.
4. Run Markdown link checks, YAML validation, `cargo fmt`, `clippy`, tests once code exists.
5. Keep a one-line rationale/date in PR description; never publish private profile data.

## Avoid duplication

- No numeric quota in BRD if YAML is source of truth.
- No duplicated persona facts across proposal templates and main identity profile.
- No hidden mutable model memories controlling the product; changes to learned preferences must be inspectable.
- Update schema and typed validation whenever adding a new YAML key.
