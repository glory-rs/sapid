const fs = require('node:fs');
const assert = require('node:assert/strict');
process.env.TEMP = process.env.TMP = 'D:/BuildCache/sapid/tmp';
const { chromium } = require(process.env.SAPID_PLAYWRIGHT || 'D:/Works/glory-rs/glory/tests/playwright/node_modules/playwright');
const base = process.env.SAPID_SHOWCASE_BASE || 'http://127.0.0.1:8795';
const output = process.env.SAPID_PALETTE_REPORT || 'D:/BuildCache/sapid/palette-order-browser.json';
(async () => {
  const browser = await chromium.launch({ headless: true });
  const report = { base, checks: [], errors: [] };
  try {
    const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
    const page = await context.newPage();
    page.on('pageerror', error => report.errors.push(String(error)));
    const ready = () => page.waitForFunction(() => window.ready === true);
    const settled = () => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
    await page.goto(`${base}/showcase.html?widget=theme-studio`);
    await ready();
    await page.getByRole('textbox', { name: 'Theme name', exact: true }).fill('Order 1');
    await page.getByRole('button', { name: 'Save & preview', exact: true }).click();
    await page.waitForFunction(() => document.documentElement.getAttribute('data-sapid-palette') === 'custom-1');
    await page.evaluate(() => {
      const key = 'sapid.theme-studio.v1';
      const store = JSON.parse(localStorage.getItem(key));
      store.themes = Array.from({ length: 8 }, (_, index) => ({
        ...store.themes[0], id: `custom-${index + 1}`, title: `Order ${index + 1}`,
      }));
      store.next_id = 9;
      localStorage.setItem(key, JSON.stringify(store));
    });
    const labels = ['blue', 'green', ...Array.from({ length: 8 }, (_, i) => `Order ${i + 1}`)].map(name => `Use ${name} palette`);
    const order = () => page.locator('.site-palette').evaluateAll(nodes => nodes.map(node => node.getAttribute('aria-label')));
    const positions = () => page.locator('.site-palette').evaluateAll(nodes => nodes.map(node => {
      const box = node.getBoundingClientRect();
      return [node.getAttribute('aria-label'), box.x, box.y];
    }));
    const select = async (name, id) => {
      const link = page.getByRole('link', { name: `Use ${name} palette`, exact: true });
      await link.scrollIntoViewIfNeeded();
      await settled();
      const before = await positions();
      await link.click();
      await page.waitForFunction(expected => document.documentElement.getAttribute('data-sapid-palette') === expected, id);
      await settled();
      assert.deepEqual(await order(), labels, `${name}: palette order changed`);
      assert.deepEqual(await positions(), before, `${name}: swatches moved after selection`);
      assert.equal(await page.locator('.site-palette[aria-current=location]').getAttribute('aria-label'), `Use ${name} palette`);
      report.checks.push(`${name} selected with stable order and positions`);
    };
    for (const width of [1440, 390]) {
      await page.setViewportSize({ width, height: 1000 });
      for (const entry of ['showcase.html?widget=P0-01', 'ssr-button-blue-dark.html']) {
        await page.goto(`${base}/${entry}`);
        await ready();
        await page.waitForFunction(() => document.querySelectorAll('.site-palette').length === 10);
        assert.deepEqual(await order(), labels);
        for (const index of [8, 7, 8, 6]) await select(`Order ${index}`, `custom-${index}`);
        await select('blue', 'blue');
        await select('green', 'green');
        await select('blue', 'blue');
        await page.reload();
        await ready();
        assert.deepEqual(await order(), labels);
        report.checks.push(`${width}px ${entry}: reload preserves saved order`);
      }
    }
  } catch (error) {
    report.errors.push(error.stack || String(error));
  } finally {
    await browser.close();
    fs.writeFileSync(output, JSON.stringify(report, null, 2));
    console.log(JSON.stringify(report, null, 2));
    if (report.errors.length) process.exitCode = 1;
  }
})();
