const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { serve, runAcceptance } = require('./showcase-acceptance.cjs');
const temporary = 'D:/BuildCache/sapid/tmp';
fs.mkdirSync(temporary, { recursive: true });

function fixture(t) {
  const root = fs.mkdtempSync(path.join(temporary, 'showcase-runner-'));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const directory = path.join(root, 'site'); fs.mkdirSync(directory);
  for (const name of ['index.html', 'showcase.html']) fs.writeFileSync(path.join(directory, name), '<!doctype html><title>Fixture</title>');
  fs.writeFileSync(path.join(directory, 'asset-manifest.json'), '{}');
  return { root, directory, output: path.join(root, 'results') };
}
function suite(root, name, source, report) {
  const file = path.join(root, `${name}.cjs`); fs.writeFileSync(file, source);
  return { id: name, source: file, program: process.execPath, args: [file], report };
}

test('a real failed HTTP assertion stops later suites and records the full pending manifest', async t => {
  const options = fixture(t), marker = path.join(options.root, 'unexpected');
  options.suites = [suite(options.root, 'missing-asset', `
    const assert = require('node:assert/strict');
    fetch(process.env.SAPID_SHOWCASE_BASE + '/missing.wasm').then(response => assert.equal(response.status, 200));
  `), suite(options.root, 'must-not-run', `require('node:fs').writeFileSync(${JSON.stringify(marker)}, 'ran');`)];
  const result = await runAcceptance(options);
  assert.equal(result.complete, false);
  assert.deepEqual(result.suites.map(entry => entry.status), ['failed', 'pending']);
  assert.deepEqual(result.suiteManifest, ['missing-asset', 'must-not-run']);
  assert.match(result.errors.join('\n'), /404/);
  assert.equal(fs.existsSync(marker), false);
});

test('JSON-reported browser errors fail even if a legacy driver exits zero; stale success is removed', async t => {
  const options = fixture(t), report = path.join(options.root, 'driver.json');
  fs.writeFileSync(report, JSON.stringify({ errors: [] }));
  options.suites = [suite(options.root, 'caught-assertion', `
    const fs = require('node:fs'), assert = require('node:assert/strict');
    const errors = []; try { assert.equal(1, 2, 'actual failed assertion'); } catch (error) { errors.push(error.message); }
    fs.writeFileSync(${JSON.stringify(report)}, JSON.stringify({ errors }));
  `, report)];
  const result = await runAcceptance(options);
  assert.equal(result.complete, false);
  assert.match(result.errors.join('\n'), /actual failed assertion/);
  options.suites = [suite(options.root, 'no-report', '', report)];
  const absent = await runAcceptance(options);
  assert.equal(absent.complete, false);
  assert.match(absent.errors.join('\n'), /ENOENT/);
});

test('a hung child is bounded and cannot produce passing evidence', async t => {
  const options = fixture(t);
  options.timeout = 200;
  options.suites = [suite(options.root, 'hung-driver', 'setInterval(() => {}, 1000);')];
  const result = await runAcceptance(options);
  assert.equal(result.complete, false);
  assert.equal(result.suites[0].status, 'failed');
});

test('the in-process server rejects traversal and follows no directory junction outside the site', async t => {
  const options = fixture(t), outside = path.join(options.root, 'private');
  fs.mkdirSync(outside); fs.writeFileSync(path.join(outside, 'private.txt'), 'private');
  fs.symlinkSync(outside, path.join(options.directory, 'linked'), process.platform === 'win32' ? 'junction' : 'dir');
  const server = await serve(options.directory);
  try {
    assert.equal((await fetch(`${server.base}/`)).status, 200);
    assert.equal((await fetch(`${server.base}/missing.js`)).status, 404);
    assert.equal((await fetch(`${server.base}/%2e%2e%2fprivate/private.txt`)).status, 403);
    assert.equal((await fetch(`${server.base}/linked/private.txt`)).status, 403);
  } finally { await server.close(); }
});
