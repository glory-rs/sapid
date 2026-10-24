async (hostPage) => {
  const context = await hostPage.context().browser().newContext({ viewport: { width: 1440, height: 1000 } });
  const reports = [], errors = [];
  const assert = (condition, message) => { if (!condition) throw new Error(message); };
  try {
    const page = await context.newPage();
    page.on('pageerror', e => errors.push(e.message));
    page.on('console', e => { if (e.type() === 'error') errors.push(e.text()); });
    const tick = () => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
    for (const hue of ['blue', 'green']) for (const scheme of ['light', 'dark']) for (const hydration of [false, true]) {
      const mode = `document-${hue}-${scheme}-${hydration ? 'hydrate' : 'csr'}`;
      await page.goto(`http://127.0.0.1:8794/${hydration ? `document-hydrate-${hue}-${scheme}.html` : `document.html?theme=${hue}&scheme=${scheme}`}`);
      await page.waitForFunction(() => window.ready === true);
      for (const [selector, tag] of [
        ['.document-code', 'CODE'], ['.sapid-kbd', 'KBD'], ['.sapid-mark', 'MARK'],
        ['.sapid-blockquote', 'BLOCKQUOTE'], ['.sapid-data-list', 'DL'],
        ['.sapid-data-list-term', 'DT'], ['.sapid-data-list-definition', 'DD'],
        ['.sapid-text-list-item', 'LI'],
      ]) assert(await page.locator(selector).first().evaluate(e => e.tagName) === tag, `${mode}: intrinsic ${tag} missing`);
      assert(await page.locator('ol.sapid-text-list').count() === 1 && await page.locator('ul.sapid-text-list').count() === 1, `${mode}: ordered/unordered nesting`);
      assert(await page.getByRole('list', { name: 'Document steps', exact: true }).count() === 1, `${mode}: named intrinsic list`);
      assert(await page.locator('.sapid-text-list-item').count() === 3, `${mode}: list item count`);
      const pairs = await page.locator('.sapid-data-list').evaluate(e => [...e.children].map(e => e.tagName));
      assert(pairs.join(',') === 'DT,DD,DT,DD', `${mode}: transparent data-pair grouping`);
      assert(await page.locator('.sapid-data-list').getAttribute('role') === null, `${mode}: description list overridden by generic role`);
      assert(await page.locator('.sapid-kbd').innerText() === 'Command', `${mode}: explicit platform key`);
      const highlighted = 'Find the Glory text 😀 without changing it.';
      assert(await page.locator('.document-highlight').textContent() === highlighted, `${mode}: source text altered`);
      assert(await page.locator('.document-highlight mark').count() === 2, `${mode}: actual highlighted ranges`);
      assert(await page.locator('.document-highlight > div').count() === 0, `${mode}: inline highlight gained a layout wrapper`);
      assert(await page.locator('.sapid-item').getAttribute('role') === null, `${mode}: passive item gained interaction role`);
      assert(await page.getByRole('button', { name: 'Open repository', exact: true }).count() === 1, `${mode}: separate named item action`);
      await page.locator('.document-code').evaluate(e => { window.documentCode = e; window.documentCodeText = e.firstChild; });
      await page.locator('.document-highlight').evaluate(e => {
        window.highlightMarks = [...e.querySelectorAll('mark')];
        const range = document.createRange(); range.selectNodeContents(e);
        const selection = window.getSelection(); selection.removeAllRanges(); selection.addRange(range);
      });
      assert(await page.evaluate(() => window.getSelection().toString()) === highlighted, `${mode}: cross-mark text selection`);
      await page.getByRole('button', { name: 'Update code source', exact: true }).evaluate(e => e.click()); await tick();
      assert(await page.locator('.document-code').evaluate(e => e === window.documentCode && e.firstChild === window.documentCodeText && e.textContent === 'let sapid = 42;'), `${mode}: reactive text identity`);
      assert(await page.locator('.document-highlight').evaluate(e => [...e.querySelectorAll('mark')].every((mark, i) => mark === window.highlightMarks[i])), `${mode}: unrelated update rebuilt marks`);
      assert(await page.evaluate(() => window.getSelection().toString()) === highlighted, `${mode}: unrelated text update replaced highlight selection`);
      const cdp = await context.newCDPSession(page);
      const ax = await cdp.send('Accessibility.getFullAXTree'); await cdp.detach();
      const roles = ax.nodes.map(n => String(n.role?.value || '').toLowerCase());
      for (const role of ['list', 'listitem', 'term', 'definition']) assert(roles.includes(role), `${mode}: AX ${role} missing`);
      assert(ax.nodes.some(n => n.role?.value === 'button' && n.name?.value === 'Open repository'), `${mode}: AX action name`);
      assert(await page.locator('.sapid-mark').first().evaluate(e => getComputedStyle(e).backgroundColor) !== 'rgba(0, 0, 0, 0)', `${mode}: mark theme`);
      await page.setViewportSize({ width: 390, height: 844 }); await tick();
      assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1), `${mode}: mobile overflow`);
      await page.setViewportSize({ width: 1440, height: 1000 });
      if (hydration) await page.screenshot({ path: `D:/BuildCache/sapid/${mode}.png`, fullPage: true });
      await page.evaluate(() => window.demo.unmount_display());
      assert(await page.locator('#app').evaluate(e => e.childElementCount === 0), `${mode}: teardown`);
      reports.push({ suite: mode, status: 'passed', checks: 'intrinsic tags/list nesting/dt-dd relations, reactive text identity, exact Unicode highlight and cross-mark selection, passive Item/actions, explicit Kbd platform, AX roles, theme/mobile, teardown' });
    }
    assert(errors.length === 0, JSON.stringify(errors));
    return { reports, errors };
  } finally { await context.close(); }
}
