async (hostPage) => {
  const context = await hostPage.context().browser().newContext({ viewport: { width: 1100, height: 1000 } });
  const reports = [];
  const errors = [];
  const assert = (value, message) => { if (!value) throw new Error(message); };
  try {
    const page = await context.newPage();
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    const frame = () => page.evaluate(() => new Promise(resolve => requestAnimationFrame(resolve)));
    for (const mode of ['composites.html', 'composites-hydrate.html']) {
      await page.goto(`http://127.0.0.1:8794/${mode}`);
      await page.waitForFunction(() => window.ready === true);
      const disclosure = page.locator('.demo-disclosure .sapid-collapsible-trigger');
      const details = page.locator('.details-input');
      assert(await details.isHidden(), 'closed kept content is visible');
      await disclosure.focus(); await disclosure.press('Enter');
      await page.waitForFunction(() => document.querySelector('.demo-disclosure button').getAttribute('aria-expanded') === 'true');
      assert(await disclosure.evaluate(element => document.activeElement === element), 'opening stole trigger focus');
      await details.fill('retained🙂');
      await frame();
      await details.evaluate(element => { window.savedDetails = element; element.setSelectionRange(1, 5, 'backward'); });
      await page.evaluate(() => window.demo.composite_open(false));
      await page.waitForFunction(() => document.activeElement === document.querySelector('.demo-disclosure button'));
      assert(await details.isHidden(), 'closing did not hide the panel');
      await disclosure.press('Space');
      await page.waitForFunction(() => !document.querySelector('.demo-disclosure [data-gly-hidden]'));
      assert(await details.evaluate(element => element === window.savedDetails && element.value === 'retained🙂'), 'kept content lost identity/draft');

      const headers = page.locator('.demo-accordion .sapid-accordion-trigger');
      await headers.nth(0).focus(); await headers.nth(0).press('ArrowDown');
      await page.waitForFunction(() => document.activeElement === document.querySelectorAll('.demo-accordion button')[2]);
      assert(await headers.nth(0).getAttribute('aria-expanded') === 'true', 'arrow navigation changed expansion');
      await headers.nth(2).press('Home');
      await page.waitForFunction(() => document.activeElement === document.querySelectorAll('.demo-accordion button')[0]);
      await headers.nth(0).press('Space');
      await page.waitForFunction(() => document.querySelectorAll('.demo-accordion button')[0].getAttribute('aria-expanded') === 'false');
      await headers.nth(0).press('Tab');
      await page.waitForFunction(() => document.activeElement === document.querySelectorAll('.demo-accordion button')[2]);
      await headers.nth(2).press('Enter');
      await page.waitForFunction(() => document.querySelectorAll('.demo-accordion button')[2].getAttribute('aria-expanded') === 'true');
      assert(await headers.nth(1).isDisabled(), 'disabled accordion member became interactive');

      const automatic = page.locator('.auto-tabs .sapid-tab');
      await page.locator('.profile-input').evaluate(element => { window.savedProfile = element; });
      await automatic.nth(0).focus(); await automatic.nth(0).press('ArrowRight');
      await page.waitForFunction(() => document.querySelectorAll('.auto-tabs [role=tab]')[2].getAttribute('aria-selected') === 'true');
      assert(await automatic.nth(2).evaluate(element => element === document.activeElement), 'automatic navigation did not focus destination');
      await automatic.nth(2).press('ArrowDown');
      assert(await automatic.nth(2).evaluate(element => element === document.activeElement), 'horizontal tabs captured perpendicular arrow');
      await automatic.nth(2).press('Home');
      await page.waitForFunction(() => document.querySelectorAll('.auto-tabs [role=tab]')[0].getAttribute('aria-selected') === 'true');
      await page.locator('.profile-input').fill('profile🙂'); await frame();
      await page.evaluate(() => window.demo.composite_value('c'));
      await page.waitForFunction(() => document.activeElement === document.querySelectorAll('.auto-tabs [role=tab]')[2]);
      await page.evaluate(() => window.demo.composite_value('a'));
      await page.waitForFunction(() => document.querySelectorAll('.auto-tabs [role=tab]')[0].getAttribute('aria-selected') === 'true');
      assert(await page.locator('.profile-input').evaluate(element => element === window.savedProfile && element.value === 'profile🙂'), 'tab switching reset kept input');
      await page.locator('.profile-input').focus(); await frame();
      await page.evaluate(() => window.demo.composite_value('b'));
      await page.waitForFunction(() => document.activeElement === document.querySelectorAll('.auto-tabs [role=tabpanel]')[1]);
      await frame();
      assert((await page.locator('.composite-status').textContent()).includes('selected:b'), 'focus restoration undid controlled selection of a disabled tab');
      await page.evaluate(() => window.demo.composite_value('a')); await frame();
      const before = await automatic.nth(0).getAttribute('aria-controls');
      await page.evaluate(() => window.demo.composite_disabled(true));
      assert(await automatic.nth(0).isDisabled(), 'dynamic disabled state did not reach the native button');
      assert(await automatic.nth(2).getAttribute('tabindex') === '0', 'roving focus did not skip disabled selected tab');
      assert(await automatic.nth(0).getAttribute('aria-controls') === before, 'disabled patch changed relations');
      await page.evaluate(() => window.demo.composite_disabled(false));

      const manual = page.locator('.manual-tabs .sapid-tab');
      await manual.nth(0).focus(); await manual.nth(0).press('ArrowRight');
      await page.waitForFunction(() => document.activeElement === document.querySelectorAll('.manual-tabs [role=tab]')[1]);
      await page.waitForFunction(() => document.querySelectorAll('.manual-tabs [role=tab]')[1].getAttribute('tabindex') === '0');
      assert(await manual.nth(0).getAttribute('aria-selected') === 'true', 'manual navigation selected a panel');
      assert(await manual.nth(1).getAttribute('tabindex') === '0', 'manual roving tab stop did not follow focus');
      await manual.nth(1).press('Tab');
      await page.waitForFunction(() => document.activeElement === document.querySelectorAll('.manual-tabs [role=tabpanel]')[0]);
      await manual.nth(1).focus(); await manual.nth(1).press('Enter');
      await page.waitForFunction(() => document.querySelectorAll('.manual-tabs [role=tab]')[1].getAttribute('aria-selected') === 'true');

      const vertical = page.locator('.vertical-tabs .sapid-tab');
      await vertical.nth(0).focus(); await vertical.nth(0).press('ArrowDown');
      await page.waitForFunction(() => document.querySelectorAll('.vertical-tabs [role=tab]')[1].getAttribute('aria-selected') === 'true');
      await vertical.nth(1).press('ArrowDown'); await frame();
      assert(await vertical.nth(1).evaluate(element => element === document.activeElement), 'non-looping navigation wrapped');
      const guarded = await vertical.nth(1).evaluate(element => {
        const event = new KeyboardEvent('keydown', { key: 'Home', bubbles: true, cancelable: true, isComposing: true });
        element.dispatchEvent(event); return event.defaultPrevented;
      });
      assert(!guarded, 'collection captured composing navigation');

      const readonly = page.locator('.readonly-tabs .sapid-tab');
      await readonly.nth(1).focus(); await frame();
      await readonly.nth(1).press('Space'); await frame();
      assert(await readonly.nth(0).getAttribute('aria-selected') === 'true', 'controlled rejection switched the panel');
      assert(await readonly.nth(1).evaluate(element => element === document.activeElement), 'controlled rejection stole focus');

      const relations = await page.evaluate(() => {
        const ids = [...document.querySelectorAll('#app [id]')].map(element => element.id);
        const refs = [...document.querySelectorAll('#app [aria-controls],#app [aria-labelledby]')].flatMap(element =>
          [element.getAttribute('aria-controls'), element.getAttribute('aria-labelledby')].filter(Boolean));
        return { unique: new Set(ids).size === ids.length, resolved: refs.every(id => document.getElementById(id)), count: refs.length };
      });
      assert(relations.unique && relations.resolved && relations.count > 20, 'relations became dangling or aliased');
      const ax = await context.newCDPSession(page);
      const tree = await ax.send('Accessibility.getFullAXTree');
      assert(tree.nodes.some(node => node.role?.value === 'tablist' && node.name?.value === 'Account'), 'AX tab list name missing');
      assert(tree.nodes.some(node => node.role?.value === 'tab' && node.properties?.some(prop => prop.name === 'selected' && prop.value.value === true)), 'AX selected tab missing');
      assert(tree.nodes.some(node => node.role?.value === 'button' && node.name?.value === 'Details' && node.properties?.some(prop => prop.name === 'expanded' && prop.value.value === true)), 'AX disclosure state missing');
      await ax.detach();
      await page.evaluate(() => window.demo.unmount());
      assert(await page.locator('#app').evaluate(element => element.childNodes.length === 0), 'composite teardown retained DOM');
      reports.push({ mode, result: 'passed' });
    }

    await page.goto('http://127.0.0.1:8794/composites-hydrate.html?manual');
    await page.waitForFunction(() => window.moduleReady === true);
    await page.locator('.profile-input').evaluate(element => {
      window.adoptedProfile = element; element.focus(); element.value = 'draft before hydration🙂'; element.setSelectionRange(2, 7, 'backward');
    });
    await page.evaluate(() => window.demo.mount_composites(true));
    await frame();
    assert(await page.locator('.profile-input').evaluate(element => element === window.adoptedProfile && element.value === 'draft before hydration🙂' && element.selectionStart === 2 && element.selectionEnd === 7), 'composite adoption replaced draft or selection');
    reports.push({ mode: 'composite pre-hydration draft', result: 'passed' });
    assert(errors.length === 0, errors.join('; '));
    return { reports, errors };
  } catch (error) {
    const page = context.pages()[0];
    const state = page ? await page.evaluate(() => ({ url: location.href, active: document.activeElement?.outerHTML,
      tabs: [...document.querySelectorAll('[role=tab]')].map(element => [element.textContent, element.getAttribute('aria-selected'), element.getAttribute('tabindex')]) })) : null;
    throw new Error(`${error.message}\nState: ${JSON.stringify(state)}`, { cause: error });
  } finally { await context.close(); }
}
