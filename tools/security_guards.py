#!/usr/bin/env python3
"""Supply-chain guards for the template's riskiest surfaces.

Run from anywhere: python tools/security_guards.py

This repo is Codex-native and Claude-compatible. These guards make risky
changes explicit:

1. .claude/settings.json permissions must remain in the reviewed allowlist.
2. .gitignore must keep personal-data protection rules.
3. .agents/**/package.json must not add lifecycle scripts or trustedDependencies.
4. Codex job workflow skills must reference shared guidance under agent-guidance/.
"""

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
errors: list[str] = []

ALLOWED_PERMISSIONS = {
    "Skill(job-application-assistant)",
    "Bash(bun run:*)",
    "Bash(python salary_lookup.py:*)",
    "Bash(python3 salary_lookup.py:*)",
    "Bash(pdftotext:*)",
}

REQUIRED_IGNORE_RULES = [
    "salary_data.json",
    "job_scraper/seen_jobs.json",
    "cv/main_*.tex",
    "!cv/main_example.tex",
    "cover_letters/cover_*.tex",
    "documents/cv/**",
    "documents/linkedin/**",
    "documents/diplomas/**",
    "documents/references/**",
    "documents/applications/**",
    "job_search_tracker.csv",
]

FORBIDDEN_SCRIPTS = {"preinstall", "install", "postinstall", "prepare", "prepack"}


def check_permissions() -> None:
    path = ROOT / ".claude" / "settings.json"
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        errors.append(f".claude/settings.json: unreadable or invalid JSON: {exc}")
        return
    allow = data.get("permissions", {}).get("allow", [])
    for entry in allow:
        if entry not in ALLOWED_PERMISSIONS:
            errors.append(
                f".claude/settings.json: permission not in reviewed allowlist: {entry!r}. "
                "If intentional, update ALLOWED_PERMISSIONS in tools/security_guards.py in the same PR."
            )


def check_gitignore() -> None:
    path = ROOT / ".gitignore"
    try:
        rules = {line.strip() for line in path.read_text(encoding="utf-8").splitlines()}
    except OSError as exc:
        errors.append(f".gitignore: unreadable: {exc}")
        return
    for rule in REQUIRED_IGNORE_RULES:
        if rule not in rules:
            errors.append(f".gitignore: required personal-data rule missing: {rule!r}")


def check_package_manifests() -> None:
    manifests = [
        p for p in ROOT.glob(".agents/**/package.json") if "node_modules" not in p.parts
    ]
    if not manifests:
        errors.append(".agents: no package.json files found - glob roots are wrong or the tree moved")
    for manifest in manifests:
        relpath = manifest.relative_to(ROOT)
        try:
            data = json.loads(manifest.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as exc:
            errors.append(f"{relpath}: unreadable or invalid JSON: {exc}")
            continue
        bad = FORBIDDEN_SCRIPTS & set(data.get("scripts", {}))
        if bad:
            errors.append(f"{relpath}: lifecycle script(s) {sorted(bad)} are forbidden")
        if "trustedDependencies" in data:
            errors.append(f"{relpath}: trustedDependencies is forbidden")


def check_codex_job_skills() -> None:
    for skill in sorted((ROOT / ".agents" / "skills").glob("job-*/SKILL.md")):
        text = skill.read_text(encoding="utf-8")
        if "agent-guidance/" not in text:
            errors.append(f"{skill.relative_to(ROOT)}: job workflow skill must reference agent-guidance/")
    if not (ROOT / "AGENTS.md").is_file():
        errors.append("AGENTS.md: missing Codex project guide")
    if not (ROOT / "guide.md").is_file():
        errors.append("guide.md: missing skill catalog")


def main() -> int:
    check_permissions()
    check_gitignore()
    check_package_manifests()
    check_codex_job_skills()
    if errors:
        print(f"security_guards: {len(errors)} failure(s)")
        for err in errors:
            print(f"  - {err}")
        return 1
    print("security_guards: OK (permissions, gitignore, package manifests, Codex skill guardrails)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
