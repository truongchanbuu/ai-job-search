# google-search-cli

Supplemental Google job discovery for this workspace.

The CLI uses the official Google Programmable Search JSON API when
`GOOGLE_API_KEY` and `GOOGLE_CSE_ID` are set. Without credentials, use `queries`
to generate targeted Google URLs for manual review.

This CLI is not a Google Jobs UI implementation. Query links and API results are
public-page discovery leads and include `requiresVerification: true`; open the
original posting before ranking, saving, or drafting application materials.
Google Jobs UI links are filtered from API-normalized results.

```bash
bun run src/cli.ts queries -q "Backend Engineer Node.js" -l "Ho Chi Minh City" --jobage 30 --format table
bun run src/cli.ts search -q "Backend Engineer Node.js" -l "Ho Chi Minh City" --jobage 30 --limit 10 --format json
```
