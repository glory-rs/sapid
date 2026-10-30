"""Exercise the real formatter across isolated workspaces and a sibling dependency."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]


class WorkspaceFormattingTests(unittest.TestCase):
    def test_native_drift_is_detected_without_formatting_path_dependencies(self):
        parent = Path(os.environ.get("TEMP", "D:/BuildCache/sapid/tmp"))
        parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="format-regression-", dir=parent) as directory:
            root = Path(directory) / "sapid"
            (root / "scripts").mkdir(parents=True)
            shutil.copyfile(ROOT / "scripts/format.ps1", root / "scripts/format.ps1")
            shutil.copyfile(ROOT / "rustfmt.toml", root / "rustfmt.toml")
            for relative, name in [(".", "sapid"), ("examples/web", "sapid-web-example"),
                                   ("examples/native", "sapid-native-acceptance"), ("../sibling", "sibling")]:
                package = root / relative
                (package / "src").mkdir(parents=True)
                dependency = '\n[dependencies]\nsibling = { path = "../sibling" }\n' if relative == "." else ""
                (package / "Cargo.toml").write_text(
                    f'[package]\nname="{name}"\nversion="0.1.0"\nedition="2024"\n[workspace]\n' + dependency,
                    encoding="utf-8")
                (package / "src/lib.rs").write_text("pub fn ready() {}\n", encoding="utf-8", newline="\n")
            native = root / "examples/native/src/lib.rs"
            sibling = root.parent / "sibling/src/lib.rs"
            drift = "pub fn drift( ) { }\n"
            native.write_text(drift, encoding="utf-8", newline="\n")
            sibling.write_text(drift, encoding="utf-8", newline="\n")

            def formatter(*arguments):
                return subprocess.run(["pwsh", "-NoProfile", "-File", str(root / "scripts/format.ps1"), *arguments],
                                      text=True, capture_output=True, cwd=directory)

            failed = formatter("-Check")
            self.assertNotEqual(failed.returncode, 0, failed.stdout + failed.stderr)
            self.assertIn("examples/native/Cargo.toml", failed.stdout + failed.stderr)
            formatted = formatter()
            self.assertEqual(formatted.returncode, 0, formatted.stdout + formatted.stderr)
            checked = formatter("-Check")
            self.assertEqual(checked.returncode, 0, checked.stdout + checked.stderr)
            self.assertNotEqual(native.read_text(encoding="utf-8"), drift)
            self.assertEqual(sibling.read_text(encoding="utf-8"), drift)


if __name__ == "__main__":
    unittest.main()
