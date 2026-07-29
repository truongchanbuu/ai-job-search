# ITviec URL and Access Reference

**Access reviewed**: 2026-07-28

- Search slugs: `https://itviec.com/it-jobs/<skill-or-level>`
- Optional city suffix: `/ho-chi-minh-hcm`, `/ha-noi`, or `/da-nang`
- Details: `/it-jobs/<title-company-slug>-<numeric-id>`
- Public pages remain discoverable in an ordinary browser/search index, but the
  single low-volume adapter smoke request on 2026-07-28 returned a challenge
  page. The adapter reported `RESTRICTED`; it did not retry around the control.
- Login-only salary/apply actions remain out of scope.

Use low-volume requests, canonical links, and bounded normalized evidence.
