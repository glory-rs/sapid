"""Build and retain bounded, content-addressed Pages assets (Python stdlib only)."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import shutil
from urllib.error import HTTPError
from urllib.parse import urljoin, urlsplit
from urllib.request import Request, urlopen

MAX_FILE = 25 * 1024 * 1024
MAX_TOTAL = 200 * 1024 * 1024
MAX_FILES = 500
MANIFEST = "asset-manifest.json"
DIRECTORY = re.compile(r"(?:assets|media)-[a-f0-9]{20}")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def path_name(name):
    path = PurePosixPath(name)
    if (not name or path.is_absolute() or str(path) != name
            or any(part in (".", "..") for part in path.parts)
            or not re.fullmatch(r"[A-Za-z0-9_.\-/]+", name)):
        raise ValueError(f"Unsafe asset path: {name}")
    return name


def records(directory, names):
    result = {}
    for name in sorted(set(names)):
        data = (directory / path_name(name)).read_bytes()
        if len(data) > MAX_FILE:
            raise ValueError(f"Pages asset exceeds 25 MiB: {name}")
        result[name] = {"sha256": sha(data), "bytes": len(data)}
    return result


def fingerprint(files):
    return sha(json.dumps(files, sort_keys=True, separators=(",", ":")).encode())[:20]


def prepare(source, output):
    files = records(source, (p.relative_to(source).as_posix() for p in source.rglob("*") if p.is_file()))
    group = f"media-{fingerprint(files)}"
    for name in files:
        target = output / group / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source / name, target)
    print(group)


def write_manifest(output, releases):
    (output / MANIFEST).write_text(
        json.dumps({"schema": 1, "releases": releases}, indent=2) + "\n", encoding="utf-8")


def bundle(output):
    names = [p.relative_to(output).as_posix() for p in output.rglob("*")
             if p.is_file() and not DIRECTORY.fullmatch(p.relative_to(output).parts[0])]
    if not names or any(not name.endswith((".css", ".js", ".wasm", ".d.ts")) for name in names):
        raise ValueError("Bundle input must contain only bindgen output and stylesheets")
    files = records(output, names)
    group = f"assets-{fingerprint(files)}"
    for name in names:
        target = output / group / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.move(output / name, target)
    media = [p.relative_to(output).as_posix() for group_path in output.glob("media-*")
             for p in group_path.rglob("*") if p.is_file()]
    all_names = [f"{group}/{name}" for name in names] + media
    write_manifest(output, [{"id": group, "files": records(output, all_names)}])
    print(group)


def read_manifest(data):
    manifest = json.loads(data)
    if manifest.get("schema") != 1 or not 1 <= len(manifest.get("releases", [])) <= 4:
        raise ValueError("Unsupported asset manifest")
    total = 0
    for release in manifest["releases"]:
        if not re.fullmatch(r"(?:assets|legacy)-[a-f0-9]{20}", release["id"]):
            raise ValueError("Invalid release identity")
        if not 1 <= len(release["files"]) <= MAX_FILES:
            raise ValueError("Invalid asset count")
        for name, info in release["files"].items():
            path_name(name)
            # Retention is asset-only. Never overwrite entry documents or Pages configuration.
            first = name.split("/")[0]
            if not DIRECTORY.fullmatch(first) and not (
                    release["id"].startswith("legacy-") and (
                        first in ("assets", "snippets") or name.endswith((".js", ".wasm", ".css")))):
                raise ValueError(f"Not a retained asset: {name}")
            if not re.fullmatch(r"[a-f0-9]{64}", info["sha256"]) or not 0 <= info["bytes"] <= MAX_FILE:
                raise ValueError(f"Invalid asset metadata: {name}")
            total += info["bytes"]
    if total > MAX_TOTAL:
        raise ValueError("Retained asset budget exceeded")
    return manifest


def download(origin, name, limit=MAX_FILE):
    url = urljoin(origin + "/", name)
    request = Request(url, headers={"Cache-Control": "no-cache", "User-Agent": "Pages-Asset-Retention/1"})
    with urlopen(request, timeout=60) as response:
        if urlsplit(response.url).netloc != urlsplit(origin).netloc:
            raise ValueError("Unexpected asset redirect")
        data = response.read(limit + 1)
        if len(data) > limit:
            raise ValueError(f"Asset download too large: {name}")
        return data


def bootstrap(origin, output, current):
    """Carry the deployed pre-manifest release through the first migration."""
    html = download(origin, "", 2 * 1024 * 1024).decode("utf-8")
    pending = set(re.findall(r'''(?:href|src)=["']([^"']+\.(?:css|ico|svg))["']''', html))
    pending.update(re.findall(r'''from\s+["']([^"']+\.js)["']''', html))
    # Include media used only after navigation, using this site's own known media names.
    for name in current["files"]:
        if name.startswith("media-"):
            pending.add("assets/" + name.split("/", 1)[1])
    files = {}
    while pending:
        name = pending.pop().removeprefix("/").removeprefix("./")
        if ":" in name or name in files:
            continue
        path_name(name)
        if len(files) >= MAX_FILES:
            raise ValueError("Legacy asset graph too large")
        data = download(origin, name)
        files[name] = {"sha256": sha(data), "bytes": len(data)}
        if sum(info["bytes"] for info in files.values()) > MAX_TOTAL:
            raise ValueError("Legacy asset budget exceeded")
        target = output / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
        if name.endswith(".js"):
            source = data.decode("utf-8")
            references = re.findall(r'''["'](\./snippets/[^"']+\.js)["']''', source)
            references += re.findall(r'''new URL\(["']([^"']+\.wasm)["']''', source)
            for reference in references:
                pending.add(str(PurePosixPath(name).parent / reference.removeprefix("./")))
    release = {"id": f"legacy-{fingerprint(files)}", "files": files}
    read_manifest(json.dumps({"schema": 1, "releases": [release]}))
    return release


def retain(output, origin):
    parsed = urlsplit(origin)
    if parsed.scheme != "https" or parsed.path not in ("", "/") or parsed.username or parsed.query:
        raise ValueError("Retention origin must be an HTTPS site root")
    origin = origin.rstrip("/")
    current = read_manifest((output / MANIFEST).read_text(encoding="utf-8"))["releases"][0]
    try:
        data = download(origin, MANIFEST, 512 * 1024)
        # A pre-manifest SPA may answer unknown paths with its index document (200).
        previous = ([bootstrap(origin, output, current)] if data.lstrip().lower().startswith(b"<!doctype html")
                    else read_manifest(data)["releases"])
    except HTTPError as error:
        if error.code != 404:
            raise
        previous = [bootstrap(origin, output, current)]
    releases = [current]
    for release in previous:
        if release["id"] in {r["id"] for r in releases}:
            continue
        if len(releases) == 4:
            break
        for name, info in release["files"].items():
            target = output / path_name(name)
            if target.exists():
                data = target.read_bytes()
            else:
                data = download(origin, name)
            if sha(data) != info["sha256"] or len(data) != info["bytes"]:
                raise ValueError(f"Retained asset integrity mismatch: {name}")
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        releases.append(release)
    # Fail the deployment rather than silently lose previous asset generations.
    read_manifest(json.dumps({"schema": 1, "releases": releases}))
    write_manifest(output, releases)
    if (output / "asset-hashes.json").is_file():
        files = records(output, (p.relative_to(output).as_posix() for p in output.rglob("*")
                                 if p.is_file() and p.name != "asset-hashes.json"))
        index = [{"path": name, **info} for name, info in files.items()]
        (output / "asset-hashes.json").write_text(json.dumps(index, indent=2) + "\n", encoding="utf-8")
    print(f"Retained {len(releases) - 1} previous releases")


def validate(output):
    manifest = read_manifest((output / MANIFEST).read_text(encoding="utf-8"))
    for release in manifest["releases"]:
        actual = records(output, release["files"])
        if actual != release["files"]:
            raise ValueError(f"Asset integrity mismatch: {release['id']}")
        for name in actual:
            if name.endswith(".js"):
                source = (output / name).read_text(encoding="utf-8")
                references = re.findall(r'''["'](\./snippets/[^"']+\.js)["']''', source)
                references += re.findall(r'''new URL\(["']([^"']+\.wasm)["']''', source)
                for reference in references:
                    dependency = str(PurePosixPath(name).parent / reference.removeprefix("./"))
                    if dependency not in actual:
                        raise ValueError(f"Missing relative loader dependency: {dependency}")
    current = manifest["releases"][0]
    for html in output.rglob("*.html"):
        text = html.read_text(encoding="utf-8")
        for group in re.findall(r"(?:assets|media)-[a-f0-9]{20}", text):
            if not any(name.startswith(group + "/") for name in current["files"]):
                raise ValueError(f"HTML points outside current release: {html}")
    print(f"Verified {len(manifest['releases'])} asset releases")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("prepare", "bundle", "retain", "validate"))
    parser.add_argument("output", type=Path)
    parser.add_argument("source", nargs="?")
    args = parser.parse_args()
    if not str(args.output.resolve()).lower().startswith("d:\\buildcache\\"):
        parser.error("Asset output must be inside D:/BuildCache")
    if args.command == "prepare":
        prepare(Path(args.source), args.output)
    elif args.command == "bundle":
        bundle(args.output)
    elif args.command == "retain":
        retain(args.output, args.source)
    else:
        validate(args.output)
