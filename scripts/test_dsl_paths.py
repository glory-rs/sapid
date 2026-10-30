"""Run the DSL gate's real path preparation and a real Windows Rust link smoke."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
PWSH = shutil.which("pwsh")
RUSTC = shutil.which("rustc")
PREPARE = r"""
$ErrorActionPreference = 'Stop'
$tokens = $null; $errors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile($env:DSL_TEST_SCRIPT, [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw ($errors | Out-String) }
$function = $ast.Find({param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Set-DslBuildPaths'}, $true)
if (-not $function) { throw 'Missing production path preparation function' }
Invoke-Expression $function.Extent.Text
$taskBuild = $env:DSL_TEST_BUILD_ROOT
if ($env:DSL_TEST_NATIVE -eq '1') {
    $branches = @($ast.FindAll({param($node) $node -is [Management.Automation.Language.IfStatementAst] -and $node.Clauses[0].Item1.Extent.Text -eq '$Native'}, $true))
    $branch = $branches[-1]
    $commands = @($branch.Clauses[0].Item2.FindAll({param($node) $node -is [Management.Automation.Language.CommandAst]}, $true))
    $prepare = $commands | Where-Object { $_.GetCommandName() -eq 'Set-DslBuildPaths' -and $_.Extent.Text -match '-Native' } | Select-Object -First 1
    $cargo = $commands | Where-Object { $_.GetCommandName() -in @('Confirm-Target','Invoke-Cargo','cargo') } | Select-Object -First 1
    if (-not $prepare -or -not $cargo -or $prepare.Extent.StartOffset -gt $cargo.Extent.StartOffset) {
        throw 'Native paths must be prepared before the first Cargo command'
    }
    # Execute the actual native-stage preparation command, not a fake Cargo driver.
    Invoke-Expression $prepare.Extent.Text
} else { Set-DslBuildPaths -Root $taskBuild }
@{ target = $env:CARGO_TARGET_DIR; temp = $env:TEMP; tmp = $env:TMP;
   site = $env:SAPID_DSL_SITE; cargo = $env:CARGO_HOME } | ConvertTo-Json -Compress
"""


@unittest.skipUnless(os.name == "nt" and PWSH and RUSTC, "Windows, PowerShell 7 and Rust are required")
class DslBuildPathTests(unittest.TestCase):
    def setUp(self):
        parent = Path(os.environ.get("BUILD_ROOT", "D:/BuildCache/sapid/fixes/gates")) / "dsl-path-tests"
        parent.mkdir(parents=True, exist_ok=True)
        self.directory = tempfile.TemporaryDirectory(prefix="standalone-", dir=parent)
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name).resolve()
        self.build = self.root / "fresh-build"
        self.env = os.environ.copy()
        self.env.update(DSL_TEST_SCRIPT=str(ROOT / "scripts/check-dsl.ps1"),
                        DSL_TEST_BUILD_ROOT=str(self.build),
                        CARGO_HOME=os.environ.get("CARGO_HOME") or "D:/BuildCache/glory-semantic-ui/cargo")

    def prepare(self, native=False, **overrides):
        env = self.env | {"DSL_TEST_NATIVE": "1" if native else "0"} | overrides
        result = subprocess.run([PWSH, "-NoProfile", "-Command", PREPARE], env=env,
                                capture_output=True, text=True, timeout=30)
        return result

    def test_fresh_native_stage_creates_temp_before_a_real_rust_link(self):
        source = self.root / "smoke.rs"
        source.write_text('fn main() { println!("DSL native link smoke"); }\n', encoding="utf-8")
        missing = self.root / "absent-temp"
        bad = subprocess.run([RUSTC, "--edition=2024", str(source), "-o", str(self.root / "bad.exe")],
                             env=self.env | {"TEMP": str(missing), "TMP": str(missing)},
                             capture_output=True, text=True, timeout=60)
        self.assertNotEqual(bad.returncode, 0, "The negative smoke must exercise a genuinely missing compiler TEMP")
        self.assertIn("temp", bad.stderr.lower(), bad.stderr)
        self.assertFalse(self.build.exists())
        prepared = self.prepare(native=True)
        self.assertEqual(prepared.returncode, 0, prepared.stdout + prepared.stderr)
        paths = json.loads(prepared.stdout)
        self.assertEqual(Path(paths["target"]), self.build / "native-target")
        self.assertEqual(Path(paths["temp"]), self.build / "tmp")
        self.assertEqual(paths["temp"], paths["tmp"])
        self.assertEqual(paths["cargo"], self.env["CARGO_HOME"])
        self.assertTrue(Path(paths["target"]).is_dir())
        self.assertTrue(Path(paths["temp"]).is_dir())
        binary = Path(paths["target"]) / "smoke.exe"
        linked = subprocess.run([RUSTC, "--edition=2024", str(source), "-o", str(binary)],
                                env=self.env | {"TEMP": paths["temp"], "TMP": paths["tmp"]},
                                capture_output=True, text=True, timeout=60)
        self.assertEqual(linked.returncode, 0, linked.stdout + linked.stderr)
        executed = subprocess.run([str(binary)], capture_output=True, text=True, timeout=10)
        self.assertEqual(executed.returncode, 0, executed.stdout + executed.stderr)
        self.assertIn("DSL native link smoke", executed.stdout)

    def test_portable_and_native_paths_share_only_the_requested_build_root(self):
        portable = self.prepare()
        self.assertEqual(portable.returncode, 0, portable.stdout + portable.stderr)
        paths = json.loads(portable.stdout)
        self.assertEqual(Path(paths["target"]), self.build / "target")
        self.assertEqual(Path(paths["site"]), self.build / "dsl")
        self.assertTrue(Path(paths["temp"]).is_dir())
        native = self.prepare(native=True)
        self.assertEqual(native.returncode, 0, native.stdout + native.stderr)
        paths = json.loads(native.stdout)
        self.assertEqual(Path(paths["target"]), self.build / "native-target")
        self.assertEqual(Path(paths["temp"]), self.build / "tmp")

    def test_foreign_build_root_is_rejected_before_any_directory_is_created(self):
        result = self.prepare(native=True, DSL_TEST_BUILD_ROOT="C:/outside-dsl-build")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("BuildRoot must be inside D:/BuildCache/sapid", result.stderr)
        self.assertFalse(self.build.exists())

    def test_foreign_cargo_home_is_rejected_before_any_directory_is_created(self):
        result = self.prepare(native=True, CARGO_HOME="C:/outside-cargo-home")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("CARGO_HOME must be on D drive", result.stderr)
        self.assertFalse(self.build.exists())


if __name__ == "__main__":
    unittest.main()
