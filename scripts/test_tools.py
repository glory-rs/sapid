"""Real file-tampering regressions; no network, credentials or deployments."""
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import unittest

SOURCE = Path(__file__).resolve().parents[1]
PWSH = shutil.which('pwsh')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


@unittest.skipUnless(PWSH, 'PowerShell 7 is required for Windows tool integrity checks')
class ToolIntegrityTests(unittest.TestCase):
    def setUp(self):
        parent = Path(os.environ.get('SAPID_TOOL_TEST_ROOT', os.environ.get('BUILD_ROOT', 'D:/BuildCache/sapid/fixes/infra'))).resolve() / 'tool-tests'
        parent.mkdir(parents=True, exist_ok=True)
        self.directory = tempfile.TemporaryDirectory(prefix='integrity-', dir=parent)
        self.root = Path(self.directory.name).resolve()
        self.assertTrue(self.root.is_relative_to(parent))
        self.addCleanup(self.directory.cleanup)
        self.source = self.root / 'source'
        (self.source / 'scripts').mkdir(parents=True)
        (self.source / 'deploy/tools').mkdir(parents=True)
        for name in ('install-build-tools.ps1', 'deploy-pages.ps1'):
            shutil.copyfile(SOURCE / 'scripts' / name, self.source / 'scripts' / name)
        for name in ('package.json', 'package-lock.json'):
            shutil.copyfile(SOURCE / 'deploy/tools' / name, self.source / 'deploy/tools' / name)
        self.manifest_path = self.source / 'deploy/build-tools.json'
        self.manifest = json.loads((SOURCE / 'deploy/build-tools.json').read_text())
        self.build = self.root / 'build'
        self.tools = self.build / 'tools'
        self.tools.mkdir(parents=True)
        package = self.manifest['wasm_bindgen']['package']
        self.exe = self.tools / package / 'wasm-bindgen.exe'
        self.exe.parent.mkdir()
        self.exe.write_bytes(b'Inert executable fixture. This must never run.')
        self.archive = self.tools / f'{package}.tar.gz'
        with tarfile.open(self.archive, 'w:gz') as output:
            member = tarfile.TarInfo(f'{package}/wasm-bindgen.exe')
            data = self.exe.read_bytes()
            member.size = len(data)
            output.addfile(member, io.BytesIO(data))
        self.manifest['wasm_bindgen']['archive_sha256'] = sha(self.archive)
        self.manifest['wasm_bindgen']['executable_sha256'] = sha(self.exe)
        self.write_manifest()
        self.node_tools = self.build / 'node-tools'
        files = {
            'node_modules/wrangler/package.json': json.dumps({'name': 'wrangler', 'version': '4.152.0'}),
            'node_modules/wrangler/bin/wrangler.js': 'throw Error("Must not execute");',
            'node_modules/playwright/package.json': json.dumps({'name': 'playwright', 'version': '1.64.1'}),
            'node_modules/playwright/cli.js': 'throw Error("Must not execute");',
            'node_modules/dependency/index.js': 'module.exports = "original dependency";',
        }
        for relative, content in files.items():
            path = self.node_tools / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding='utf-8')
        self.receipt = self.node_tools / 'tools-receipt.json'
        inventory = {name: sha(self.node_tools / name) for name in sorted(files)}
        self.receipt.write_text(json.dumps({'schema': 1, 'package_lock_sha256': sha(self.source / 'deploy/tools/package-lock.json'),
                                            'files': inventory}), encoding='utf-8')
        self.receipt_hash = sha(self.receipt)
        self.marker = self.root / 'executions.txt'
        self.wrapper = self.root / 'invoke.ps1'
        self.tmp = self.root / 'tmp'
        self.tmp.mkdir()

    def write_manifest(self):
        self.manifest_path.write_text(json.dumps(self.manifest), encoding='utf-8')

    def invoke(self, deploy=False):
        # Shadow all external entry points. Real hashing and script control flow
        # run unchanged; any accidental network/tool call leaves evidence.
        marker = str(self.marker).replace("'", "''")
        self.wrapper.write_text("""param([string]$Script,[string]$Root,[string]$Digest,[switch]$Deploy)
$ErrorActionPreference = 'Stop'
function global:node { Add-Content -LiteralPath 'MARKER' -Value "node $args"; $global:LASTEXITCODE = 0; 'v24.18.0' }
function global:npm { Add-Content -LiteralPath 'MARKER' -Value 'npm'; throw 'Unexpected npm execution' }
function global:npx { Add-Content -LiteralPath 'MARKER' -Value 'npx'; throw 'Unexpected npx execution' }
function global:python { Add-Content -LiteralPath 'MARKER' -Value 'python'; throw 'Unexpected asset request' }
function global:Invoke-WebRequest { Add-Content -LiteralPath 'MARKER' -Value 'network'; throw 'Unexpected network' }
function global:Invoke-RestMethod { Add-Content -LiteralPath 'MARKER' -Value 'api'; throw 'Unexpected API call' }
if ($Deploy) { & $Script -BuildRoot $Root -ToolsReceiptSha256 $Digest -Directory $Root }
else { & $Script -BuildRoot $Root -ToolsReceiptSha256 $Digest -VerifyOnly | ConvertTo-Json -Compress }
""".replace('MARKER', marker), encoding='utf-8')
        script = self.source / 'scripts' / ('deploy-pages.ps1' if deploy else 'install-build-tools.ps1')
        arguments = [PWSH, '-NoProfile', '-File', str(self.wrapper), '-Script', str(script),
                     '-Root', str(self.build), '-Digest', self.receipt_hash]
        if deploy:
            arguments.append('-Deploy')
        environment = dict(os.environ)
        for name in ('CLOUDFLARE_API_TOKEN', 'CLOUDFLARE_ACCOUNT_ID', 'CARGO_REGISTRY_TOKEN', 'NPM_TOKEN', 'NODE_AUTH_TOKEN'):
            environment.pop(name, None)
        environment.update(TEMP=str(self.tmp), TMP=str(self.tmp))
        return subprocess.run(arguments, check=False, text=True, encoding='utf-8', errors='replace',
                              env=environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)

    def assert_rejected_before_execution(self, diagnostic, deploy=False):
        result = self.invoke(deploy=deploy)
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn(diagnostic, result.stdout)
        self.assertFalse(self.marker.exists(), self.marker.read_text() if self.marker.exists() else result.stdout)

    def test_matching_cache_verifies_without_installing_or_network(self):
        result = self.invoke()
        self.assertEqual(result.returncode, 0, result.stdout)
        self.assertEqual(self.marker.read_text().strip(), 'node --version')
        self.assertEqual(json.loads(result.stdout)['ReceiptSha256'], self.receipt_hash)

    def test_tampered_archive_is_rejected_before_extraction_or_execution(self):
        self.archive.write_bytes(self.archive.read_bytes() + b'tampered')
        self.assert_rejected_before_execution('wasm-bindgen archive SHA-256 mismatch')

    def test_tampered_cached_executable_is_rejected_before_execution(self):
        self.exe.write_bytes(b'tampered executable')
        self.assert_rejected_before_execution('wasm-bindgen executable SHA-256 mismatch')

    def test_wrong_pinned_hash_is_rejected(self):
        self.manifest['wasm_bindgen']['executable_sha256'] = '0' * 64
        self.write_manifest()
        self.assert_rejected_before_execution('wasm-bindgen executable SHA-256 mismatch')

    def test_missing_cached_executable_is_rejected_without_download(self):
        self.exe.unlink()
        self.assert_rejected_before_execution('Missing cached wasm-bindgen executable')

    def test_tampered_transitive_node_module_is_rejected_before_execution(self):
        (self.node_tools / 'node_modules/dependency/index.js').write_text('tampered dependency')
        self.assert_rejected_before_execution('Node tool cache SHA-256 inventory mismatch')

    def test_replaced_receipt_is_rejected(self):
        self.receipt.write_bytes(self.receipt.read_bytes() + b' ')
        self.assert_rejected_before_execution('Node tools receipt SHA-256 mismatch')

    def test_missing_wrangler_fails_deployment_before_asset_or_api_request(self):
        (self.node_tools / 'node_modules/wrangler/bin/wrangler.js').unlink()
        self.assert_rejected_before_execution('Missing local tool entrypoint', deploy=True)

    def test_tampered_cache_fails_deployment_before_asset_or_api_request(self):
        (self.node_tools / 'node_modules/wrangler/bin/wrangler.js').write_text('tampered CLI')
        self.assert_rejected_before_execution('Node tool cache SHA-256 inventory mismatch', deploy=True)

    def test_valid_tools_without_credentials_do_not_contact_cloudflare(self):
        result = self.invoke(deploy=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('Set CLOUDFLARE_API_TOKEN', result.stdout)
        self.assertEqual(self.marker.read_text().strip(), 'node --version')


if __name__ == '__main__':
    unittest.main()
