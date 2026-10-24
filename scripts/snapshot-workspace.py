"""Freeze first-party source on D: before a browser build; never copy caches.

Agents pause writes only during this copy, then can resume while Cargo builds the
immutable snapshot. Each output is new and includes hashes for reproducibility.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("output", type=Path)
args = parser.parse_args()
output = args.output.resolve()
build_root = Path("D:/BuildCache/sapid").resolve()
if not output.is_relative_to(build_root) or output == build_root:
    parser.error("snapshot output must be a new directory below D:/BuildCache/sapid")
if output.exists():
    parser.error("snapshot output already exists; use a fresh name")
sapid = Path(__file__).resolve().parents[1]
glory = sapid.parent / "glory"
if not (glory / "Cargo.toml").is_file():
    parser.error("the sibling Glory workspace was not found")
excluded = {".git", "target", "node_modules", ".venv", "__pycache__", ".codex", ".claude", ".idea", ".vscode", ".next", ".cache", ".pytest_cache"}
files = []
for source in (glory, sapid):
    for directory, children, names in os.walk(source, followlinks=False):
        children[:] = [name for name in children if name not in excluded
                       and not (Path(directory) / name).is_symlink()]
        for name in names:
            path = Path(directory) / name
            if name in excluded or name in {".npmrc", ".netrc"} or name.startswith(".env") or path.is_symlink():
                continue
            files.append((path, Path(source.name) / path.relative_to(source)))
output.mkdir(parents=True)
manifest = {}
for source, relative in files:
    target = output / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)
    manifest[relative.as_posix()] = hashlib.sha256(target.read_bytes()).hexdigest()
(output / "source-hashes.json").write_text(
    json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"Frozen {len(files)} first-party workspace files under {output}")
