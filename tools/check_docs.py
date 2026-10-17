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
fixtures = {
    "python-parity.json": "export_python_fixtures.py",
    "python-core-edges.json": "export_core_edges.py",
    "python-recipes.json": "export_recipe_fixtures.py",
    "python-bench.json": "export_bench_fixtures.py",
}
for name, generator in fixtures.items():
    fixture = json.loads((ROOT / "tests/fixtures" / name).read_text())
    if fixture["reference_revision"] != baseline["sources"][0]["revision"]:
        errors.append(f"{name}: reference revision differs from source inventory")
    generator_hash = hashlib.sha256((ROOT / "tools" / generator).read_bytes()).hexdigest()
    if fixture["generator_sha256"] != generator_hash:
        errors.append(f"{name}: generator changed; inspect and regenerate reference outputs")
if errors:
    raise SystemExit("\n".join(errors))
print(f"Checked {len(documents)} Markdown files, {links} local file links, and fixture provenance")

