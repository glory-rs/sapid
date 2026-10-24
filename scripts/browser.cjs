// Run after check.ps1 -Web -Serve. All browser output/profiles belong on D:.
const fs = require('node:fs');
const path = require('node:path');
const { chromium } = require(process.env.SAPID_PLAYWRIGHT || 'playwright');

(async () => {
  const output = 'D:/BuildCache/sapid';
  process.env.TEMP = process.env.TMP = `${output}/tmp`;
  fs.mkdirSync(process.env.TEMP, { recursive: true });
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage();
    const reports = [], errors = [], completedSuites = [];
    for (const file of ['acceptance.js', 'composite-acceptance.js', 'overlay-acceptance.js', 'field-acceptance.js', 'input-acceptance.js', 'choice-acceptance.js', 'form-acceptance.js', 'display-acceptance.js', 'document-acceptance.js', 'document-stream-acceptance.js', 'toggle-acceptance.js', 'layout-acceptance.js', 'typography-acceptance.js', 'timeline-acceptance.js', 'spoiler-acceptance.js', 'navigation-acceptance.js', 'grid-acceptance.js', 'visually-hidden-acceptance.js', 'stepper-acceptance.js', 'menu-acceptance.js', 'avatar-acceptance.js', 'shell-acceptance.js', 'hints-acceptance.js', 'image-acceptance.js', 'buttons-acceptance.js', 'scroll-area-acceptance.js', 'alerts-acceptance.js', 'modals-acceptance.js', 'notifications-acceptance.js', 'toolbar-acceptance.js', 'color-swatch-acceptance.js', 'sheet-acceptance.js', 'loading-overlay-acceptance.js', 'graphics-acceptance.js', 'native-select-acceptance.js', 'select-acceptance.js', 'combobox-acceptance.js', 'choice-extras-acceptance.js', 'progress-acceptance.js', 'spinner-acceptance.js', 'navigation-progress-acceptance.js', 'splitter-acceptance.js', 'affix-acceptance.js', 'navigation-menu-acceptance.js', 'slider-acceptance.js']) {
      console.log(`Running ${file}`);
      const source = fs.readFileSync(path.join(__dirname, '../examples/web', file), 'utf8');
      const acceptance = new Function(`return (${source})`)();
      const result = await acceptance(page);
      reports.push(...result.reports); errors.push(...result.errors);
      completedSuites.push(file);
      fs.writeFileSync(`${output}/browser-progress.json`, JSON.stringify({browser: browser.version(), complete:false, completedSuites, reports, errors}, null, 2));
    }
    const result = { browser: browser.version(), complete:true, completedSuites, reports, errors };
    fs.writeFileSync(`${output}/browser-results.json`, JSON.stringify(result, null, 2));
    if(errors.length) process.exitCode=1;
    await page.goto('http://127.0.0.1:8794/');
    await page.waitForFunction(() => window.ready === true);
    await page.screenshot({ path: `${output}/widgets.png`, fullPage: true });
    await page.goto('http://127.0.0.1:8794/composites.html');
    await page.waitForFunction(() => window.ready === true);
    await page.screenshot({ path: `${output}/composites.png`, fullPage: true });
    await page.goto('http://127.0.0.1:8794/overlays.html');
    await page.waitForFunction(() => window.ready === true);
    await page.getByRole('button', { name: 'Open dialog', exact: true }).click();
    await page.waitForFunction(() => document.activeElement?.matches('.dialog-input'));
    await page.screenshot({ path: `${output}/overlays.png` });
    console.log(JSON.stringify(result, null, 2));
  } finally {
    await browser.close();
  }
})().catch(error => {
  console.error(error);
  process.exitCode = 1;
});
