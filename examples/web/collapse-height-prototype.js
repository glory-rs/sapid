async (hostPage) => {
  const context = await hostPage.context().browser().newContext({ reducedMotion: 'no-preference' });
  try {
    const page = await context.newPage();
    await page.goto('http://127.0.0.1:8794/actions.html');
    await page.waitForFunction(() => window.ready === true);
    const base = await page.locator('head style').allTextContents();
    const css = base.join('');
    const assert = (value, message) => { if (!value) throw new Error(message); };
    assert(css.includes('[data-gly-hidden]{display:none!important}'), 'actual BASE_CSS missing');
    await page.evaluate(() => window.demo.unmount_display());
    await page.setContent(`<style>${css}
      [data-proto-collapse]{display:block!important;height:auto;overflow:hidden;interpolate-size:allow-keywords;transition:height 500ms linear}
      [data-proto-collapse][data-gly-hidden]{display:block!important;height:0}
      @media(prefers-reduced-motion:reduce){[data-proto-collapse]{transition:none!important}}
      </style><div id="panel" data-gly-kind="view" data-proto-collapse hidden data-gly-hidden aria-hidden="true" inert>
      <div style="height:120px"><input aria-label="Prototype editor"></div></div>`);
    const panel = page.locator('#panel');
    const height = () => panel.evaluate(element => element.getBoundingClientRect().height);
    const toggle = closed => panel.evaluate((element, closed) => {
      for (const name of ['hidden', 'data-gly-hidden', 'inert']) element.toggleAttribute(name, closed);
      if (closed) element.setAttribute('aria-hidden', 'true'); else element.removeAttribute('aria-hidden');
    }, closed);
    const report = { supported: await page.evaluate(() => CSS.supports('interpolate-size', 'allow-keywords')), samples: {} };
    assert(report.supported, 'intrinsic interpolation unavailable');
    report.samples.closed = await height();
    assert(report.samples.closed === 0, 'semantic hidden initial endpoint');
    await toggle(false); await height(); await page.waitForTimeout(100);
    report.samples.enter = await height();
    assert(report.samples.enter > 0 && report.samples.enter < 120, 'no intrinsic enter interpolation');
    await page.waitForFunction(() => document.querySelector('#panel').getAnimations().length === 0);
    assert(await height() === 120, 'open endpoint');
    await toggle(true); await height(); await page.waitForTimeout(100);
    report.samples.exit = await height();
    assert(report.samples.exit > 0 && report.samples.exit < 120, 'hidden/display truncated exit');
    const cdp = await context.newCDPSession(page);
    const ax = await cdp.send('Accessibility.getFullAXTree');
    assert(!ax.nodes.some(node => !node.ignored && node.name?.value === 'Prototype editor'), 'closing content remains AX available');
    await page.locator('input').evaluate(element => element.focus());
    assert(await page.evaluate(() => document.activeElement.tagName !== 'INPUT'), 'closing content remains focusable');
    report.samples.beforeReverse = await height();
    await toggle(false); report.samples.reverseStart = await height(); await page.waitForTimeout(35);
    report.samples.reverse = await height();
    assert(Math.abs(report.samples.reverseStart - report.samples.beforeReverse) < 12, 'reverse jumped geometry');
    assert(report.samples.reverse > report.samples.reverseStart && report.samples.reverse < 120, 'reverse did not advance continuously');
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await page.waitForFunction(() => getComputedStyle(document.querySelector('#panel')).transitionProperty === 'none');
    report.samples.reduced = await height();
    assert(report.samples.reduced === 120, 'in-flight Reduce failed to reach endpoint');
    await page.emulateMedia({ reducedMotion: 'no-preference' });
    await toggle(true); await height(); await page.waitForTimeout(100);
    report.samples.beforeClear = await height();
    assert(report.samples.beforeClear > 0 && report.samples.beforeClear < 120, 'no exit transition to clear');
    await panel.evaluate(element => element.removeAttribute('data-proto-collapse'));
    report.samples.clear = await height();
    assert(report.samples.clear === 0 && await panel.evaluate(element => element.getAnimations().length === 0), 'Clear did not restore hidden endpoint and cancel motion');
    report.status = 'passed';
    report.scope = 'browser prototype with actual Web16 BASE_CSS only; shared facet, component and Unmount completion protocol remain unimplemented';
    return report;
  } finally { await context.close(); }
}
