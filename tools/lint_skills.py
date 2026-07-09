#!/usr/bin/env python3
"""Lint the repo's skill, command, settings, and guide files.

Run from anywhere: python tools/lint_skills.py

Checks:
- Every SKILL.md under .claude/skills/* and .agents/skills/* has YAML
  frontmatter with non-empty name and description keys.
- `allowed-tools` entries of the form `Bash(bun run <path> *)` point at files
  that exist.
- Every .claude/commands/*.md starts with a `# /<name>` title.
- .claude/settings.json remains valid for Claude compatibility.
- guide.md lists every Codex skill from .agents/skills/*/SKILL.md.
"""

import json
import re
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    sys.exit("lint_skills.py requires PyYAML: pip install pyyaml")

ROOT = Path(__file__).resolve().parent.parent
errors: list[str] = []


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT))


def read_frontmatter(path: Path) -> dict:
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        errors.append(f"{rel(path)}: missing YAML frontmatter (file must start with ---)")
        return {}
    end = text.find("\n---", 4)
    if end == -1:
        errors.append(f"{rel(path)}: unterminated YAML frontmatter")
        return {}
    try:
        data = yaml.safe_load(text[4:end])
    except yaml.YAMLError as exc:
        errors.append(f"{rel(path)}: frontmatter is not valid YAML: {exc}")
        return {}
    if not isinstance(data, dict):
        errors.append(f"{rel(path)}: frontmatter did not parse to a mapping")
        return {}
    return data


def check_skill(path: Path) -> str | None:
    data = read_frontmatter(path)
    if not data:
        return None
    for key in ("name", "description"):
        if not data.get(key):
            errors.append(f"{rel(path)}: frontmatter missing required key '{key}'")

    allowed = data.get("allowed-tools", "")
    if isinstance(allowed, str):
        for match in re.finditer(r"bun run ([^\s)]+)", allowed):
            target = match.group(1).rstrip("*")
            if not target or target.endswith("/"):
                continue
            if "*" in target:
                if not list(ROOT.glob(target)) and not list((ROOT / ".agents").glob(target)):
                    errors.append(f"{rel(path)}: allowed-tools glob matches no files: {target}")
            else:
                candidates = [ROOT / target, ROOT / ".agents" / target]
                if not any(c.is_file() for c in candidates):
                    errors.append(f"{rel(path)}: allowed-tools references a missing file: {target}")
    name = data.get("name")
    return str(name) if name else None


def check_command(path: Path) -> None:
    lines = path.read_text(encoding="utf-8").lstrip().splitlines()
    first = lines[0] if lines else ""
    if not first.startswith("# /"):
        errors.append(f"{rel(path)}: command file must start with a '# /<name>' title (found: {first[:50]!r})")


def check_settings() -> None:
    path = ROOT / ".claude" / "settings.json"
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        errors.append(f".claude/settings.json: {exc}")
        return
    if not isinstance(data.get("permissions", {}).get("allow"), list):
        errors.append(".claude/settings.json: expected permissions.allow to be a list")


def check_guide(codex_skill_names: list[str]) -> None:
    path = ROOT / "guide.md"
    try:
        text = path.read_text(encoding="utf-8")
    except OSError as exc:
        errors.append(f"guide.md: unreadable: {exc}")
        return
    for name in sorted(codex_skill_names):
        if f"`{name}`" not in text:
            errors.append(f"guide.md: missing Codex skill entry `{name}`")


def main() -> int:
    claude_skills = sorted(ROOT.glob(".claude/skills/*/SKILL.md"))
    codex_skills = sorted(ROOT.glob(".agents/skills/*/SKILL.md"))
    commands = sorted((ROOT / ".claude" / "commands").glob("*.md"))
    if not codex_skills:
        errors.append("no Codex SKILL.md files found under .agents/skills/")
    if not claude_skills:
        errors.append("no Claude compatibility SKILL.md files found under .claude/skills/")
    if not commands:
        errors.append("no command files found under .claude/commands/")

    for skill in claude_skills:
        check_skill(skill)

    codex_skill_names: list[str] = []
    for skill in codex_skills:
        name = check_skill(skill)
        if name:
            codex_skill_names.append(name)

    for command in commands:
        check_command(command)
    check_settings()
    check_guide(codex_skill_names)

    if errors:
        print(f"lint_skills: {len(errors)} failure(s)")
        for err in errors:
            print(f"  - {err}")
        return 1
    print(
        f"lint_skills: OK ({len(codex_skills)} Codex skills, "
        f"{len(claude_skills)} Claude skills, {len(commands)} commands, guide.md)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
