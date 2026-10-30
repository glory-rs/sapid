"""Prepare a missing library lock once; never update an existing resolution."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess


def run(*args, capture=False):
    result = subprocess.run(args, check=True, text=True, encoding="utf-8",
                            stdout=subprocess.PIPE if capture else None)
    return result.stdout.strip() if capture else None


def prepare_library_lock(manifest, *, toolchain=None, metadata=None, runner=run):
    manifest = Path(manifest).resolve()
    configured = os.environ.get("CARGO_TARGET_DIR")
    if not configured:
        raise SystemExit("Set CARGO_TARGET_DIR before preparing a library lock")
    target = Path(configured).resolve()
    if os.name == "nt" and not target.is_relative_to(Path("D:/BuildCache").resolve()):
        raise SystemExit("Library lock artifacts must stay under D:/BuildCache")
    cargo = ["cargo"] + ([f"+{toolchain}"] if toolchain else [])
    if metadata is None:
        metadata = json.loads(runner(*cargo, "metadata", "--manifest-path", str(manifest),
                                    "--no-deps", "--format-version", "1", capture=True))
    if Path(metadata["target_directory"]).resolve() != target:
        raise SystemExit("Unexpected Cargo target while preparing a library lock")
    lock = Path(metadata.get("workspace_root", manifest.parent)) / "Cargo.lock"
    created = not lock.is_file()
    if created:
        runner(*cargo, "--config", 'resolver.incompatible-rust-versions="fallback"',
               "generate-lockfile", "--manifest-path", str(manifest))
    # Reading the actual result also refuses a successful-but-incomplete driver.
    return {"manifest": str(manifest), "lockfile": str(lock), "created": created,
            "sha256": hashlib.sha256(lock.read_bytes()).hexdigest(), "target_directory": str(target)}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest-path", default="Cargo.toml")
    parser.add_argument("--toolchain")
    options = parser.parse_args()
    result = prepare_library_lock(options.manifest_path, toolchain=options.toolchain)
    if Path(options.manifest_path).resolve() == Path(__file__).resolve().parents[1] / "Cargo.toml":
        for relative in ("examples/native/Cargo.toml", "examples/web/Cargo.toml"):
            manifest = Path(__file__).resolve().parents[1] / relative
            if manifest.is_file():
                prepare_library_lock(manifest, toolchain=options.toolchain)
    print(json.dumps(result))
