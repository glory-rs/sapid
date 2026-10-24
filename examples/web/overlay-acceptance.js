async (hostPage) => {
  const context = await hostPage.context().browser().newContext({ viewport: { width: 1100, height: 850 } });
  const reports = [], errors = [];
  const assert = (value, message) => { if (!value) throw new Error(message); };
  try {
    const page = await context.newPage();
    page.on('pageerror', e => errors.push(e.message));
    page.on('console', e => { if (e.type() === 'error') errors.push(e.text()); });
    const tick = () => page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))));
    const trigger = name => page.getByRole('button', { name, exact: true });
    const panel = async name => page.locator(`#${await trigger(name).getAttribute('aria-controls')}`);
    const opened = async p => page.waitForFunction(id => document.getElementById(id)?.matches(':popover-open'), await p.getAttribute('id'));
    const closed = async p => page.waitForFunction(id => !document.getElementById(id)?.matches(':popover-open'), await p.getAttribute('id'));
    for (const mode of ['overlays.html', 'overlays-hydrate.html']) {
      await page.goto(`http://127.0.0.1:8794/${mode}`);
      await page.waitForFunction(() => window.ready === true);
      const dialog = await panel('Open dialog');
      const dialogTrigger = trigger('Open dialog');
      await dialogTrigger.click(); await opened(dialog);
      await page.waitForFunction(() => document.activeElement?.matches('.dialog-input'));
      assert(await dialog.getAttribute('aria-modal') === 'true', 'missing dialog modality');
      const cdp = await context.newCDPSession(page);
      const ax = await cdp.send('Accessibility.getFullAXTree');
      assert(ax.nodes.some(node => node.role?.value === 'dialog' && node.name?.value === 'Account settings' && node.description?.value === 'Edit the account' && node.properties?.some(p => p.name === 'modal' && p.value.value === true)), 'dialog accessible name/description/modal missing');
      await cdp.detach();
      assert(await page.locator('#foreign').getAttribute('inert') !== null, 'modal did not inert sibling app content');
      const input = page.locator('.dialog-input');
      await input.press('Shift+Tab');
      assert(await dialog.locator(':scope > .sapid-layer-close').evaluate(e => document.activeElement === e), 'Shift+Tab did not loop');
      await page.keyboard.press('Tab');
      assert(await input.evaluate(e => document.activeElement === e), 'Tab did not loop');
      await page.locator('#foreign').evaluate(e => e.focus());
      assert(await input.evaluate(e => document.activeElement === e), 'program focus escaped modal');

      const nested = await panel('Nested popover');
      const nestedTrigger = trigger('Nested popover');
      await nestedTrigger.click(); await opened(nested);
      await page.waitForFunction(() => document.activeElement?.matches('.nested-action'));
      await page.keyboard.press('Escape'); await closed(nested); await tick();
      assert(await dialog.evaluate(e => e.matches(':popover-open')), 'nested Escape also closed parent');
      assert(await nestedTrigger.evaluate(e => document.activeElement === e), 'nested focus did not restore');
      const child = await panel('Nested dialog');
      await trigger('Nested dialog').click(); await opened(child);
      await page.mouse.click(5, 5); await closed(child); await tick();
      assert(await dialog.evaluate(e => e.matches(':popover-open')), 'outside pointer closed two layers');
      await page.keyboard.press('Escape'); await closed(dialog); await tick();
      assert(await dialogTrigger.evaluate(e => document.activeElement === e), 'dialog focus did not restore');
      assert(await page.locator('#foreign').getAttribute('inert') === null, 'inert leaked after close');

      const drawer = await panel('Open drawer');
      await trigger('Open drawer').click(); await opened(drawer);
      await page.waitForFunction(() => document.activeElement?.matches('.drawer-action'));
      const bounds = await drawer.boundingBox();
      assert(bounds.x + bounds.width > 1098 && bounds.y === 0, 'drawer is not pinned to viewport edge');
      await page.keyboard.press('Escape'); await closed(drawer);

      const popover = await panel('Open popover');
      await trigger('Open popover').click(); await opened(popover);
      const popBounds = await popover.boundingBox(), anchor = await trigger('Open popover').boundingBox();
      assert(Math.abs(popBounds.x-anchor.x) < 2 && popBounds.y > anchor.y, 'popover anchor placement failed');
      const draft = page.locator('.popover-input');
      await draft.fill('retained🙂'); await tick();
      await draft.evaluate(e => window.savedLayerInput = e);
      await page.locator('#foreign').click(); await closed(popover); await tick();
      assert(await page.locator('#foreign').evaluate(e => document.activeElement === e), 'nonmodal outside click stole focus');
      await trigger('Open popover').click(); await opened(popover);
      assert(await draft.evaluate(e => e === window.savedLayerInput && e.value === 'retained🙂'), 'kept layer lost identity/value');
      const widthBefore = (await popover.boundingBox()).x;
      await page.setViewportSize({ width: 480, height: 700 }); await tick();
      const smaller = await popover.boundingBox();
      assert(smaller.x >= 0 && smaller.x + smaller.width <= 480, 'resize placed popover outside viewport');
      assert(Number.isFinite(widthBefore), 'invalid placement');
      await page.keyboard.press('Escape'); await closed(popover);
      await page.setViewportSize({ width: 1100, height: 850 });

      const empty = await panel('Empty dialog');
      await trigger('Empty dialog').click(); await opened(empty);
      assert(await empty.evaluate(e => document.activeElement === e), 'empty scope fallback missing');
      await page.keyboard.press('Tab');
      assert(await empty.evaluate(e => document.activeElement === e), 'empty scope let Tab escape');
      await page.keyboard.press('Escape'); await closed(empty);

      const initial = await panel('Initial focus');
      await trigger('Initial focus').click(); await opened(initial);
      await page.waitForFunction(() => document.activeElement?.id === 'preferred-focus');
      const composing = await page.locator('.preferred').evaluate(e => {
        const key = new KeyboardEvent('keydown', { key: 'Escape', isComposing: true, bubbles: true, cancelable: true });
        e.dispatchEvent(key); return key.defaultPrevented;
      });
      assert(!composing && await initial.evaluate(e => e.matches(':popover-open')), 'composition Escape dismissed');
      await page.locator('.preferred').evaluate(e => {
        e.dispatchEvent(new CompositionEvent('compositionstart', { bubbles: true }));
        e.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
      });
      await tick();
      assert(await initial.evaluate(e => e.matches(':popover-open')), 'active composition without key flag dismissed');
      await page.locator('.preferred').evaluate(e => e.dispatchEvent(new CompositionEvent('compositionend', { bubbles: true })));
      await page.keyboard.press('Escape'); await closed(initial);

      const controlled = await panel('Controlled dialog');
      await page.evaluate(() => window.demo.overlay_open(true)); await opened(controlled);
      await page.keyboard.press('Escape'); await tick();
      assert(await controlled.evaluate(e => e.matches(':popover-open')), 'controlled rejection physically closed the layer');
      assert(await page.locator('.layer-status').textContent() === 'proposals:1', 'controlled Escape proposal missing');
      await page.mouse.click(5, 5); await tick();
      assert(await page.locator('.layer-status').textContent() === 'proposals:2', 'controlled outside proposal missing');
      await page.evaluate(() => window.demo.overlay_open(false)); await closed(controlled);
      await page.evaluate(() => window.demo.overlay_open(true)); await opened(controlled);
      await page.evaluate(() => window.demo.unmount());
      assert(await page.locator('[inert]').count() === 0 && await page.locator(':popover-open').count() === 0, 'unmount leaked top-layer/inert state');
      reports.push({ suite: mode, status: 'passed', checks: 'nested Escape/outside, modal Tab/inert/focus restore, nonmodal outside, anchored resize/clipping, controlled rejection, empty/initial focus, IME guard, retained input, cleanup' });
    }
    await page.goto('http://127.0.0.1:8794/overlays-hydrate.html?manual');
    await page.waitForFunction(() => window.moduleReady === true);
    await page.locator('.popover-input').evaluate(e => { window.preHydrationLayer = e; e.value = 'before hydration🙂'; });
    await page.evaluate(() => window.demo.mount_overlays(true));
    await trigger('Open popover').click();
    await page.waitForFunction(() => document.querySelector('.sapid-popover:popover-open'));
    assert(await page.locator('.popover-input').evaluate(e => e === window.preHydrationLayer && e.value === 'before hydration🙂'), 'hydration replaced retained panel input');
    reports.push({ suite: 'overlays-hydrate-draft', status: 'passed' });
    await page.goto('http://127.0.0.1:8794/overlays-initial-hydrate.html?manual');
    await page.waitForFunction(() => window.moduleReady === true);
    const initialPanel = await panel('Controlled dialog');
    assert(await initialPanel.isHidden(), 'SSR open panel appeared before modal host activation');
    await initialPanel.evaluate(e => { window.ssrLayer = e; e.classList.add('invalid-hydration'); });
    const rejected = await page.evaluate(() => { try { window.demo.mount_overlays_initial(true); return false; } catch { return true; } });
    assert(rejected, 'mismatched hydration was accepted');
    assert(await page.locator(':popover-open,[inert]').count() === 0, 'failed adoption activated a modal');
    await initialPanel.evaluate(e => e.classList.remove('invalid-hydration'));
    await page.evaluate(() => window.demo.mount_overlays_initial(true));
    await opened(initialPanel);
    assert(await initialPanel.evaluate(e => e === window.ssrLayer && e.contains(document.activeElement)), 'initial open adoption replaced panel or lost focus');
    await page.evaluate(() => window.demo.overlay_open(false)); await closed(initialPanel);
    reports.push({ suite: 'overlays-initial-hydrate', status: 'passed', checks: 'initial open state, no modal side effects on failed adoption, identity and focus after successful retry' });
    assert(errors.length === 0, JSON.stringify(errors));
    return { reports, errors };
  } finally { await context.close(); }
}
