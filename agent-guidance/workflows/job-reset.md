# Job Reset Workflow

Canonical workflow for clearing profile or document state.

## Steps

1. Parse requested reset scope: profile, documents, or both.
2. Show exactly what will be cleared.
3. Require explicit confirmation before destructive changes.
4. Reset only the selected scope.
5. Report what changed and what remains.

## Rules

- Do not delete without explicit confirmation.
- Preserve framework structure and templates.
