// One fail-fast entry point for the complete showcase acceptance contract.
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const crypto = require('node:crypto');
const { spawn, spawnSync } = require('node:child_process');
const artifactRoot = 'D:/BuildCache/sapid';
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const mime = { '.html': 'text/html; charset=utf-8', '.js': 'application/javascript',
  '.wasm': 'application/wasm', '.css': 'text/css', '.svg': 'image/svg+xml', '.png': 'image/png',
  '.json': 'application/json', '.ico': 'image/x-icon', '.woff2': 'font/woff2' };

function artifactPath(value) {
  const resolved = path.resolve(value);
  const relative = path.relative(path.resolve(artifactRoot), resolved);
  if (relative.startsWith('..') || path.isAbsolute(relative)) throw Error(`Artifacts must stay under ${artifactRoot}: ${resolved}`);
  return resolved;
}

async function serve(directory) {
  const root = await fs.promises.realpath(directory);
  const server = http.createServer({ maxHeaderSize: 16384 }, async (request, response) => {
    try {
      if (!['GET', 'HEAD'].includes(request.method)) { response.writeHead(405).end(); return; }
      let name;
      try { name = decodeURIComponent(new URL(request.url, 'http://localhost').pathname); }
      catch { response.writeHead(400).end(); return; }
      const file = path.resolve(root, `.${name === '/' ? '/index.html' : name}`);
      const relative = path.relative(root, file);
      if (relative.startsWith('..') || path.isAbsolute(relative)) { response.writeHead(403).end(); return; }
      const actual = await fs.promises.realpath(file);
      const actualRelative = path.relative(root, actual);
      if (actualRelative.startsWith('..') || path.isAbsolute(actualRelative)) { response.writeHead(403).end(); return; }
      const stat = await fs.promises.stat(actual);
      if (!stat.isFile()) { response.writeHead(404).end(); return; }
      // Debug WASM fixtures can exceed 100 MiB. Release upload limits are
      // checked separately by the asset builder before publication.
      if (stat.size > 256 * 1024 * 1024) { response.writeHead(413).end(); return; }
      response.writeHead(200, { 'Content-Type': mime[path.extname(actual)] || 'application/octet-stream',
        'Content-Length': stat.size, 'Cache-Control': /\/(assets|media)-[a-f0-9]{20}\//.test(name)
          ? 'public, max-age=31536000, immutable' : 'no-cache' });
      if (request.method === 'HEAD') response.end();
      else fs.createReadStream(actual).on('error', () => response.destroy()).pipe(response);
    } catch (error) {
      if (!response.headersSent) response.writeHead(error.code === 'ENOENT' ? 404 : 500);
      response.end();
    }
  });
  server.requestTimeout = 30000;
  server.headersTimeout = 15000;
  server.keepAliveTimeout = 5000;
  server.maxRequestsPerSocket = 100;
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
  return { base: `http://127.0.0.1:${server.address().port}`,
    close: async () => { server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); } };
}

function command(program, args, options) {
  return new Promise((resolve, reject) => {
    const log = fs.createWriteStream(options.log);
    const child = spawn(program, args, { cwd: options.cwd, env: options.env, windowsHide: true,
      timeout: options.timeout, stdio: ['ignore', 'pipe', 'pipe'] });
    let bytes = 0, tail = '', exceeded = false;
    const record = chunk => {
      bytes += chunk.length;
      tail = (tail + chunk.toString()).slice(-16384);
      if (bytes <= 16 * 1024 * 1024) log.write(chunk);
      else { exceeded = true; child.kill(); }
    };
    child.stdout.on('data', record); child.stderr.on('data', record);
    child.once('error', error => { log.end(); reject(error); });
    child.once('close', (code, signal) => {
      log.end(() => {
        if (code !== 0 || exceeded) reject(Error(`${program} ${args.join(' ')} failed (${exceeded ? 'log budget exceeded' : signal || code})\n${tail}`));
        else resolve();
      });
    });
  });
}

function fileHashes(directory) {
  const hashes = {};
  const visit = (current, prefix = '') => {
    for (const entry of fs.readdirSync(current, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
      const name = prefix + entry.name, full = path.join(current, entry.name);
      if (entry.isDirectory()) visit(full, `${name}/`);
      else if (entry.isFile()) hashes[name] = hash(fs.readFileSync(full));
      else throw Error(`Unexpected linked fixture asset: ${full}`);
    }
  };
  visit(directory);
  return hashes;
}

function showcaseSuites(root, output) {
  const script = name => path.join(root, 'scripts', name);
  const browser = (id, name, report = `${name.replace(/\.cjs$/, '')}.json`) => ({
    id, program: process.execPath, args: [script(name)], source: script(name),
    report: path.join(artifactRoot, report),
  });
  return [
    browser('showcase', 'showcase-browser.cjs'),
    { ...browser('theme-studio', 'theme-studio-browser.cjs'),
      copiedExport: path.join(artifactRoot, 'studio-export.rs') },
    { ...browser('palette-order', 'palette-order-browser.cjs'), report: path.join(output, 'palette-order-browser.json') },
    { id: 'theme-export', program: process.env.SAPID_PWSH || 'pwsh', source: script('check-theme-export.ps1'),
      args: ['-NoProfile', '-File', script('check-theme-export.ps1')] },
    browser('navigation', 'showcase-navigation-browser.cjs'),
    browser('routing', 'showcase-routing-browser.cjs'),
    browser('style', 'showcase-style-browser.cjs'),
    browser('mantine-api', 'mantine-api-browser.cjs'),
    { id: 'cache', program: process.execPath, source: script('cache-browser.cjs'), args: [script('cache-browser.cjs')] },
  ];
}

async function runAcceptance(options) {
  const root = options.root || path.resolve(__dirname, '..');
  const directory = path.resolve(options.directory);
  const output = artifactPath(options.output || `${artifactRoot}/showcase-acceptance`);
  fs.mkdirSync(output, { recursive: true });
  const env = { ...process.env, TEMP: path.join(output, 'tmp'), TMP: path.join(output, 'tmp'),
    SAPID_SHOWCASE_MANIFEST: path.join(directory, 'asset-hashes.json'),
    SAPID_PALETTE_REPORT: path.join(output, 'palette-order-browser.json'),
    SAPID_STUDIO_EXPORT: process.env.SAPID_STUDIO_EXPORT || path.join(output, 'studio-export.rs'),
    SAPID_EXPORT_ROOT: process.env.SAPID_EXPORT_ROOT || path.join(output, 'export-check'),
    SAPID_EXPORT_TARGET: process.env.SAPID_EXPORT_TARGET || path.join(output, 'export-target'),
    PLAYWRIGHT_BROWSERS_PATH: process.env.PLAYWRIGHT_BROWSERS_PATH || `${artifactRoot}/browsers` };
  fs.mkdirSync(env.TEMP, { recursive: true });
  for (const name of ['SAPID_STUDIO_EXPORT', 'SAPID_EXPORT_ROOT', 'SAPID_EXPORT_TARGET']) artifactPath(env[name]);
  const suites = options.suites || showcaseSuites(root, output);
  const summary = { complete: false, directory, suiteManifest: suites.map(suite => suite.id),
    suites: suites.map(suite => ({ id: suite.id, status: 'pending' })), errors: [], sourceHashes: {}, artifactHashes: {} };
  const revision = spawnSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8', windowsHide: true });
  const dirty = spawnSync('git', ['status', '--porcelain'], { cwd: root, encoding: 'utf8', windowsHide: true });
  summary.harness = { revision: revision.status === 0 ? revision.stdout.trim() : null,
    dirty: dirty.status === 0 ? dirty.stdout.trim().length > 0 : null };
  const save = () => fs.writeFileSync(path.join(output, 'results.json'), JSON.stringify(summary, null, 2));
  let server;
  try {
    if (!options.suites && !env.SAPID_PLAYWRIGHT) env.SAPID_PLAYWRIGHT = require.resolve('playwright');
    for (const name of ['index.html', 'showcase.html', 'asset-manifest.json']) {
      if (!fs.statSync(path.join(directory, name)).isFile()) throw Error(`Missing showcase fixture: ${name}`);
    }
    summary.artifactHashes = fileHashes(directory);
    summary.sourceHashes[path.relative(root, __filename)] = hash(fs.readFileSync(__filename));
    for (const suite of suites) summary.sourceHashes[path.relative(root, suite.source)] = hash(fs.readFileSync(suite.source));
    for (const name of ['examples/web/showcase-acceptance.js', 'scripts/test_theme_studio_browser.cjs']) {
      if (fs.existsSync(path.join(root, name))) summary.sourceHashes[name] = hash(fs.readFileSync(path.join(root, name)));
    }
    server = await serve(directory); env.SAPID_SHOWCASE_BASE = server.base; summary.base = server.base;
    save();
    for (let index = 0; index < suites.length; index++) {
      const suite = suites[index], result = summary.suites[index];
      console.log(`Showcase acceptance ${index + 1}/${suites.length}: ${suite.id}`);
      result.status = 'running'; save();
      try {
        if (suite.report) fs.rmSync(suite.report, { force: true });
        if (suite.copiedExport) fs.rmSync(suite.copiedExport, { force: true });
        const log = path.join(output, `${suite.id}.log`);
        const args = suite.id === 'cache' ? [...suite.args, directory] : suite.args;
        await command(suite.program, args, { cwd: root, env, log, timeout: options.timeout || 600000 });
        result.log = path.basename(log);
        if (suite.report) {
          const report = JSON.parse(fs.readFileSync(suite.report, 'utf8').replace(/^\uFEFF/, ''));
          if (!Array.isArray(report.errors) || report.errors.length) throw Error(`${suite.id}: browser errors: ${JSON.stringify(report.errors)}`);
          result.report = `${suite.id}.json`;
          fs.writeFileSync(path.join(output, result.report), JSON.stringify(report, null, 2));
        }
        if (suite.copiedExport) {
          if (path.resolve(suite.copiedExport) !== path.resolve(env.SAPID_STUDIO_EXPORT)) {
            fs.mkdirSync(path.dirname(env.SAPID_STUDIO_EXPORT), { recursive: true });
            fs.copyFileSync(suite.copiedExport, env.SAPID_STUDIO_EXPORT);
          }
          result.exportSha256 = hash(fs.readFileSync(env.SAPID_STUDIO_EXPORT));
        }
        result.status = 'passed';
      } catch (error) {
        result.status = 'failed'; summary.errors.push(error.stack || String(error)); save(); break;
      }
      save();
    }
    if (JSON.stringify(fileHashes(directory)) !== JSON.stringify(summary.artifactHashes)) summary.errors.push('Showcase fixtures changed during acceptance');
    for (const [file, expected] of Object.entries(summary.sourceHashes)) {
      if (hash(fs.readFileSync(path.join(root, file))) !== expected) summary.errors.push(`Acceptance source changed: ${file}`);
    }
    summary.complete = summary.errors.length === 0 && summary.suites.every(suite => suite.status === 'passed');
  } catch (error) { summary.errors.push(error.stack || String(error)); }
  finally { if (server) await server.close(); save(); }
  return summary;
}

function argumentsFrom(args) {
  const result = {};
  for (let index = 0; index < args.length; index++) {
    const [name, ...rest] = args[index].split('=');
    if (!['--directory', '--output'].includes(name)) throw Error(`Unknown argument: ${name}`);
    result[name.slice(2)] = rest.length ? rest.join('=') : args[++index];
    if (!result[name.slice(2)]) throw Error(`Missing value for ${name}`);
  }
  if (!result.directory) throw Error('--directory is required');
  return result;
}

module.exports = { serve, runAcceptance, showcaseSuites };
if (require.main === module) runAcceptance(argumentsFrom(process.argv.slice(2))).then(result => {
  console.log(JSON.stringify({ complete: result.complete, suites: result.suites, errors: result.errors }, null, 2));
  if (!result.complete || result.errors.length) process.exitCode = 1;
}).catch(error => { console.error(error); process.exitCode = 1; });
