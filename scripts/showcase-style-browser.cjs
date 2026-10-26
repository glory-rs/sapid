// Visual and interaction regressions for shared tag/icon/choice styling.
const fs = require('node:fs');
const assert = require('node:assert/strict');
process.env.TEMP = process.env.TMP = 'D:/BuildCache/sapid/tmp';
const { chromium } = require(process.env.SAPID_PLAYWRIGHT || 'D:/Works/glory-rs/glory/tests/playwright/node_modules/playwright');
const base = process.env.SAPID_SHOWCASE_BASE || 'http://127.0.0.1:8795';
const output = 'D:/BuildCache/sapid';

(async () => {
  const report = { reports: [], errors: [] };
  let browser;
  try {
    browser = await chromium.launch({ headless: true });
    report.chromium = browser.version();
    const page = await browser.newPage();
    page.on('pageerror', error => report.errors.push(error.message));
    const tick = () => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
    for (const theme of ['blue', 'green']) for (const scheme of ['light', 'dark']) {
      for (const width of [1440, 360]) for (const hydrate of [false, true]) {
        await page.setViewportSize({ width, height: 1000 });
        for (const [id, name, selector] of [
          ['P4-08', 'multi-select', '.sapid-multi-select-tag'],
          ['P4-09', 'tags', '.sapid-tags-input-tag,.sapid-pill-input-tag'],
          ['P3-10', 'pills', '.sapid-pill'],
          ['P2-08', 'actions', '.sapid-action-icon'],
        ]) {
          const suite = `${name}-${theme}-${scheme}-${width}-${hydrate ? 'hydrate' : 'csr'}`;
          await page.goto(`${base}/${hydrate ? `ssr-${name}-${theme}-${scheme}.html` : `showcase.html?widget=${id}&theme=${theme}&scheme=${scheme}`}`);
          await page.waitForFunction(() => window.ready === true);
          const demo = page.locator('.site-demo');
          const subject = name === 'multi-select' ? demo.locator('.sapid-field').filter({has:page.getByRole('combobox',{name:'Countries',exact:true})}) : demo;
          assert.equal(await demo.locator('button:not([class*="sapid-"])').count(), 0, `${suite}: public demo uses styled actions`);
          if (name === 'multi-select') {
            await demo.getByRole('button', { name: 'Select filtered countries', exact: true }).click();
            await tick();
            assert.equal(await subject.locator(selector).count(), 4, `${suite}: selected tags`);
          }
          const boxes = await subject.locator(selector).evaluateAll(nodes => nodes.filter(node => node.checkVisibility()).map(node => {
            const r = node.getBoundingClientRect(), button = node.matches('button') ? node : node.querySelector('button');
            const b = button?.getBoundingClientRect(), style = button && getComputedStyle(button);
            return { width: r.width, height: r.height, x: r.x, y: r.y, button: b && { width: b.width, height: b.height, padding: style.padding, border: style.borderTopColor, cursor: style.cursor } };
          }));
          assert(boxes.length, `${suite}: visible widgets`);
          for (const box of boxes) {
            assert(box.width > 0 && box.height > 0, `${suite}: nonempty box`);
            if (!box.button) { assert.equal(name, 'pills', `${suite}: passive pill`); continue; }
            assert(Math.abs(box.button.width - box.button.height) < 1, `${suite}: square button`);
            if (name !== 'actions') {
              assert(box.height <= 28, `${suite}: compact tag ${box.height}`);
              assert(box.button.height <= 20, `${suite}: compact remove ${box.button.height}`);
              assert(box.width <= 240, `${suite}: content-sized pill ${box.width}`);
            }
          }
          assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), `${suite}: no page overflow`);
          if (!hydrate && theme === 'blue' && scheme === 'light') {
            await page.locator('.site-preview').screenshot({ path: `${output}/style-${name}-${width}.png` });
          }
          if (name === 'multi-select') {
            if (width === 1440) assert.equal(new Set(boxes.map(box => Math.round(box.y))).size, 1, `${suite}: tags share a row`);
            const input = demo.getByRole('combobox', { name: 'Countries', exact: true });
            await input.focus();
            await page.keyboard.press('ArrowDown');
            await tick();
            assert.equal(await input.evaluate(node => getComputedStyle(node).outlineStyle), 'none', `${suite}: focus uses themed border`);
            const picked = page.locator('.sapid-combobox-option[aria-selected=true]').first();
            assert(await picked.isVisible(), `${suite}: selected option visible`);
            assert((await picked.evaluate(node => getComputedStyle(node, '::after').content)).includes('✓'), `${suite}: selection mark`);
            await page.keyboard.press('Escape');
            await demo.getByRole('button', { name: 'Remove Canada', exact: true }).click();
            await tick();
            assert.equal(await subject.locator(selector).count(), 3, `${suite}: removal`);
            assert(await input.evaluate(node => node === document.activeElement), `${suite}: focus returns to editor`);
            await demo.getByRole('checkbox', { name: 'Read only', exact: true }).check();
            await tick();
            assert(await demo.getByRole('button', { name: 'Remove France', exact: true }).isDisabled(), `${suite}: readonly removal`);
            await demo.getByRole('checkbox', { name: 'Read only', exact: true }).uncheck();
            await demo.getByRole('checkbox', { name: 'Disabled', exact: true }).check();
            await tick();
            assert(await input.isDisabled(), `${suite}: disabled editor`);
          } else if (name === 'tags') {
            const input = demo.getByRole('textbox', { name: 'Skills', exact: true });
            await demo.getByRole('button', { name: 'Remove Rust (1)', exact: true }).click();
            await tick();
            assert.equal(await demo.locator('.sapid-tags-input-tag').count(), 1, `${suite}: tag removal`);
            assert(await input.evaluate(node => node === document.activeElement), `${suite}: editor focus`);
            await input.fill('Long label '.repeat(20));
            await page.keyboard.press('Enter');
            await tick();
            const label = demo.locator('.sapid-tags-input-tag .sapid-tag-label').filter({ hasText: 'Long label' });
            assert.equal(await label.count(), 1, `${suite}: long tag accepted`);
            assert.equal(await label.evaluate(node => getComputedStyle(node).textOverflow), 'ellipsis', `${suite}: long label truncation`);
            assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), `${suite}: long label stays in page`);
          } else if (name === 'pills') {
            await demo.getByRole('button', { name: 'Remove local tag', exact: true }).click();
            await tick();
            assert.equal(await demo.locator('.local-pill').isVisible(), false, `${suite}: local dismissal`);
            assert(await demo.getByRole('button', { name: 'Remove disabled tag', exact: true }).isDisabled(), `${suite}: disabled removal`);
            await demo.getByRole('button', { name: 'Request controlled removal', exact: true }).click();
            await tick();
            assert(await demo.locator('.controlled-pill').isVisible(), `${suite}: controlled refusal`);
          } else {
            const action = demo.getByRole('button', { name: 'Light icon action', exact: true });
            const original = await action.evaluate(node => getComputedStyle(node).color);
            await demo.getByRole('checkbox', { name: 'Disabled', exact: true }).check();
            await tick();
            assert(await action.isDisabled(), `${suite}: native disable`);
            assert.notEqual(await action.evaluate(node => getComputedStyle(node).color), original, `${suite}: disabled color`);
            await demo.getByRole('checkbox', { name: 'Disabled', exact: true }).uncheck();
            await tick();
            assert.equal(await action.evaluate(node => getComputedStyle(node).color), original, `${suite}: restored variant`);
            await action.focus();
            await page.keyboard.press('Tab');
            assert.equal(await page.locator(':focus').evaluate(node => getComputedStyle(node).outlineStyle), 'solid', `${suite}: keyboard focus`);
          }
          report.reports.push({ suite, boxes, status: 'passed' });
        }
      }
    }
    await page.emulateMedia({ reducedMotion: 'reduce' });
    assert.equal(await page.locator('.sapid-action-icon').first().evaluate(node => getComputedStyle(node).transitionDuration), '0s');
  } catch (error) {
    report.errors.push(error.stack || String(error));
  } finally {
    if (browser) await browser.close();
    fs.writeFileSync(`${output}/showcase-style-browser.json`, JSON.stringify(report, null, 2));
    console.log(JSON.stringify({ reports: report.reports.length, errors: report.errors }, null, 2));
    if (report.errors.length) process.exitCode = 1;
  }
})();
