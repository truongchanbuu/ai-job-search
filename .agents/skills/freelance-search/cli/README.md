# freelance-search-cli

Freelance opportunity discovery, readiness, evaluation, proposal drafting, and
history tracking for this workspace.

The CLI uses local file-backed records. It does not store platform passwords,
payment credentials, private messages, or secret tokens. Google/social/platform
links are discovery leads until the original opportunity is verified or supplied
by the user.

```bash
bun run src/cli.ts search --profile documents/freelance-profile.json --format table
bun run src/cli.ts readiness --platform upwork --account-status limited --known-blocker "Connects balance unknown"
bun run src/cli.ts evaluate --opportunity job_scraper/freelance/example.json --profile documents/freelance-profile.json
bun run src/cli.ts proposal --opportunity job_scraper/freelance/example.json --profile documents/freelance-profile.json
bun run src/cli.ts save --opportunity job_scraper/freelance/example.json --status proposal_drafted
```

Readiness output distinguishes setup tasks from proposal actions. Proposal
output includes evidence references and unsupported-claim warnings.
