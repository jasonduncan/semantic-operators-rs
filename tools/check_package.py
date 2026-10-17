#!/usr/bin/env python3
"""Build the actual Cargo archive and check downstream default/all-feature consumers."""

import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def run(*args, **kwargs):
    subprocess.run(args, cwd=ROOT, check=True, **kwargs)


def main():
    package = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]
    tree = subprocess.check_output(
        ["cargo", "tree", "--locked", "--offline", "--no-default-features", "--edges", "normal", "--prefix", "none"],
        cwd=ROOT, text=True,
    )
    forbidden = {"tokio", "reqwest", "hyper", "rustls"}
    if any(line.split()[0] in forbidden for line in tree.splitlines() if line):
        raise SystemExit("Default dependency graph unexpectedly contains transport/runtime")
    run("cargo", "package", "--locked", "--offline", "--allow-dirty", "--no-verify")
    name = f"{package['name']}-{package['version']}"
    archive = ROOT / "target" / "package" / f"{name}.crate"
    with tempfile.TemporaryDirectory(prefix="semops-consumer-") as temp:
        directory = Path(temp)
        with tarfile.open(archive) as source:
            names = source.getnames()
            files = {n.removeprefix(name + "/") for n in names}
            allowed = {"Cargo.toml", "Cargo.toml.orig", "Cargo.lock", "CRATE.md",
                       "CHANGELOG.md", "LICENSE", ".cargo_vcs_info.json"}
            if any(n not in allowed and not n.startswith(("src/", "examples/")) for n in files):
                raise SystemExit("Unexpected file entered Cargo archive")
            if not {"CHANGELOG.md", "LICENSE", "CRATE.md", "src/lib.rs"} <= files:
                raise SystemExit("Required release content missing from Cargo archive")
            source.extractall(directory, filter="data")
        crate = directory / name
        consumer = directory / "consumer"
        (consumer / "src").mkdir(parents=True)
        (consumer / "src/main.rs").write_text('''use semantic_operators::{Boolean, Operator, Result, State, wire};
fn main() -> Result<()> {
    let operator = Operator::new("ready", Boolean::new("Is it ready?")?)?;
    let state = State::text("Synthetic consumer input");
    assert_eq!(operator.name(), "ready");
    let request = wire::decode_request(serde_json::json!({
        "state": state.as_value(),
        "questions": [{"name":"ready", "type":"boolean", "instructions":"Is it ready?"}]
    }))?;
    assert_eq!(request.questions.len(), 1);
    Ok(())
}
''')
        # A new consumer resolves its own dependency graph, rather than inheriting
        # the library's lockfile. Reuse downloaded crates and compiled artifacts.
        env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target" / "consumer"))
        for features in [[], ["typesafe", "images", "blocking"]]:
            import json
            (consumer / "Cargo.toml").write_text(f'''[package]
name = "semops-downstream-check"
version = "0.0.0"
edition = "2024"
rust-version = "1.85"
[dependencies]
semantic-operators = {{ path = {json.dumps(str(crate))}, default-features = false, features = {json.dumps(features)} }}
serde_json = "1"
''')
            run("cargo", "run", "--offline", "--manifest-path", str(consumer / "Cargo.toml"), env=env)
    print("Package archive, default dependency isolation, and both downstream consumers passed")


if __name__ == "__main__":
    main()
