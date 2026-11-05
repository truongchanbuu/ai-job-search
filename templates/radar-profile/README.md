# Personal knowledge base templates

Copy **each** `.md` from this directory to `private/profiles/` and replace placeholders. Never commit completed copies to the public repository.

| File | What to edit |
|---|---|
| `identity.md` | Verified identity, experience and skills |
| `mindset.md` | Values, motivation, desired work type |
| `availability.md` | Work status and realistic capacity |
| `job-preferences.md` | Desired roles and avoid-list |
| `portfolio.md` | Projects, links and truthfully attributable work |
| `proposal-style.md` | Preferred tone and proposals |
| `lessons-learned.md` | Feedback-derived observations and hypotheses |

**Source hierarchy:** verified facts > user-approved preferences > explicit feedback > model suggestions. Never turn a suggestion into a fact. Cloud sync must protect these files and preserve revision history.

## Phase 1 typed metadata

`identity.md` requires YAML front matter containing `skills`, a map of lowercase
skill names to `{ proof, context }`. Context is one of `employment`, `contract`,
`academic`, `training`, `personal`. Every entry requires real verified proof.
`job-preferences.md` requires `preferred_categories`, `blocked_categories` and
a confirmed positive `minimum_hourly_usd`. Empty template metadata deliberately
fails validation until completed. Other modules remain versioned prose.

For runnable synthetic examples, see `rust/fixtures/profiles/`. Those fixtures
contain no personal facts. See the [local core guide](../../docs/radar/plan/local-core.md)
for commands and the distinction between capacity config and profile prose.
