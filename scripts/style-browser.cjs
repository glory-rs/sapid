// Focused CSS acceptance with an in-process server; no WASM fixture build needed.
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const { chromium } = require(process.env.SAPID_PLAYWRIGHT || 'playwright');
const output = process.env.SAPID_STYLE_OUTPUT || 'D:/BuildCache/sapid/style-browser';
process.env.SAPID_STYLE_OUTPUT = output;
process.env.TEMP = process.env.TMP = `${output}/tmp`;
fs.mkdirSync(process.env.TEMP, { recursive: true });
const css = fs.readFileSync(path.join(__dirname, '../styles/sapid.css'));
const server = http.createServer((request, response) => {
  if (request.url === '/sapid.css') {
    response.writeHead(200, { 'Content-Type': 'text/css' }); response.end(css);
  } else { response.writeHead(404); response.end(); }
});
(async () => {
  let browser;
  try {
    await new Promise((resolve, reject) => {
      server.once('error', reject); server.listen(0, '127.0.0.1', resolve);
    });
    process.env.SAPID_STYLE_BASE = `http://127.0.0.1:${server.address().port}`;
    browser = await chromium.launch({ headless: true, executablePath: process.env.SAPID_CHROME });
    const source = fs.readFileSync(path.join(__dirname, '../examples/web/style-acceptance.js'), 'utf8');
    const result = await new Function(`return (${source})`)()(await browser.newPage());
    fs.writeFileSync(`${output}/browser-results.json`, JSON.stringify(result, null, 2));
    console.log(JSON.stringify(result, null, 2));
    if (result.errors.length) process.exitCode = 1;
  } finally {
    if (browser) await browser.close();
    await new Promise(resolve => server.close(resolve));
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
