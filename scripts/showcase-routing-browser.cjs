const assert = require('node:assert/strict');
const fs = require('node:fs');
const { chromium } = require(process.env.SAPID_PLAYWRIGHT || 'D:/Works/glory-rs/glory/tests/playwright/node_modules/playwright');
const base = process.env.SAPID_SHOWCASE_BASE || 'http://127.0.0.1:8795';
process.env.TEMP = process.env.TMP = 'D:/BuildCache/sapid/tmp';
(async () => {
  const browser = await chromium.launch({ headless: true, ...(process.env.GLORY_CHROME ? { executablePath: process.env.GLORY_CHROME } : {}) });
  const report = { checks: [], errors: [] };
  const ready = (page, widget, palette, scheme) => page.waitForFunction(({ widget, palette, scheme }) => window.ready &&
    document.documentElement.getAttribute('data-sapid-widget') === widget &&
    document.documentElement.getAttribute('data-sapid-palette') === palette &&
    document.documentElement.getAttribute('data-sapid-scheme') === scheme, { widget, palette, scheme });
  try {
    const page = await browser.newPage();
    page.on('pageerror', error => report.errors.push(error.message));
    await page.route('**/sapid_web_example.js', async route => {
      await page.evaluate(() => history.replaceState({ otherApplication: 'retained' }, '', location.href));
      await route.continue();
    });
    await page.goto(base + '/showcase.html?widget=P0-01&theme=blue&scheme=dark');
    await ready(page, 'P0-01', 'blue', 'dark');
    const firstKey = await page.evaluate(() => history.state.__glory_routing_v1);
    assert.equal(typeof firstKey, 'string');
    await page.locator('.site-demo').getByRole('button', { name: 'Filled button', exact: true }).click();
    await page.getByText('Clicked 1 times', { exact: true }).waitFor();
    await page.evaluate(() => { window.retainedDemo = document.querySelector('.site-demo'); });
    await page.getByRole('navigation', { name: 'On this page' }).getByRole('link', { name: 'Composition', exact: true }).click();
    await page.waitForURL(/#composition$/);
    const secondKey = await page.evaluate(() => history.state.__glory_routing_v1);
    assert.notEqual(secondKey, firstKey);
    assert.equal(await page.evaluate(() => history.state.otherApplication), 'retained');
    await page.goBack(); await ready(page, 'P0-01', 'blue', 'dark');
    assert.equal(await page.evaluate(() => history.state.__glory_routing_v1), firstKey);
    assert(await page.evaluate(() => window.retainedDemo === document.querySelector('.site-demo')));
    await page.getByText('Clicked 1 times', { exact: true }).waitFor();
    report.checks.push('Glory history keys retain foreign state and fragment traversal keeps the live demo');

    const late = await browser.newPage();
    late.on('pageerror', error => report.errors.push(error.message));
    await late.route('**/sapid_web_example.js', async route => {
      assert.equal(await late.locator('.site-demo').count(), 1);
      await late.evaluate(() => history.replaceState({ foreign: 'late' }, '', '/showcase.html?widget=P0-02&theme=green&scheme=light'));
      await route.continue();
    });
    await late.goto(base + '/ssr-button-blue-dark.html');
    await ready(late, 'P0-02', 'green', 'light');
    assert.equal(await late.evaluate(() => history.state.foreign), 'late');
    report.checks.push('SSR alias adopts its bootstrap before a changed pre-WASM URL selects a different widget/theme');

    assert.deepEqual(report.errors, []);
  } finally {
    await browser.close();
    fs.writeFileSync('D:/BuildCache/sapid/showcase-routing-browser.json', JSON.stringify(report, null, 2));
  }
  console.log(JSON.stringify(report, null, 2));
})().catch(error => { console.error(error); process.exitCode = 1; });
