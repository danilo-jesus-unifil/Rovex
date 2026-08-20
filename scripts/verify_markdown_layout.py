#!/usr/bin/env python3
"""Validate the repository's Markdown layout and local Markdown links."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ALLOWED_ROOT_MARKDOWN = {"README.md", "CHANGELOG.md", "SECURITY.md", "COMPATIBILITY.md"}
LOCAL_LINK = re.compile(r"\]\((<[^>]+>|[^)\s]+)")
REFERENCE_LINK = re.compile(r"^\s*\[[^\]]+\]:\s+(<[^>]+>|[^\s]+)", re.MULTILINE)


def markdown_files(root: Path) -> list[Path]:
    return sorted(
        path
        for path in root.rglob("*.md")
        if ".git" not in path.parts and "target" not in path.parts
    )


def local_targets(text: str) -> list[str]:
    targets = LOCAL_LINK.findall(text) + REFERENCE_LINK.findall(text)
    result = []
    for raw in targets:
        target = raw.strip("<>").split("#", 1)[0]
        if not target or "://" in target or target.startswith("mailto:"):
            continue
        result.append(target)
    return result


def broken_links(root: Path) -> list[str]:
    errors = []
    for source in markdown_files(root):
        for target in local_targets(source.read_text(encoding="utf-8")):
            resolved = (source.parent / target).resolve()
            if not resolved.exists():
                errors.append(
                    f"{source.relative_to(root)} -> {target} "
                    f"(resolved {resolved.relative_to(root) if resolved.is_relative_to(root) else resolved})"
                )
    return errors


def root_layout_violations(root: Path) -> list[str]:
    return sorted(
        path.name
        for path in root.glob("*.md")
        if path.name not in ALLOWED_ROOT_MARKDOWN
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", nargs="?", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    root = args.root.resolve()
    violations = root_layout_violations(root)
    links = broken_links(root)
    print(f"Markdown files: {len(markdown_files(root))}")
    print(f"Root layout: {len(violations)} violation(s)")
    print(f"Broken local links: {len(links)}")
    if violations:
        print("Unexpected Markdown files in repository root:")
        print("\n".join(f"- {name}" for name in violations))
    if links:
        print("Broken local Markdown links:")
        print("\n".join(f"- {item}" for item in links))
    if violations or links:
        return 1
    print("Markdown layout and local links are valid.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
