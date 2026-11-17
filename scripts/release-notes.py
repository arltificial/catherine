#!/usr/bin/env python3
"""Extract one release body from CHANGELOG.md.

Purpose: the release body is the changelog section for the tagged version,
plus the compare link to the previous tag (beads catherine-ply). GitHub's
`--generate-notes` walks merge commits and finds nothing on this repository
(no PRs are merged here), so the published body was empty.

Responsibilities: locate the `## [<version>]` section, refuse loudly when it
is absent or empty, and append the compare link for the previous release tag.

Rationale: the changelog is the reviewed, committed source of truth for what
changed; deriving the body from it cannot drift from the code the way
generated notes did. An absent section is a release blocker, not an empty
body — the gate fails with the version named so the fix is obvious.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

HEADING = re.compile(r"^## \[(?P<version>[^\]]+)\](?P<rest>.*)$")


def section_body(changelog: str, version: str) -> str | None:
    """Return the body under `## [version]`, or None when absent."""
    lines = changelog.splitlines()
    start = None
    for index, line in enumerate(lines):
        match = HEADING.match(line)
        if match is None:
            continue
        if match.group("version").strip() == version:
            start = index + 1
        elif start is not None:
            return "\n".join(lines[start:index]).strip()
    if start is None:
        return None
    return "\n".join(lines[start:]).strip()


def previous_tag(tag: str) -> str | None:
    """The nearest earlier release tag reachable from HEAD, if any."""
    try:
        out = subprocess.run(
            ["git", "tag", "--sort=-v:refname", "--merged", "HEAD"],
            capture_output=True,
            text=True,
            check=True,
        ).stdout
    except (OSError, subprocess.CalledProcessError):
        return None
    tags = [line.strip() for line in out.splitlines() if line.strip()]
    for candidate in tags:
        if candidate != tag:
            return candidate
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True, help="release version, e.g. 0.2.0")
    parser.add_argument("--tag", required=True, help="release tag, e.g. v0.2.0")
    parser.add_argument(
        "--repo",
        default=None,
        help="GitHub owner/repo for the compare link (link omitted when unset)",
    )
    parser.add_argument("--changelog", default="CHANGELOG.md", help="changelog path")
    args = parser.parse_args()

    path = Path(args.changelog)
    try:
        changelog = path.read_text(encoding="utf-8")
    except OSError as error:
        print(f"::error::cannot read {path}: {error}", file=sys.stderr)
        return 2

    body = section_body(changelog, args.version)
    if body is None:
        print(
            f"::error::{path} has no [{args.version}] section for tag {args.tag}",
            file=sys.stderr,
        )
        return 2
    if not body:
        print(
            f"::error::{path} [{args.version}] section for tag {args.tag} is empty",
            file=sys.stderr,
        )
        return 2

    print(body)
    previous = previous_tag(args.tag)
    if args.repo and previous:
        print()
        print(
            f"**Full Changelog**: "
            f"https://github.com/{args.repo}/compare/{previous}...{args.tag}"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())