"""Exercise fresh MSRV/release entrypoints with real Cargo and local Git only."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class LibraryLockTests(unittest.TestCase):
    def setUp(self):
        parent = Path("D:/BuildCache/sapid/fixes/pr12-review/lock-regressions")
        parent.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(prefix="fresh-", dir=parent)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.project = self.root / "project"
        (self.project / "src").mkdir(parents=True)
        (self.project / "Cargo.toml").write_text(
            '[package]\nname="owned-lock-fixture"\nversion="0.6.0"\nedition="2024"\n'
            'rust-version="1.94"\nlicense="MIT"\ndescription="Lock policy regression"\n'
            'repository="https://github.com/example/owned"\n'
            '[features]\nextra=[]\n[workspace]\n', encoding="utf-8")
        (self.project / "src/lib.rs").write_text("pub fn ready() {}\n", encoding="utf-8")
        self.application_locks = {}
        for name in ("web", "native"):
            lock = self.project / f"examples/{name}/Cargo.lock"
            lock.parent.mkdir(parents=True)
            lock.write_bytes(f'version = 4\n# Committed {name} application resolution\n'.encode())
            self.application_locks[lock] = lock.read_bytes()
        self.environment = {**os.environ, "CARGO_TARGET_DIR": str(self.root / "target"),
                            "CARGO_HOME": os.environ.get("CARGO_HOME") or "D:/BuildCache/glory-semantic-ui/cargo",
                            "CARGO_NET_OFFLINE": "true", "CARGO_BUILD_JOBS": "1",
                            "CARGO_INCREMENTAL": "0", "CARGO_PROFILE_DEV_DEBUG": "0",
                            "TEMP": str(self.root / "tmp"), "TMP": str(self.root / "tmp"),
                            "PYTHONPYCACHEPREFIX": str(self.root / "pycache")}
        (self.root / "tmp").mkdir()

    def command(self, *args):
        return subprocess.run(args, cwd=self.project, env=self.environment,
                              text=True, capture_output=True, timeout=120)

    def assert_applications_unchanged(self):
        for lock, expected in self.application_locks.items():
            self.assertEqual(lock.read_bytes(), expected)

    def prepare(self):
        return self.command(sys.executable, str(ROOT / "scripts/library_lock.py"))

    def test_real_msrv_fresh_default_then_locked_all_features(self):
        for features in ("default", "all"):
            result = self.command("pwsh", "-NoProfile", "-File", str(ROOT / "scripts/check-msrv.ps1"),
                                  "-ProjectRoot", str(self.project), "-FeatureSet", features)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            lock = (self.project / "Cargo.lock").read_bytes()
            if features == "default":
                first_resolution = lock
            else:
                self.assertEqual(lock, first_resolution)
            self.assert_applications_unchanged()

    def test_existing_resolution_is_preserved_byte_for_byte(self):
        result = self.prepare()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        lock = self.project / "Cargo.lock"
        before = lock.read_bytes() + b"# Retain the developer's exact local resolution.\n"
        lock.write_bytes(before)
        result = self.prepare()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertFalse(json.loads(result.stdout)["created"])
        self.assertEqual(lock.read_bytes(), before)
        self.assert_applications_unchanged()

    def test_failed_generation_is_not_silently_accepted(self):
        with (self.project / "Cargo.toml").open("a", encoding="utf-8") as stream:
            stream.write('[dependencies]\nsapid-lock-policy-missing-fixture="=0.0.0"\n')
        result = self.prepare()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("generate-lockfile", result.stderr)
        self.assertFalse((self.project / "Cargo.lock").exists())
        self.assert_applications_unchanged()

    def test_unconfigured_target_fails_before_creating_a_lock(self):
        self.environment.pop("CARGO_TARGET_DIR")
        result = self.prepare()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Set CARGO_TARGET_DIR", result.stderr)
        self.assertFalse((self.project / "Cargo.lock").exists())

    def test_real_release_archive_from_fresh_lock_keeps_git_clean(self):
        (self.project / "release.json").write_text(json.dumps({
            "repository": "example/owned", "allowed_prefixes": ["owned"],
            "tag_prefix": "v", "uniform_version": True, "packages": ["owned-lock-fixture"]}), encoding="utf-8")
        (self.project / ".gitignore").write_text("/Cargo.lock\n", encoding="utf-8")
        for args in (("init",), ("add", "."),
                     ("-c", "user.name=Lock fixture", "-c", "user.email=fixture@example.invalid", "commit", "-m", "Reviewed fixture")):
            result = self.command("git", *args)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        result = self.command(sys.executable, str(ROOT / "scripts/release.py"),
                              "--product-root", str(self.project), "--check-package", "owned-lock-fixture")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertTrue((self.project / "Cargo.lock").is_file())
        self.assertTrue((self.root / "target/package/owned-lock-fixture-0.6.0.crate").is_file())
        self.assertEqual(self.command("git", "status", "--porcelain").stdout.strip(), "")
        self.assert_applications_unchanged()


if __name__ == "__main__":
    unittest.main()
