"""Run the real RustSec auditor on isolated, dependency-free Cargo workspaces."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
AUDITOR = os.environ.get("SAPID_CARGO_AUDIT") or shutil.which("cargo-audit")

# Minimal metadata from the CC0 RustSec RUSTSEC-2020-0071 advisory:
# https://github.com/RustSec/advisory-db/blob/main/crates/time/RUSTSEC-2020-0071.md
# Keep its Unix-only affected scope: the gate must still report it on Windows.
ADVISORY = '''```toml
[advisory]
id = "RUSTSEC-2020-0071"
package = "time"
date = "2020-11-18"
url = "https://github.com/time-rs/time/issues/293"
[affected]
os = ["linux", "macos"]
[versions]
patched = [">= 0.2.23"]
```
# Potential segfault in the time crate
The affected functions mutate environment variables without synchronization.
'''


class AdvisoryGateTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if not AUDITOR:
            raise RuntimeError("Install cargo-audit 0.22.2 and set SAPID_CARGO_AUDIT for these regressions")

    def test_real_vulnerable_lock_fails_all_three_scans_and_clean_lock_passes(self):
        parent = Path("D:/BuildCache/sapid/tmp")
        parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="advisory-gate-", dir=parent) as temporary:
            directory = Path(temporary)
            project, database = directory / "project", directory / "database"
            for relative, name in [(".", "audit-root"), ("examples/web", "audit-web"),
                                   ("examples/native", "audit-native")]:
                package = project / relative
                package.mkdir(parents=True, exist_ok=True)
                (package / "Cargo.toml").write_text(
                    f'[package]\nname="{name}"\nversion="0.0.0"\nedition="2024"\n'
                    '[workspace]\n[lib]\npath="lib.rs"\n', encoding="utf-8")
                (package / "lib.rs").write_text("pub fn ready() {}\n", encoding="utf-8")
                (package / "Cargo.lock").write_text(
                    f'version = 4\n[[package]]\nname="{name}"\nversion="0.0.0"\n'
                    '[[package]]\nname="time"\nversion="0.1.44"\n'
                    'source="registry+https://github.com/rust-lang/crates.io-index"\n', encoding="utf-8")
            advisory = database / "crates/time/RUSTSEC-2020-0071.md"
            advisory.parent.mkdir(parents=True)
            advisory.write_text(ADVISORY, encoding="utf-8")
            for arguments in [("init",), ("add", "."),
                              ("-c", "user.name=Audit fixture", "-c", "user.email=fixture@example.invalid",
                               "commit", "-m", "Current local advisory database")]:
                subprocess.run(["git", *arguments], cwd=database, check=True, capture_output=True)
            output = directory / "results"
            env = {**os.environ, "SAPID_CARGO_AUDIT": AUDITOR}

            def audit():
                return subprocess.run(["pwsh", "-NoProfile", "-File", str(ROOT / "scripts/audit-dependencies.ps1"),
                                       "-ProjectRoot", str(project), "-BuildRoot", str(output),
                                       "-Database", str(database), "-NoFetch"], env=env, text=True, capture_output=True)

            failed = audit()
            self.assertNotEqual(failed.returncode, 0, failed.stdout + failed.stderr)
            report = json.loads((output / "results.json").read_text(encoding="utf-8-sig"))
            self.assertFalse(report["complete"])
            self.assertEqual(len(report["locks"]), 3)
            for entry in report["locks"]:
                self.assertTrue(entry["vulnerabilities"], (entry, failed.stdout, failed.stderr))
                self.assertEqual(entry["vulnerabilities"][0]["advisory"]["id"], "RUSTSEC-2020-0071", entry)
                self.assertEqual(entry["status"], "failed")
            for lock in project.rglob("Cargo.lock"):
                lock.write_text(lock.read_text(encoding="utf-8").split('[[package]]\nname="time"')[0], encoding="utf-8")
            passed = audit()
            self.assertEqual(passed.returncode, 0, passed.stdout + passed.stderr)
            report = json.loads((output / "results.json").read_text(encoding="utf-8-sig"))
            self.assertTrue(report["complete"])
            self.assertTrue(all(entry["status"] == "passed" for entry in report["locks"]))

            # Informational maintenance notices remain visible without becoming
            # vulnerability failures, matching cargo-audit's default policy.
            notice = database / "crates/ttf-parser/RUSTSEC-2026-0192.md"
            notice.parent.mkdir(parents=True)
            notice.write_text('''```toml
[advisory]
id = "RUSTSEC-2026-0192"
package = "ttf-parser"
date = "2026-06-28"
informational = "unmaintained"
[versions]
patched = []
```
# `ttf-parser` is unmaintained
See https://github.com/harfbuzz/ttf-parser/issues/217.
''', encoding="utf-8")
            subprocess.run(["git", "add", "."], cwd=database, check=True, capture_output=True)
            subprocess.run(["git", "-c", "user.name=Audit fixture", "-c", "user.email=fixture@example.invalid",
                            "commit", "-m", "Add maintenance notice"], cwd=database, check=True, capture_output=True)
            for lock in project.rglob("Cargo.lock"):
                with lock.open("a", encoding="utf-8") as stream:
                    stream.write('[[package]]\nname="ttf-parser"\nversion="0.25.1"\n'
                                 'source="registry+https://github.com/rust-lang/crates.io-index"\n')
            warned = audit()
            self.assertEqual(warned.returncode, 0, warned.stdout + warned.stderr)
            self.assertIn("RUSTSEC-2026-0192", warned.stdout)
            report = json.loads((output / "results.json").read_text(encoding="utf-8-sig"))
            self.assertTrue(report["complete"])
            self.assertTrue(all(len(entry["warnings"]) == 1 for entry in report["locks"]))

            # A local config must not silently remove an advisory from this gate.
            (project / ".cargo").mkdir()
            (project / ".cargo/audit.toml").write_text(
                '[advisories]\nignore=["RUSTSEC-2026-0192"]\n', encoding="utf-8")
            ignored = audit()
            self.assertNotEqual(ignored.returncode, 0)
            self.assertIn("filters are not permitted", ignored.stderr)


if __name__ == "__main__":
    unittest.main()
