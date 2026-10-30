"""Publication retry tests; no network, credentials or registry mutations."""
import contextlib
import importlib.util
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import tarfile
from types import SimpleNamespace
import unittest
from unittest.mock import MagicMock, patch

spec = importlib.util.spec_from_file_location('publisher', Path(__file__).with_name('release.py'))
publisher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(publisher)


class PublicationRetryTests(unittest.TestCase):
    def test_success_does_not_contact_registry(self):
        with patch.object(publisher.subprocess, 'run', return_value=SimpleNamespace(returncode=0, stdout='')), \
             patch.object(publisher.urllib.request, 'urlopen') as network:
            publisher.publish(['cargo', 'publish'])
            network.assert_not_called()

    def test_compiler_or_auth_failure_is_not_retried(self):
        for diagnostic in ('compiler error', '403 Forbidden', '429 without a retry time'):
            with self.subTest(diagnostic=diagnostic), \
                 patch.object(publisher.subprocess, 'run', return_value=SimpleNamespace(returncode=101, stdout=diagnostic)) as command, \
                 contextlib.redirect_stdout(io.StringIO()):
                with self.assertRaises(subprocess.CalledProcessError):
                    publisher.publish(['cargo', 'publish'])
                self.assertEqual(command.call_count, 1)

    def test_only_explicit_rate_limit_waits_until_registry_time(self):
        limited = SimpleNamespace(returncode=101, stdout='429 Too Many Requests: try again after Thu, 29 Oct 2026 07:50:05 GMT')
        response = MagicMock()
        response.__enter__.return_value.headers = {'Date': 'Thu, 29 Oct 2026 07:50:00 GMT'}
        with patch.object(publisher.subprocess, 'run', side_effect=[limited, SimpleNamespace(returncode=0, stdout='')]) as command, \
             patch.object(publisher.urllib.request, 'urlopen', return_value=response), \
             patch.object(publisher.time, 'monotonic', side_effect=[0, 1, 2, 11]), \
             patch.object(publisher.time, 'sleep') as sleep, \
             contextlib.redirect_stdout(io.StringIO()):
            publisher.publish(['cargo', 'publish'])
            self.assertEqual(command.call_count, 2)
            sleep.assert_called_once_with(8)


class ReleasePlanTests(unittest.TestCase):
    def validate(self, mutate=lambda plan, packages: None, tag=None):
        parent=Path(os.environ.get('CARGO_TARGET_DIR','D:/BuildCache/release-tests/target')).parent
        parent.mkdir(parents=True,exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='publisher-test-',dir=parent) as directory:
            root=Path(directory)
            target=root/'target'
            (root/'Cargo.toml').write_text('[workspace.package]\nversion="0.6.0"\n[package]\nversion="0.0.1"\n',encoding='utf-8')
            plan={'repository':'example/owned','allowed_prefixes':['owned'],'tag_prefix':'v','uniform_version':True,
                  'packages':['owned-base','owned-app']}
            packages=[{'name':name,'version':'0.6.0','repository':'https://github.com/example/owned','dependencies':[],
                       'manifest_path':str(root/'Cargo.toml')} for name in plan['packages']]
            mutate(plan,packages)
            (root/'release.json').write_text(json.dumps(plan),encoding='utf-8')
            metadata=json.dumps({'target_directory':str(target),'packages':packages})
            arguments=['release.py']+(['--tag',tag] if tag else [])
            with patch.object(publisher,'ROOT',root),patch.object(publisher,'run',return_value=metadata), \
                 patch.object(publisher.sys,'argv',arguments),patch.dict(os.environ,{'CARGO_TARGET_DIR':str(target),'GITHUB_REF_TYPE':'branch','GITHUB_REF_NAME':'main'}), \
                 contextlib.redirect_stdout(io.StringIO()) as output:
                publisher.main()
                return output.getvalue()

    def test_workspace_version_precedes_private_example_version(self):
        self.assertIn('"tag": "v0.6.0"',self.validate())

    def test_wrong_tag_is_rejected(self):
        with self.assertRaises(SystemExit):
            self.validate(tag='v0.0.1')

    def test_wrong_repository_namespace_version_and_disabled_package_are_rejected(self):
        mutations=[lambda p,v:v[0].update(repository='https://github.com/another/owner'),
                   lambda p,v:p.update(allowed_prefixes=['other']),
                   lambda p,v:v[0].update(version='0.3.1'),
                   lambda p,v:v[0].update(publish=[])]
        for mutation in mutations:
            with self.subTest(mutation=mutation),self.assertRaises(SystemExit):
                self.validate(mutation)

    def test_git_external_path_and_reversed_order_are_rejected(self):
        dependency={'name':'foreign','req':'*','kind':None,'source':None,'path':'D:/unpublished'}
        def git(plan,packages):
            packages[1]['dependencies']=[dict(dependency,source='git+https://github.com/example/foreign')]
        def path(plan,packages):
            packages[1]['dependencies']=[dependency]
        def order(plan,packages):
            plan['packages'].reverse()
            packages[1]['dependencies']=[dict(dependency,name='owned-base',req='=0.6.0')]
        for mutation in (git,path,order):
            with self.subTest(mutation=mutation),self.assertRaises(SystemExit):
                self.validate(mutation)

    def test_reviewed_exact_external_path_is_allowed(self):
        def mutation(plan,packages):
            plan['external_path_dependencies']={'foreign':'=0.6.0'}
            packages[1]['dependencies']=[{'name':'foreign','req':'=0.6.0','kind':None,'source':None,'path':'D:/published'}]
        self.assertIn('v0.6.0',self.validate(mutation))


class ArchiveTests(unittest.TestCase):
    def inspect(self, license_content, sha='a'*40, dirty=False):
        parent=Path(os.environ.get('CARGO_TARGET_DIR','D:/BuildCache/release-tests/target')).parent
        parent.mkdir(parents=True,exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='archive-test-',dir=parent) as directory:
            archive=Path(directory)/'test.crate'
            with tarfile.open(archive,'w:gz') as output:
                for name,data in [('pkg/.cargo_vcs_info.json',json.dumps({'git':{'sha1':sha,'dirty':dirty}}).encode()),
                                  ('pkg/LICENSE',license_content)]:
                    entry=tarfile.TarInfo(name);entry.size=len(data)
                    output.addfile(entry,io.BytesIO(data))
            return publisher.validate_archive(archive,'a'*40)

    def test_full_license_is_accepted(self):
        self.assertEqual(self.inspect(b'Apache License\nVersion 2.0\n')['git']['sha1'],'a'*40)

    def test_windows_link_stub_is_rejected_before_upload(self):
        with self.assertRaisesRegex(SystemExit,'unresolved link'):
            self.inspect(b'../LICENSE')

    def test_wrong_or_dirty_source_is_rejected(self):
        for arguments in ({'sha':'b'*40},{'dirty':True}):
            with self.subTest(arguments=arguments),self.assertRaises(SystemExit):
                self.inspect(b'Apache License',**arguments)


class ReleaseProvenanceTests(unittest.TestCase):
    """Exercise real local Git/tag transitions; registry access is forbidden."""

    def setUp(self):
        parent = Path(os.environ.get('CARGO_TARGET_DIR', 'D:/BuildCache/sapid/fixes/release/target')).parent.resolve()
        parent.mkdir(parents=True, exist_ok=True)
        self.directory = tempfile.TemporaryDirectory(prefix='provenance-', dir=parent)
        self.base = Path(self.directory.name).resolve()
        self.assertTrue(self.base.is_relative_to(parent))
        self.addCleanup(self.directory.cleanup)
        self.root = self.base / 'product'
        self.glory = self.base / 'glory'
        self.origin = self.base / 'origin.git'
        self.root.mkdir()
        self.glory.mkdir()
        self.git(self.origin, 'init', '--bare', str(self.origin), cwd=self.base)
        self.git(self.glory, 'init')
        (self.glory / 'source').write_text('validated Glory', encoding='utf-8')
        self.commit(self.glory)
        self.glory_commit = self.git(self.glory, 'rev-parse', 'HEAD')
        self.git(self.root, 'init')
        (self.root / 'Cargo.toml').write_text('[package]\nname="owned"\nversion="0.6.0"\n', encoding='utf-8')
        self.plan = {'repository': 'example/owned'}
        (self.root / 'release.json').write_text(json.dumps(self.plan), encoding='utf-8')
        (self.root / 'deploy').mkdir()
        self.pin = {'repository': 'example/glory', 'revision': self.glory_commit}
        (self.root / 'deploy/dependencies.json').write_text(json.dumps({'glory': self.pin}), encoding='utf-8')
        (self.root / 'releases').mkdir()
        (self.root / 'releases/0.6.0.md').write_text('Reviewed release', encoding='utf-8')
        self.commit(self.root)
        self.product_commit = self.git(self.root, 'rev-parse', 'HEAD')
        self.git(self.root, 'tag', 'v0.6.0')
        self.git(self.root, 'remote', 'add', 'origin', str(self.origin))
        self.git(self.root, 'push', 'origin', 'refs/tags/v0.6.0')
        self.provenance = self.base / 'browser-provenance.json'
        self.harness = self.base / 'harness'
        (self.harness / 'scripts').mkdir(parents=True)
        (self.harness / 'examples/web').mkdir(parents=True)
        runner = b"const suites = ['acceptance.js'];\n"
        source = b'(async page => ({reports: [], errors: []}))'
        (self.harness / 'scripts/browser.cjs').write_bytes(runner)
        (self.harness / 'examples/web/acceptance.js').write_bytes(source)
        self.git(self.harness, 'init')
        self.commit(self.harness)
        self.acceptance_commit = self.git(self.harness, 'rev-parse', 'HEAD')
        self.fixtures = self.base / 'fixtures'
        self.fixtures.mkdir()
        (self.fixtures / 'product.js').write_bytes(b'Actual built product fixture')
        self.results = self.provenance.with_name('browser-results.json')
        self.report = {'complete': True, 'errors': [], 'completedSuites': ['acceptance.js'],
                       'sourceHashes': {'acceptance.js': hashlib.sha256(source).hexdigest()},
                       'artifactHashes': {'product.js': hashlib.sha256((self.fixtures / 'product.js').read_bytes()).hexdigest()}}
        self.write_report()
        self.evidence = {
            'schema': 1, 'repository': 'example/owned', 'workflow_run_id': '123',
            'workflow_run_attempt': '2', 'product_commit': self.product_commit,
            'acceptance_commit': self.acceptance_commit, 'glory_repository': 'example/glory',
            'glory_commit': self.glory_commit,
            'browser_results_sha256': hashlib.sha256(self.results.read_bytes()).hexdigest(),
            'browser_runner_sha256': hashlib.sha256(runner).hexdigest(),
        }
        self.write_evidence()
        self.environment = {
            'GITHUB_ACTIONS': 'true', 'GITHUB_REPOSITORY': 'example/owned',
            'GITHUB_RUN_ID': '123', 'GITHUB_RUN_ATTEMPT': '2',
            'ACCEPTANCE_ARTIFACT': 'sapid-browser-acceptance-2',
            'ACCEPTANCE_PROVENANCE_SHA256': hashlib.sha256(self.provenance.read_bytes()).hexdigest(),
        }
        self.options = SimpleNamespace(provenance=self.provenance, expected_commit=self.product_commit,
                                       acceptance_commit=self.acceptance_commit, harness_root=self.harness,
                                       fixtures_root=self.fixtures)

    @staticmethod
    def git(root, *args, cwd=None):
        command = ['git', '-c', 'user.name=Release test', '-c', 'user.email=release-test@example.invalid']
        if cwd is None:
            command += ['-C', str(root)]
        result = subprocess.run(command + list(args), cwd=cwd, check=True, text=True,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        return result.stdout.strip()

    def commit(self, root):
        self.git(root, 'add', '.')
        self.git(root, 'commit', '-m', 'Fixture source')

    def write_evidence(self):
        self.provenance.write_text(json.dumps(self.evidence), encoding='utf-8')
        if hasattr(self, 'environment'):
            self.environment['ACCEPTANCE_PROVENANCE_SHA256'] = hashlib.sha256(self.provenance.read_bytes()).hexdigest()

    def write_report(self):
        self.results.write_text(json.dumps(self.report), encoding='utf-8')

    def gate(self):
        with patch.object(publisher, 'ROOT', self.root), patch.dict(os.environ, self.environment), \
             patch.object(publisher.urllib.request, 'urlopen') as registry, \
             patch.object(publisher, 'publish') as upload:
            publisher.validate_release_gate(self.options, self.plan, 'v0.6.0',
                                            self.git(self.root, 'rev-parse', 'HEAD'))
            registry.assert_not_called()
            upload.assert_not_called()

    def assert_rejected_before_registry(self, diagnostic):
        original_run = publisher.run

        def checked_run(*args, **kwargs):
            if args[0] == 'cargo':
                self.fail('Invalid provenance reached Cargo before the release gate')
            return original_run(*args, **kwargs)

        arguments = ['release.py', '--product-root', str(self.root), '--publish', '--tag', 'v0.6.0',
                     '--expected-commit', self.product_commit, '--acceptance-commit', self.acceptance_commit,
                     '--harness-root', str(self.harness),
                     '--provenance', str(self.provenance)]
        with patch.object(publisher, 'ROOT', self.root), patch.dict(os.environ, self.environment), \
             patch.object(publisher.sys, 'argv', arguments), patch.object(publisher, 'run', side_effect=checked_run), \
             patch.object(publisher.urllib.request, 'urlopen') as registry, \
             patch.object(publisher, 'publish') as upload:
            with self.assertRaisesRegex(SystemExit, diagnostic):
                publisher.main()
            registry.assert_not_called()
            upload.assert_not_called()

    def test_matching_evidence_accepts_lightweight_and_annotated_tags(self):
        self.gate()
        self.git(self.root, 'tag', '-f', '-a', 'v0.6.0', '-m', 'Annotated release')
        self.git(self.root, 'push', '--force', 'origin', 'refs/tags/v0.6.0')
        self.gate()

    def test_other_product_glory_harness_repository_run_or_attempt_is_rejected(self):
        for field in ('schema', 'repository', 'workflow_run_id',
                      'product_commit', 'acceptance_commit', 'glory_repository', 'glory_commit'):
            with self.subTest(field=field):
                original = self.evidence[field]
                self.evidence[field] = 2 if field == 'schema' else 'different'
                self.write_evidence()
                self.assert_rejected_before_registry('Acceptance provenance differs')
                self.evidence[field] = original

    def test_previous_successful_attempt_can_be_reused_only_by_its_exact_artifact(self):
        self.evidence['workflow_run_attempt'] = '1'
        self.write_evidence()
        self.assert_rejected_before_registry('exact artifact')
        self.environment['ACCEPTANCE_ARTIFACT'] = 'sapid-browser-acceptance-1'
        self.gate()
        for attempt in ('0', '3', 'invalid'):
            self.evidence['workflow_run_attempt'] = attempt
            self.write_evidence()
            self.assert_rejected_before_registry('invalid workflow attempt')

    def test_partial_failed_missing_suite_or_wrong_source_reports_are_rejected(self):
        mutations = [dict(complete=False), dict(errors=['one suite failed']),
                     dict(completedSuites=[]), dict(completedSuites=['acceptance.js', 'acceptance.js']),
                     dict(sourceHashes={'acceptance.js': 'f' * 64}), dict(artifactHashes={})]
        original = dict(self.report)
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                self.report = {**original, **mutation}
                self.write_report()
                self.evidence['browser_results_sha256'] = hashlib.sha256(self.results.read_bytes()).hexdigest()
                self.write_evidence()
                self.assert_rejected_before_registry('Invalid browser acceptance report')

    def test_results_or_runner_digest_changes_are_rejected(self):
        self.results.write_bytes(self.results.read_bytes() + b'\n')
        self.assert_rejected_before_registry('digest differs')
        self.write_report()
        runner = self.harness / 'scripts/browser.cjs'
        runner.write_bytes(runner.read_bytes() + b'// changed runner\n')
        self.assert_rejected_before_registry('digest differs')

    def test_provenance_cannot_be_replaced_without_the_successful_job_output_digest(self):
        self.provenance.write_bytes(self.provenance.read_bytes() + b'\n')
        self.assert_rejected_before_registry('provenance digest differs')

    def test_recording_provenance_checks_the_actual_product_fixtures(self):
        with patch.object(publisher, 'ROOT', self.root), patch.dict(os.environ, self.environment), \
             patch.object(publisher.urllib.request, 'urlopen') as registry:
            publisher.record_acceptance(self.options)
            self.assertEqual(json.loads(self.provenance.read_text())['product_commit'], self.product_commit)
            (self.fixtures / 'product.js').write_bytes(b'Different built product')
            self.provenance.unlink()
            with self.assertRaisesRegex(SystemExit, 'fixtures differ'):
                publisher.record_acceptance(self.options)
            self.assertFalse(self.provenance.exists())
            registry.assert_not_called()

    def test_missing_or_malformed_provenance_is_rejected(self):
        self.provenance.unlink()
        self.assert_rejected_before_registry('Missing or invalid')
        self.provenance.write_text('{broken', encoding='utf-8')
        self.environment['ACCEPTANCE_PROVENANCE_SHA256'] = hashlib.sha256(self.provenance.read_bytes()).hexdigest()
        self.assert_rejected_before_registry('Missing or invalid')

    def test_glory_checkout_drift_is_rejected(self):
        (self.glory / 'source').write_text('Different Glory', encoding='utf-8')
        self.commit(self.glory)
        self.assert_rejected_before_registry('Glory checkout differs')

    def test_moved_tag_and_new_checkout_cannot_reuse_old_successful_checks(self):
        (self.root / 'new-source').write_text('Unchecked source B', encoding='utf-8')
        self.commit(self.root)
        self.git(self.root, 'tag', '-f', 'v0.6.0')
        self.git(self.root, 'push', '--force', 'origin', 'refs/tags/v0.6.0')
        # The old implementation accepted this HEAD == tag despite checks
        # having run at A. The frozen product commit must still reject B.
        self.assert_rejected_before_registry('Publisher HEAD differs')

    def test_remote_tag_drift_is_rejected_even_with_frozen_local_checkout(self):
        (self.root / 'new-source').write_text('Unchecked source B', encoding='utf-8')
        self.commit(self.root)
        self.git(self.root, 'tag', '-f', 'v0.6.0')
        self.git(self.root, 'push', '--force', 'origin', 'refs/tags/v0.6.0')
        self.git(self.root, 'reset', '--hard', self.product_commit)
        self.git(self.root, 'tag', '-f', 'v0.6.0', self.product_commit)
        self.assert_rejected_before_registry('Release tag moved')

    def test_missing_gate_inputs_or_wrong_repository_cannot_publish(self):
        self.options.expected_commit = None
        with self.assertRaisesRegex(SystemExit, 'Publication requires frozen'):
            self.gate()
        self.environment['GITHUB_REPOSITORY'] = 'different/repository'
        self.assert_rejected_before_registry('gated Actions release workflow')


if __name__ == '__main__':
    unittest.main()
