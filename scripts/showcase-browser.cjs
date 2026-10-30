const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { chromium } = require(process.env.SAPID_PLAYWRIGHT || 'D:/Works/glory-rs/glory/tests/playwright/node_modules/playwright');
process.env.TEMP = process.env.TMP = 'D:/BuildCache/sapid/tmp';
const output = 'D:/BuildCache/sapid';
(async () => {
  let browser;
  const report = { base: process.env.SAPID_SHOWCASE_BASE || 'http://127.0.0.1:8795', reports: [], errors: [] };
  const manifest = process.env.SAPID_SHOWCASE_MANIFEST || path.join(output, 'showcase/asset-hashes.json');
  if (fs.existsSync(manifest)) report.assetManifestSha256 = crypto.createHash('sha256').update(fs.readFileSync(manifest)).digest('hex');
  try {
    browser = await chromium.launch({headless:true});
    report.chromium = browser.version();
    const page = await browser.newPage();
    const source = fs.readFileSync(path.join(__dirname, '../examples/web/showcase-acceptance.js'), 'utf8');
    const suite = eval(`(${source})`);
    Object.assign(report, await suite(page));
  } catch (error) { report.errors.push(error.stack || String(error)); }
  finally {
    if (browser) await browser.close();
    fs.writeFileSync(path.join(output, 'showcase-browser.json'), JSON.stringify(report, null, 2));
    console.log(JSON.stringify(report, null, 2));
    if (report.errors.length) process.exitCode = 1;
  }
})();
