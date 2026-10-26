// Real-browser checks for audit batches A1–A3. No UI is built in JavaScript.
const fs = require('node:fs');
const assert = require('node:assert/strict');
process.env.TEMP = process.env.TMP = 'D:/BuildCache/sapid/tmp';
const {chromium} = require(process.env.SAPID_PLAYWRIGHT || 'D:/Works/glory-rs/glory/tests/playwright/node_modules/playwright');
const base = process.env.SAPID_SHOWCASE_BASE || 'http://127.0.0.1:8795';
const output = 'D:/BuildCache/sapid';

(async () => {
  const report = {reports: [], errors: []};
  let browser;
  try {
    browser = await chromium.launch({headless: true});
    report.chromium = browser.version();
    const page = await browser.newPage();
    page.on('pageerror', error => report.errors.push(error.message));
    page.on('console', message => {if (message.type() === 'error') report.errors.push(message.text());});
    const tick = () => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
    for (const theme of ['blue', 'green']) for (const scheme of ['light', 'dark']) {
      for (const width of [1440, 360]) for (const hydrate of [false, true]) {
        const suffix = `${theme}-${scheme}-${width}-${hydrate ? 'hydrate' : 'csr'}`;
        await page.setViewportSize({width, height: 1000});
        const go = async (id, name) => {
          await page.goto(`${base}/${hydrate ? `ssr-${name}-${theme}-${scheme}.html` : `showcase.html?widget=${id}&theme=${theme}&scheme=${scheme}`}`);
          await page.waitForFunction(() => window.ready === true);
          return page.locator('.site-demo');
        };

        // A1: visible options, groups, selection admission and a focus-preserving clear.
        let demo = await go('P4-08', 'multi-select');
        const tools = demo.getByRole('combobox', {name: 'Tools', exact: true});
        const field = demo.locator('.sapid-field').filter({has: page.getByRole('combobox', {name: 'Tools', exact: true})});
        const popup = page.getByRole('listbox', {name: 'Tools', exact: true});
        const tags = field.locator('.sapid-multi-select-tag');
        const option = name => popup.locator('.sapid-combobox-option').filter({hasText: new RegExp(`^${name}$`)});
        assert.equal(await tools.getAttribute('placeholder'), 'Search tools');
        await tools.focus();
        await tools.press('ArrowDown');
        await tick();
        assert.equal(await option('Rust').isVisible(), false, `${suffix}: accepted pick hidden`);
        await option('Glory').click();
        await tick();
        assert.equal(await tags.count(), 2, `${suffix}: two accepted values`);
        assert.equal(await popup.locator('[role=group][aria-label=Runtime]').isVisible(), false, `${suffix}: empty group hidden`);
        assert(await option('Sapid').isDisabled(), `${suffix}: unselected option disabled at limit`);
        assert.equal(await tools.getAttribute('aria-activedescendant'), null, `${suffix}: no stale active option`);
        await field.getByRole('button', {name: 'Clear tools', exact: true}).click();
        await tick();
        assert.equal(await tags.count(), 0, `${suffix}: cleared`);
        assert(await tools.evaluate(node => node === document.activeElement), `${suffix}: focus returned`);
        assert(await option('Rust').isVisible() && await option('Sapid').isEnabled(), `${suffix}: options reappear`);
        await option('Rust').click();
        await tools.fill('Sa');
        await tick();
        await field.getByRole('button', {name: 'Clear tools', exact: true}).click();
        await tick();
        assert.equal(await tools.inputValue(), 'Sa', `${suffix}: clear preserves independent search`);
        assert.equal(await tags.count(), 0);
        await tools.fill('Rust');
        await tick();
        await option('Rust').click();
        await tools.fill('Rust');
        await tick();
        assert(await popup.getByText('No matching options', {exact: true}).isVisible(), `${suffix}: picked-only query is empty`);
        await tools.fill('');
        await tools.focus();
        const cdp = await page.context().newCDPSession(page);
        await cdp.send('Input.imeSetComposition', {text: '候选', selectionStart: 2, selectionEnd: 2});
        await tick();
        assert.equal(await field.getByRole('button', {name: 'Clear tools', exact: true}).count(), 0, `${suffix}: no clear during composition`);
        await cdp.send('Input.insertText', {text: '候选'});
        await tick();
        await cdp.detach();
        const countries = demo.getByRole('combobox', {name: 'Countries', exact: true});
        await demo.getByRole('checkbox', {name: 'Read only', exact: true}).check();
        await tick();
        assert.equal(await demo.getByRole('button', {name: 'Clear countries', exact: true}).count(), 0, `${suffix}: readonly clear absent`);
        await demo.getByRole('checkbox', {name: 'Read only', exact: true}).uncheck();
        await demo.getByRole('checkbox', {name: 'Disabled', exact: true}).check();
        await tick();
        assert(await countries.isDisabled());
        assert.equal(await demo.getByRole('button', {name: 'Clear countries', exact: true}).count(), 0, `${suffix}: disabled clear absent`);
        assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), `${suffix}: no choice overflow`);
        report.reports.push({suite: `multi-select-policy-${suffix}`, status: 'passed'});

        // A2: a reactive hint is an attribute, never an editor/property replacement.
        demo = await go('P0-02', 'textfield');
        const input = demo.getByRole('textbox', {name: 'Your name', exact: true});
        assert.equal(await input.getAttribute('placeholder'), 'Ada Lovelace');
        await input.fill('😀 Rust');
        await tick();
        await input.evaluate(node => {
          node.setSelectionRange(2, 4);
          window.placeholderEditor = node;
          window.placeholderValueWrites = 0;
          const property = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value');
          Object.defineProperty(node, 'value', {configurable: true, get() {return property.get.call(this);}, set(value) {window.placeholderValueWrites++; property.set.call(this, value);}});
        });
        await demo.getByRole('button', {name: 'Change hint', exact: true}).evaluate(node => node.click());
        await tick();
        assert.equal(await input.getAttribute('placeholder'), 'Grace Hopper');
        assert.equal(await input.inputValue(), '😀 Rust');
        assert.deepEqual(await input.evaluate(node => [node.selectionStart, node.selectionEnd]), [2, 4]);
        assert(await input.evaluate(node => node === window.placeholderEditor));
        assert.equal(await page.evaluate(() => window.placeholderValueWrites), 0, `${suffix}: no value setter on hint update`);
        await input.focus();
        const ime = await page.context().newCDPSession(page);
        await ime.send('Input.imeSetComposition', {text: '输入', selectionStart: 2, selectionEnd: 2});
        await tick();
        const composing = await input.inputValue();
        await page.evaluate(() => {window.placeholderValueWrites = 0;});
        await demo.getByRole('button', {name: 'Change hint', exact: true}).evaluate(node => node.click());
        await tick();
        assert.equal(await input.inputValue(), composing, `${suffix}: preedit preserved`);
        assert.equal(await page.evaluate(() => window.placeholderValueWrites), 0, `${suffix}: no composing value rewrite`);
        await ime.send('Input.insertText', {text: '输入'});
        await tick();
        await ime.detach();
        report.reports.push({suite: `placeholder-editor-${suffix}`, status: 'passed'});

        // A3: use actual variant builders rather than showcase-only style overrides.
        demo = await go('P0-01', 'button');
        const light = demo.getByRole('button', {name: 'Light variant', exact: true});
        const outline = demo.getByRole('button', {name: 'Outline variant', exact: true});
        const subtle = demo.getByRole('button', {name: 'Subtle variant', exact: true});
        const colors = await light.evaluate(node => ({background: getComputedStyle(node).backgroundColor, color: getComputedStyle(node).color}));
        assert.notEqual(colors.background, 'rgba(0, 0, 0, 0)');
        assert.equal(await outline.evaluate(node => getComputedStyle(node).backgroundColor), 'rgba(0, 0, 0, 0)');
        assert.equal(await subtle.evaluate(node => getComputedStyle(node).backgroundColor), 'rgba(0, 0, 0, 0)');
        await demo.getByRole('checkbox', {name: 'Loading', exact: true}).check();
        await tick();
        for (const button of [light, outline, subtle]) assert(await button.isDisabled());
        await light.evaluate(node => node.click());
        assert(await demo.getByText('Clicked 0 times', {exact: true}).isVisible());
        await demo.getByRole('checkbox', {name: 'Loading', exact: true}).uncheck();
        await demo.getByRole('checkbox', {name: 'Disabled', exact: true}).check();
        await tick();
        for (const button of [light, outline, subtle]) assert(await button.isDisabled());
        await demo.getByRole('checkbox', {name: 'Disabled', exact: true}).uncheck();
        await tick();
        await page.waitForFunction(color => getComputedStyle(document.querySelector('.site-demo .sapid-button-light')).backgroundColor === color, colors.background);
        assert.equal(await light.evaluate(node => getComputedStyle(node).backgroundColor), colors.background);
        await light.click();
        await tick();
        assert(await demo.getByText('Clicked 1 times', {exact: true}).isVisible());
        assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), `${suffix}: no button overflow`);
        report.reports.push({suite: `button-variants-${suffix}`, status: 'passed'});
        if (!hydrate && theme === 'green' && scheme === 'dark') {
          await page.locator('.site-preview').screenshot({path: `${output}/mantine-button-${width}.png`});
        }
      }
      console.log(`Mantine API browser: ${report.reports.length} cases passed`);
    }
  } catch (error) {report.errors.push(error.stack || String(error));}
  finally {
    if (browser) await browser.close();
    fs.writeFileSync(`${output}/mantine-api-browser.json`, JSON.stringify(report, null, 2));
    console.log(JSON.stringify({reports: report.reports.length, errors: report.errors}, null, 2));
    if (report.errors.length) process.exitCode = 1;
  }
})();
