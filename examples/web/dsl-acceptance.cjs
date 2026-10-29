'use strict';
const fs = require('node:fs'), path = require('node:path'), http = require('node:http'), assert = require('node:assert/strict');
const { chromium } = require(process.env.SAPID_PLAYWRIGHT || 'D:/BuildCache/glory-v2-dioxus-20261028/tools/node_modules/playwright');
const root = path.resolve(process.env.SAPID_DSL_SITE || 'D:/BuildCache/sapid/dsl');
process.env.TEMP = process.env.TMP = 'D:/BuildCache/sapid/tmp';
const server = http.createServer((req, res) => {
    const file = path.resolve(root, '.' + new URL(req.url, 'http://localhost').pathname);
    if (!file.startsWith(root + path.sep) || !fs.existsSync(file)) { res.statusCode = 404; res.end(); return; }
    res.setHeader('Content-Type', file.endsWith('.wasm') ? 'application/wasm' : file.endsWith('.js') ? 'text/javascript' : file.endsWith('.css') ? 'text/css' : 'text/html; charset=utf-8');
    fs.createReadStream(file).pipe(res);
});
(async () => {
    await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
    const browser = await chromium.launch({ headless: true, executablePath: process.env.SAPID_CHROME || 'C:/Program Files/Google/Chrome/Application/chrome.exe' });
    const errors = [], evidence = { checks: [] };
    try {
        for (const hydrate of [false, true]) {
            const page = await browser.newPage(); page.on('pageerror', e => errors.push(e.stack));
            await page.goto(`http://127.0.0.1:${server.address().port}/dsl${hydrate ? '-hydrate' : ''}.html?manual`);
            await page.waitForFunction(() => window.moduleReady);
            if (hydrate) await page.evaluate(() => {
                window.saved = document.querySelector('.dsl-editor'); window.raw = document.querySelector('.dsl-raw');
                saved.value = 'before adoption'; saved.focus(); saved.setSelectionRange(2, 6);
            });
            await page.evaluate(hydrate => window.demo.dsl_mount(hydrate), hydrate);
            if (hydrate) assert(await page.evaluate(() => saved === document.querySelector('.dsl-editor') && raw === document.querySelector('.dsl-raw') && saved.value === 'before adoption' && saved.selectionStart === 2 && saved.selectionEnd === 6));
            for (const className of ['dsl-raw', 'dsl-type', 'dsl-factory', 'dsl-concrete']) await page.locator('.' + className).click();
            await page.waitForFunction(() => document.querySelector('.dsl-count').textContent === '4');
            assert.equal(await page.locator('.dsl-type').textContent(), 'Imported type 4');
            await page.evaluate(() => { window.valueRow=document.querySelector('.dsl-value-row'); window.memberRow=document.querySelector('.dsl-member-row'); });
            await page.locator('.dsl-reorder').click();
            await page.waitForFunction(() => document.querySelector('.dsl-value-row').textContent === '2:0');
            assert.deepEqual(await page.locator('.dsl-value-row').allTextContents(), ['2:0', '1:1']);
            assert.deepEqual(await page.locator('.dsl-member-row').allTextContents(), ['2', '1']);
            assert(await page.evaluate(() => valueRow===document.querySelectorAll('.dsl-value-row')[1] && memberRow===document.querySelectorAll('.dsl-member-row')[1]));
            await page.locator('.dsl-editor').fill('Shared 🦀');
            await page.waitForFunction(() => document.querySelector('.dsl-name').textContent === 'Shared 🦀');
            for (let i = 0; i < 4; i++) {
                await page.locator('.dsl-toggle').click(); await page.waitForFunction(() => !document.querySelector('.dsl-branch'));
                await page.locator('.dsl-toggle').click(); await page.waitForFunction(() => !!document.querySelector('.dsl-branch'));
            }
            assert.equal(await page.evaluate(() => window.demo.dsl_close()), 0);
            assert.equal(await page.locator('#app > *').count(), 0);
            evidence.checks.push(`${hydrate ? 'hydration' : 'CSR'}: HTML/type/opaque/concrete factory events, reactive fields, controlled input, branches, teardown`);
            await page.close();
        }
        assert.deepEqual(errors, []); evidence.browser = browser.version(); evidence.passed = true;
        fs.writeFileSync(path.join(root, 'evidence.json'), JSON.stringify(evidence, null, 2));
        console.log(JSON.stringify(evidence));
    } finally { await browser.close(); server.close(); }
})().catch(error => { console.error(error); server.close(); process.exitCode = 1; });
