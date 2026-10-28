// Exercise a two-generation deployment with the real SSR/CSR + release WASM.
const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const { chromium } = require(process.env.GLORY_PLAYWRIGHT || process.env.SAPID_PLAYWRIGHT || 'D:/Works/glory-rs/glory/tests/playwright/node_modules/playwright');
process.env.TEMP = process.env.TMP = 'D:/BuildCache/sapid/tmp/site-cache-tests';
fs.mkdirSync(process.env.TEMP, { recursive: true });
const directory = path.resolve(process.argv[2]);
const manifest = JSON.parse(fs.readFileSync(path.join(directory, 'asset-manifest.json')));
const first = manifest.releases[0].id;
const cssName = `${first}/site.css`;
const css = fs.readFileSync(path.join(directory, cssName));
const nextCSS = Buffer.concat([css, Buffer.from('\n/* next-deployment cache acceptance */\n')]);
const second = `assets-${crypto.createHash('sha256').update(nextCSS).digest('hex').slice(0, 20)}`;
assert.notEqual(first, second);
const original = fs.readFileSync(path.join(directory, 'index.html'), 'utf8');
const next = original.replaceAll(first, second);
assert.notEqual(original, next);
let generation = 1;
const requests = [];
const mime = { '.html': 'text/html; charset=utf-8', '.js': 'application/javascript', '.wasm': 'application/wasm', '.css': 'text/css', '.svg': 'image/svg+xml', '.ico': 'image/x-icon', '.png': 'image/png' };
const server = http.createServer((request, response) => {
  const name = decodeURIComponent(new URL(request.url, 'http://localhost').pathname).slice(1);
  let body;
  let extension = path.extname(name);
  const isHTML = name === '' || name === 'index.html';
  if (isHTML) {
    const legacy = request.headers['x-legacy-document'] === '1';
    body = Buffer.from(generation === 1 || legacy ? original : next);
    extension = '.html';
  } else {
    const actual = name.startsWith(`${second}/`) ? name.replace(second, first) : name;
    const file = path.resolve(directory, actual);
    if (!file.startsWith(directory + path.sep) || !fs.existsSync(file) || !fs.statSync(file).isFile()) {
      response.writeHead(404).end();
      return;
    }
    body = name === `${second}/site.css` ? nextCSS : fs.readFileSync(file);
  }
  const etag = `"${crypto.createHash('sha256').update(body).digest('hex')}"`;
  const immutable = /^(assets|media)-[a-f0-9]{20}\//.test(name);
  response.setHeader('Cache-Control', immutable ? 'public, max-age=31536000, immutable' : 'public, max-age=0, must-revalidate');
  response.setHeader('ETag', etag);
  const status = request.headers['if-none-match'] === etag ? 304 : 200;
  requests.push({ name, status, generation });
  response.writeHead(status, { 'Content-Type': mime[extension] || 'application/octet-stream' });
  response.end(status === 304 ? undefined : body);
});

(async () => {
  let browser;
  try {
    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
    const base = `http://127.0.0.1:${server.address().port}/`;
    browser = await chromium.launch({ headless: true });
    const context = await browser.newContext();
    const errors = [];
    const open = async (ctx) => {
      const page = await ctx.newPage();
      page.on('pageerror', error => errors.push(error.message));
      await page.goto(base);
      await page.waitForFunction(() => window.gloryReady === true || window.ready === true);
      return page;
    };
    await open(context);
    const oldCSSRequests = requests.filter(r => r.name === cssName).length;
    await open(context);
    assert(requests.some(r => r.name === '' && r.status === 304), 'unchanged entry HTML must revalidate with ETag/304');
    assert.equal(requests.filter(r => r.name === cssName).length, oldCSSRequests, 'immutable CSS should be reused from browser cache');
    generation = 2;
    const updated = await open(context);
    assert(await updated.locator(`link[href*="${second}/site.css"]`).count());
    for (const suffix of ['.css', '.js', '.wasm']) {
      assert(requests.some(r => r.generation === 2 && r.name.startsWith(second + '/') && r.name.endsWith(suffix) && r.status === 200), `new ${suffix} URL must fetch the new asset graph`);
    }
    // Fresh clients with an older entry document can still fetch its full graph after rollout.
    const legacyContext = await browser.newContext({ extraHTTPHeaders: { 'x-legacy-document': '1' } });
    const legacy = await open(legacyContext);
    assert(await legacy.locator(`link[href*="${first}/site.css"]`).count());
    assert(requests.some(r => r.generation === 2 && r.name.startsWith(first + '/') && r.name.endsWith('.wasm') && r.status === 200));
    assert.deepEqual(errors, []);
    console.log('PASS real release assets: ETag/304 entry, immutable browser reuse, new CSS/JS/WASM URLs, and old-document loading after rollout');
  } finally {
    await browser?.close();
    server.closeAllConnections();
    await new Promise(resolve => server.close(resolve));
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
