const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const digest = value => crypto.createHash('sha256').update(value).digest('hex');
const output = process.env.SAPID_STUDIO_TIMING_OUTPUT || 'D:/BuildCache/sapid/theme-studio-frame-regression';

// Runs automatically from theme-studio-browser.cjs. For a focused real-WASM
// run, set SAPID_SHOWCASE_DIRECTORY to the built showcase directory and run
// node --test scripts/test_theme_studio_browser.cjs (same Playwright env).
async function verifyPreviewCommitBoundary(browser, base) {
  const source = fs.readFileSync(path.join(__dirname, 'theme-studio-browser.cjs'), 'utf8');
  const start = source.indexOf('  const go=async');
  const end = source.indexOf("  await button('Edit Dark colors')");
  assert(start >= 0 && end > start, 'Exercise an exact source slice of the real driver');
  const scenario = source.slice(start, end);
  const barrier = /  \/\/ BEGIN PREVIEW COMMIT BARRIER\r?\n[\s\S]*?  \/\/ END PREVIEW COMMIT BARRIER\r?\n/;
  assert(barrier.test(scenario), 'The negative control removes only the explicit readiness barrier');
  const priorScenario = scenario.replace(barrier, '');
  const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
  const results = [];
  const evidence = { mechanismReproduction: true, physicalCISchedulingReproduced: false,
    driverSha256: digest(source), helperSha256: digest(fs.readFileSync(__filename)), scenarioSha256: digest(scenario), results };
  try {
  for (const { rate, ready } of [{ rate: 1, ready: false }, { rate: 1, ready: true }, { rate: 8, ready: true }]) {
    const context = await browser.newContext({ viewport: { width: 1440, height: 1000 } });
    try {
      const page = await context.newPage();
      const cdp = await context.newCDPSession(page);
      await cdp.send('Emulation.setCPUThrottlingRate', { rate });
      await page.addInitScript(() => {
        window.__previewTiming = { heldFrames: 0, release: null, inputEvents: [] };
        const nativeFrame = window.requestAnimationFrame.bind(window);
        const nativeCancel = window.cancelAnimationFrame.bind(window);
        const pending = new Map();
        let delaying = false;
        const release = reason => {
          if (!delaying) return;
          delaying = false;
          window.__previewTiming.release = reason;
          window.__previewTiming.heldFrames = pending.size;
          const callbacks = [...pending]; pending.clear();
          callbacks.forEach(([id, callback]) => { nativeCancel(id); callback(performance.now()); });
        };
        window.requestAnimationFrame = callback => {
          const id = nativeFrame(at => {
            if (!pending.has(id) || delaying) return;
            pending.delete(id); callback(at);
          });
          pending.set(id, callback);
          return id;
        };
        window.cancelAnimationFrame = id => { pending.delete(id); nativeCancel(id); };
        document.addEventListener('click', event => {
          if (event.target.closest('button')?.getAttribute('aria-label') !== 'Preview light') return;
          // Test-host scheduling only: deliver this pending mode turn between
          // the old driver's focus/select and insertText protocol operations.
          // A readiness-aware driver waits for the delayed turn instead.
          delaying = true;
          setTimeout(() => release('delayed-frame'), 250);
        }, true);
        document.addEventListener('focusin', event => {
          if (delaying && event.target.getAttribute('aria-label') === 'Preview name') {
            queueMicrotask(() => release('focus-before-insert'));
          }
        }, true);
        document.addEventListener('input', event => {
          if (event.target.getAttribute('aria-label') === 'Preview name') window.__previewTiming.inputEvents.push(event.target.value);
        }, true);
      });
      const checks = [], errors = [];
      page.on('pageerror', error => errors.push(error.message));
      let failure;
      try {
        await new AsyncFunction('page', 'base', 'assert', 'ok', ready ? scenario : priorScenario)(page, base, assert, check => checks.push(check));
      } catch (error) { failure = error.message; }
      const timing = await page.evaluate(() => window.__previewTiming);
      results.push({ rate, ready, checks, failure, timing, pageErrors: errors });
      assert.deepEqual(errors, []);
      assert(timing.heldFrames > 0, 'Exercise a pending host turn');
      if (ready) {
        assert.equal(failure, undefined);
        assert.equal(checks.length, 4, 'Keep every original retained-name and click-count assertion');
        assert.equal(timing.release, 'delayed-frame');
        assert.deepEqual(timing.inputEvents, ['Retained sample']);
      } else {
        assert.equal(checks.length, 3);
        assert.equal(failure, 'Preview name should become Retained sample');
        assert.equal(timing.release, 'focus-before-insert');
        assert.deepEqual(timing.inputEvents, [], 'Observe a lost native insertion, not rejected model state');
      }
    } finally { await context.close(); }
  }
  } catch (error) { error.previewTiming = evidence; throw error; }
  return evidence;
}

module.exports = { verifyPreviewCommitBoundary };
if (require.main === module) {
  const test = require('node:test');
  const { chromium } = require(process.env.SAPID_PLAYWRIGHT || 'playwright');
  const { serve } = require('./showcase-acceptance.cjs');
  test('sample editing waits for the committed mode rather than an editor being retired', { timeout: 120000 }, async () => {
    const relative = path.relative(path.resolve('D:/BuildCache/sapid'), path.resolve(output));
    assert(!relative.startsWith('..') && !path.isAbsolute(relative), 'Timing output stays under D:/BuildCache/sapid');
    fs.mkdirSync(output, { recursive: true });
    process.env.TEMP = process.env.TMP = path.join(output, 'tmp');
    fs.mkdirSync(process.env.TEMP, { recursive: true });
    const directory = process.env.SAPID_SHOWCASE_DIRECTORY || 'D:/BuildCache/sapid/showcase';
    const server = await serve(directory);
    let browser;
    try {
      browser = await chromium.launch({ headless: true, executablePath: process.env.SAPID_CHROME });
      const result = await verifyPreviewCommitBoundary(browser, server.base);
      fs.writeFileSync(path.join(output, 'results.json'), JSON.stringify({ directory, ...result }, null, 2));
    } catch (error) {
      fs.writeFileSync(path.join(output, 'results.json'), JSON.stringify({ directory, ...error.previewTiming, error: error.stack }, null, 2));
      throw error;
    } finally { await browser?.close(); await server.close(); }
  });
}
