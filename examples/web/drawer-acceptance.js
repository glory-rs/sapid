async (hostPage) => {
  const context = await hostPage.context().browser().newContext({ viewport: { width: 1100, height: 850 } });
  const reports = [], errors = [];
  const assert = (value, message) => { if (!value) throw new Error(message); };
  try {
    const page = await context.newPage();
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    for (const hue of ['blue', 'green']) for (const scheme of ['light', 'dark']) for (const hydrate of [false, true]) {
      const mode = `drawer-${hue}-${scheme}-${hydrate ? 'hydrate' : 'csr'}`;
      await page.goto(`http://127.0.0.1:8794/${hydrate ? `overlays-hydrate-${hue}-${scheme}.html` : `overlays.html?theme=${hue}&scheme=${scheme}`}`);
      await page.waitForFunction(() => window.ready === true);
      const trigger = page.getByRole('button', { name: 'Open drawer', exact: true });
      const id = await trigger.getAttribute('aria-controls');
      const panel = page.locator(`#${id}`), destination = panel.locator('.drawer-action');
      await panel.evaluate(element => window.savedDrawerPanel = element);
      await trigger.click();
      await page.waitForFunction(id => document.getElementById(id)?.matches(':popover-open'), id);
      await page.waitForFunction(() => document.activeElement?.matches('.drawer-action'));
      assert(await panel.getAttribute('data-gly-sheet') === null, `${mode}: baseline unexpectedly opted into gestures`);
      assert(await panel.getAttribute('aria-modal') === 'true' && await trigger.getAttribute('aria-expanded') === 'true', `${mode}: modal/expanded state`);
      const box = await panel.boundingBox();
      assert(Math.abs(box.x + box.width - 1100) < 1 && box.y === 0, `${mode}: right viewport edge`);
      assert(await page.locator('#foreign').getAttribute('inert') !== null, `${mode}: modal background not inert`);
      const cdp = await context.newCDPSession(page);
      const ax = await cdp.send('Accessibility.getFullAXTree');
      assert(ax.nodes.some(node => !node.ignored && node.role?.value === 'dialog' && node.name?.value === 'Navigation' && node.properties?.some(property => property.name === 'modal' && property.value.value === true)), `${mode}: real Chrome dialog AX`);
      await cdp.detach();
      await destination.press('Shift+Tab');
      assert(await panel.locator(':scope > .sapid-layer-close').evaluate(element => document.activeElement === element), `${mode}: reverse Tab cycle or disabled skipping`);
      await page.keyboard.press('Tab');
      assert(await destination.evaluate(element => document.activeElement === element), `${mode}: forward Tab cycle`);
      await page.locator('#foreign').evaluate(element => element.focus());
      assert(await destination.evaluate(element => document.activeElement === element), `${mode}: focus escaped modal`);
      await page.keyboard.press('Escape');
      await page.waitForFunction(id => !document.getElementById(id)?.matches(':popover-open'), id);
      await page.waitForFunction(() => document.activeElement?.textContent === 'Open drawer');
      assert(await trigger.getAttribute('aria-expanded') === 'false' && await page.locator('#foreign').getAttribute('inert') === null, `${mode}: Escape state/focus/inert cleanup`);
      assert(await destination.count() === 0, `${mode}: default Unmount retained content`);
      await trigger.click();
      await page.waitForFunction(id => document.getElementById(id)?.matches(':popover-open'), id);
      assert(await panel.evaluate(element => element === window.savedDrawerPanel), `${mode}: reopen replaced panel identity`);
      await page.mouse.click(5, 5);
      await page.waitForFunction(id => !document.getElementById(id)?.matches(':popover-open'), id);
      await trigger.click();
      await page.waitForFunction(id => document.getElementById(id)?.matches(':popover-open'), id);
      await page.evaluate(() => window.demo.unmount());
      assert(await page.locator(':popover-open').count() === 0 && await page.locator('[inert]').count() === 0 && await page.locator('#app').evaluate(element => element.childNodes.length === 0), `${mode}: Owner retirement leaked layer/inert/content`);
      reports.push({ suite: mode, status: 'passed', checks: 'ordinary non-snap Drawer, real right-edge geometry, Chrome dialog AX and modality, disabled skipping and Tab cycle, focus containment/return, Escape/outside, stable panel and default Unmount, Owner cleanup' });
    }
    assert(errors.length === 0, JSON.stringify(errors));
    return { reports, errors };
  } finally { await context.close(); }
}
