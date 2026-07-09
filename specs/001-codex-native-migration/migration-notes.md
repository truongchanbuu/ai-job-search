# Migration Notes

Codex is now the primary workflow surface, with Claude Code retained as a compatibility surface.

Implementation principles:

- Shared behavior belongs in `agent-guidance/`.
- Codex skills live in `.agents/skills/`.
- Claude commands and skills remain available under `.claude/`.
- `guide.md` is the human-readable skill catalog.
