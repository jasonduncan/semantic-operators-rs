#!/usr/bin/env python3
"""Check repository-local Markdown file links and fixture provenance without network access."""

import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
errors = []
links = 0
documents = [path for path in ROOT.rglob("*.md") if not any(
    part in {".git", "target"} for part in path.relative_to(ROOT).parts)]
for path in documents:
    for link in re.findall(r"\[[^\]]*\]\(([^)]+)\)", path.read_text()):
        if "://" in link or link.startswith(("#", "mailto:")):
            continue
        destination = link.split("#", 1)[0]
        if destination and not (path.parent / destination).exists():
            errors.append(f"{path.relative_to(ROOT)}: missing {destination}")
        links += 1
baseline = json.loads((ROOT / "docs/source-baseline.json").read_text())
fixture = json.loads((ROOT / "tests/fixtures/python-parity.json").read_text())
if fixture["reference_revision"] != baseline["sources"][0]["revision"]:
    errors.append("Fixture source revision differs from source inventory")
generator_hash = hashlib.sha256((ROOT / "tools/export_python_fixtures.py").read_bytes()).hexdigest()
if fixture["generator_sha256"] != generator_hash:
    errors.append("Fixture generator changed; inspect and regenerate expected results")
if errors:
    raise SystemExit("\n".join(errors))
print(f"Checked {len(documents)} Markdown files, {links} local file links, and fixture provenance")

