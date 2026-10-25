"""Regression tests for publishing new assets without breaking previous HTML."""
from contextlib import redirect_stdout
import importlib.util
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("assets", Path(__file__).with_name("site-assets.py"))
assets = importlib.util.module_from_spec(spec)
spec.loader.exec_module(assets)


def quietly(function, *args):
    with redirect_stdout(io.StringIO()):
        return function(*args)


class AssetTests(unittest.TestCase):
    def setUp(self):
        root = Path(os.environ.get("SITE_CACHE_TEST_ROOT", "D:/BuildCache/site-cache-tests/tmp"))
        if not str(root.resolve()).lower().startswith("d:\\buildcache\\"):
            raise RuntimeError("Test fixtures must stay under D:/BuildCache")
        root.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=root)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def build(self, name, css="body{color:red}", media=b"logo", wasm=b"wasm-v1"):
        output = self.root / name
        output.mkdir()
        source = self.root / (name + "-media")
        source.mkdir()
        (source / "logo.svg").write_bytes(media)
        quietly(assets.prepare, source, output)
        (output / "site.css").write_text(css)
        (output / "app.js").write_text("import './snippets/host.js';new URL('app_bg.wasm', import.meta.url)")
        (output / "app_bg.wasm").write_bytes(wasm)
        (output / "snippets").mkdir()
        (output / "snippets/host.js").write_text("export const host = true;")
        quietly(assets.bundle, output)
        release = json.loads((output / assets.MANIFEST).read_text())["releases"][0]
        (output / "index.html").write_text(f'<link href="/{release["id"]}/site.css">')
        quietly(assets.validate, output)
        return output, release

    @staticmethod
    def remote(output):
        return {p.relative_to(output).as_posix(): p.read_bytes() for p in output.rglob("*") if p.is_file()}

    def test_fingerprint_tracks_css_wasm_and_media_content(self):
        _, one = self.build("one")
        _, same = self.build("same")
        _, css = self.build("css", css="body{color:blue}")
        _, wasm = self.build("wasm", wasm=b"wasm-v2")
        _, media = self.build("media", media=b"changed-logo")
        self.assertEqual(one, same)
        self.assertNotEqual(one["id"], css["id"])
        self.assertNotEqual(one["id"], wasm["id"])
        media_names = lambda r: [p for p in r["files"] if p.startswith("media-")]
        self.assertNotEqual(media_names(one), media_names(media))

    def test_rollover_keeps_last_three_old_graphs_and_wasm(self):
        previous, old = self.build("first")
        for index in range(1, 6):
            output, current = self.build(f"v{index}", wasm=f"wasm-{index}".encode())
            remote = self.remote(previous)
            with patch.object(assets, "download", side_effect=lambda origin, name, *limit: remote[name]):
                quietly(assets.retain, output, "https://example.test")
            quietly(assets.validate, output)
            manifest = json.loads((output / assets.MANIFEST).read_text())
            self.assertEqual(len(manifest["releases"]), min(index + 1, 4))
            self.assertEqual(manifest["releases"][0], current)
            old_wasm = output / old["id"] / "app_bg.wasm"
            self.assertEqual(old_wasm.exists(), index <= 3)
            if index <= 3:
                self.assertEqual(old_wasm.read_bytes(), b"wasm-v1")
            previous = output

    def test_corrupt_remote_stops_publish(self):
        previous, old = self.build("first")
        output, _ = self.build("new", wasm=b"new-wasm")
        remote = self.remote(previous)
        remote[old["id"] + "/app_bg.wasm"] = b"corruption"
        with patch.object(assets, "download", side_effect=lambda origin, name, *limit: remote[name]):
            with self.assertRaisesRegex(ValueError, "integrity mismatch"):
                quietly(assets.retain, output, "https://example.test")

    def test_legacy_spa_bootstrap_keeps_relative_snippets_and_media(self):
        output, _ = self.build("new")
        remote = {
            assets.MANIFEST: b"<!doctype html><html>SPA fallback</html>",
            "": b'''<link href="/site.css"><script type="module">import init from '/app.js'</script>''',
            "site.css": b"old styles",
            "app.js": b"import './snippets/host.js';new URL('app_bg.wasm',import.meta.url)",
            "app_bg.wasm": b"old-wasm",
            "snippets/host.js": b"export const host=true;",
            "assets/logo.svg": b"old-logo",
        }
        with patch.object(assets, "download", side_effect=lambda origin, name, *limit: remote[name]):
            quietly(assets.retain, output, "https://example.test")
        quietly(assets.validate, output)
        self.assertEqual((output / "app_bg.wasm").read_bytes(), b"old-wasm")
        self.assertEqual((output / "assets/logo.svg").read_bytes(), b"old-logo")

    def test_rejects_traversal_and_missing_loader_dependency(self):
        with self.assertRaises(ValueError):
            assets.path_name("../index.html")
        output, release = self.build("one")
        name = release["id"] + "/snippets/host.js"
        del release["files"][name]
        assets.write_manifest(output, [release])
        with self.assertRaisesRegex(ValueError, "Missing relative loader dependency"):
            quietly(assets.validate, output)


if __name__ == "__main__":
    unittest.main()
