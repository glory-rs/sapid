// Run after check.ps1 -Web. Optional --jobs=1..4 uses isolated browsers.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { chromium } = require(process.env.SAPID_PLAYWRIGHT || 'playwright');
const suites = ['trace-acceptance.js', 'stack-trace-acceptance.js', 'acceptance.js', 'composite-acceptance.js', 'accordion-dynamic-acceptance.js', 'tabs-dynamic-acceptance.js', 'collapse-motion-acceptance.js', 'overflow-list-acceptance.js', 'table-of-contents-acceptance.js', 'table-acceptance.js', 'data-table-acceptance.js', 'data-table-server-acceptance.js', 'code-acceptance.js', 'json-viewer-acceptance.js', 'carousel-acceptance.js', 'rich-editor-acceptance.js', 'rich-editor-stream-acceptance.js', 'rich-editor-extra-acceptance.js', 'files-acceptance.js', 'files-stream-acceptance.js', 'copy-button-acceptance.js', 'floating-window-acceptance.js', 'tour-acceptance.js', 'tree-acceptance.js', 'tree-choices-acceptance.js', 'charts-acceptance.js', 'plots-acceptance.js', 'lightbox-acceptance.js', 'chat-evidence-acceptance.js', 'output-acceptance.js', 'tasks-acceptance.js', 'confirmation-acceptance.js', 'json-tree-acceptance.js', 'code-copy-acceptance.js', 'translation-acceptance.js', 'text-motion-acceptance.js', 'floating-indicator-acceptance.js', 'overlay-acceptance.js', 'popover-placement-acceptance.js', 'layer-document-acceptance.js', 'portal-acceptance.js', 'focus-scope-acceptance.js', 'focus-scope-stream-acceptance.js', 'gallery-acceptance.js', 'messages-acceptance.js', 'message-scroller-acceptance.js', 'prompt-acceptance.js', 'questionnaire-acceptance.js', 'model-suggestions-acceptance.js', 'environment-acceptance.js', 'responsive-acceptance.js', 'environment-stream-acceptance.js', 'field-acceptance.js', 'input-acceptance.js', 'choice-acceptance.js', 'form-acceptance.js', 'display-acceptance.js', 'document-acceptance.js', 'document-stream-acceptance.js', 'toggle-acceptance.js', 'layout-acceptance.js', 'typography-acceptance.js', 'timeline-acceptance.js', 'spoiler-acceptance.js', 'navigation-acceptance.js', 'grid-acceptance.js', 'visually-hidden-acceptance.js', 'stepper-acceptance.js', 'menu-acceptance.js', 'context-menu-acceptance.js', 'command-acceptance.js', 'menubar-acceptance.js', 'avatar-acceptance.js', 'shell-acceptance.js', 'hints-acceptance.js', 'image-acceptance.js', 'buttons-acceptance.js', 'burger-acceptance.js', 'scroll-area-acceptance.js', 'alerts-acceptance.js', 'modals-acceptance.js', 'notifications-acceptance.js', 'toolbar-acceptance.js', 'color-swatch-acceptance.js', 'color-sliders-acceptance.js', 'color-picker-acceptance.js', 'sheet-acceptance.js', 'drawer-acceptance.js', 'loading-overlay-acceptance.js', 'graphics-acceptance.js', 'native-select-acceptance.js', 'select-acceptance.js', 'combobox-acceptance.js', 'choice-extras-acceptance.js', 'tags-acceptance.js', 'progress-acceptance.js', 'spinner-acceptance.js', 'navigation-progress-acceptance.js', 'splitter-acceptance.js', 'affix-acceptance.js', 'navigation-menu-acceptance.js', 'slider-acceptance.js', 'range-slider-acceptance.js', 'angle-slider-acceptance.js', 'rating-acceptance.js', 'number-input-acceptance.js', 'pin-input-acceptance.js', 'mask-input-acceptance.js', 'calendar-acceptance.js', 'date-picker-acceptance.js', 'date-input-acceptance.js', 'period-picker-acceptance.js', 'time-input-acceptance.js', 'time-picker-acceptance.js', 'date-time-picker-acceptance.js', 'schedule-acceptance.js', 'schedule-timeline-acceptance.js', 'schedule-calendar-acceptance.js', 'resource-schedule-acceptance.js', 'json-input-acceptance.js'];
const output = 'D:/BuildCache/sapid';
const sourceRoot = path.join(__dirname, '../examples/web');
const fixtureRoot = path.join(output, 'web');
const jobsArg = process.argv.slice(2).find(arg => arg.startsWith('--jobs='));
const jobs = Number(jobsArg?.slice('--jobs='.length) || 1);
if (!Number.isInteger(jobs) || jobs < 1 || jobs > 4) throw new Error('--jobs must be an integer in 1..4');
process.env.TEMP = process.env.TMP = `${output}/tmp`;
fs.mkdirSync(process.env.TEMP, { recursive: true });
const hash = value => crypto.createHash('sha256').update(value).digest('hex');
function fixtureHashes(directory = fixtureRoot, prefix = '') {
  const result = {};
  for (const entry of fs.readdirSync(directory, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    const file = path.join(directory, entry.name), relative = `${prefix}${entry.name}`;
    if (entry.isDirectory()) Object.assign(result, fixtureHashes(file, `${relative}/`));
    else if (entry.isFile()) result[relative] = hash(fs.readFileSync(file));
  }
  return result;
}
// Clipboard is shared by the desktop even when browser contexts are isolated.
let clipboardTail = Promise.resolve();
async function withClipboardLock(run) {
  const before = clipboardTail;
  let release;
  clipboardTail = new Promise(resolve => { release = resolve; });
  await before;
  try { return await run(); } finally { release(); }
}
(async () => {
  const sourceHashes = Object.fromEntries(suites.map(file => [file, hash(fs.readFileSync(path.join(sourceRoot, file)))]));
  const artifactHashes = fixtureHashes();
  const results = new Array(suites.length), versions = new Set(), runErrors = [];
  let cursor = 0;
  function snapshot(complete) {
    const done = results.filter(Boolean);
    return { browser: [...versions].join(', '), jobs, complete,
      completedSuites: done.map(result => result.file), reports: done.flatMap(result => result.reports),
      errors: [...runErrors, ...done.flatMap(result => result.errors)], sourceHashes, artifactHashes };
  }
  function save(name, value) {
    const destination = path.join(output, name), temporary = `${destination}.tmp`;
    fs.writeFileSync(temporary, JSON.stringify(value, null, 2));
    fs.renameSync(temporary, destination);
  }
  async function worker(workerId) {
    let browser, processed = 0, diagnostics = [];
    async function launchBrowser() {
      const browser = await chromium.launch({ headless: true, executablePath: process.env.SAPID_CHROME });
      const newContext = browser.newContext.bind(browser);
      browser.newContext = async (...args) => {
        const context = await newContext(...args);
        context.on('page', page => {
          const record = value => { if (diagnostics.length < 100) diagnostics.push(value); };
          page.on('pageerror', error => record({ type: 'pageerror', url: page.url(), message: error.message }));
          page.on('requestfailed', request => record({ type: 'requestfailed', url: request.url(), message: request.failure()?.errorText }));
          page.on('response', response => { if (response.status() >= 400) record({ type: 'response', url: response.url(), status: response.status() }); });
        });
        return context;
      };
      return browser;
    }
    try {
      browser = await launchBrowser();
      versions.add(browser.version());
      while (cursor < suites.length) {
        const index = cursor++, file = suites[index];
        if (processed && processed % 16 === 0) {
          await browser.close(); browser = await launchBrowser();
          versions.add(browser.version());
        }
        const source = fs.readFileSync(path.join(sourceRoot, file), 'utf8');
        diagnostics = [];
        const run = async () => {
          console.log(`Worker ${workerId}: ${index + 1}/${suites.length} ${file}`);
          const page = await browser.newPage();
          try {
            if (hash(source) !== sourceHashes[file]) throw new Error('acceptance source changed during this run');
            const acceptance = new Function(`return (${source})`)();
            const result = await acceptance(page);
            return { file, reports: result.reports, errors: result.errors.map(error => `${file}: ${error}`) };
          } catch (error) {
            return { file, reports: [], errors: [`${file} at ${page.url()}: ${error.stack || error}\nBrowser diagnostics: ${JSON.stringify(diagnostics)}`] };
          } finally { await page.close(); }
        };
        results[index] = await (/clipboard|copy/i.test(source) ? withClipboardLock(run) : run());
        processed++;
        save('browser-progress.json', snapshot(false));
      }
    } finally { if (browser) await browser.close(); }
  }
  await Promise.all(Array.from({ length: jobs }, (_, index) => worker(index + 1)));
  if (versions.size !== 1) runErrors.push('Browser version changed between workers');
  for (const file of suites) {
    if (hash(fs.readFileSync(path.join(sourceRoot, file))) !== sourceHashes[file]) runErrors.push(`${file}: source changed during this run`);
  }
  if (JSON.stringify(fixtureHashes()) !== JSON.stringify(artifactHashes)) runErrors.push('Fixture assets changed during this run');
  const result = snapshot(true);
  save('browser-results.json', result);
  console.log(JSON.stringify({ browser: result.browser, jobs, suites: result.completedSuites.length, reports: result.reports.length, errors: result.errors }, null, 2));
  if (result.errors.length) process.exitCode = 1;
  const browser = await chromium.launch({ headless: true, executablePath: process.env.SAPID_CHROME });
  try {
    const page = await browser.newPage();
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
  } finally { await browser.close(); }
})().catch(error => { console.error(error); process.exitCode = 1; });
