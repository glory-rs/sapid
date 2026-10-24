async (hostPage) => {
  const context = await hostPage.context().browser().newContext({ viewport: { width: 1100, height: 850 } });
  const reports = [];
  const assert = (value, message) => { if (!value) throw new Error(message); };
  try {
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    for (const mode of ['index.html', 'hydrate.html']) {
      await page.goto(`http://127.0.0.1:8794/${mode}`);
      await page.waitForFunction(() => window.ready === true);
      const layout = await page.locator('.demo').evaluate(element => {
        const style = getComputedStyle(element);
        const fields = [...element.querySelectorAll('.echo,.locked,.local')].map(field => field.getBoundingClientRect());
        return { display: style.display, direction: style.flexDirection,
          ordered: fields[0].bottom <= fields[1].top && fields[1].bottom <= fields[2].top,
          width: fields[0].width };
      });
      assert(layout.display === 'flex' && layout.direction === 'column' && layout.ordered && layout.width > 200,
        `demo layout lost its control boxes: ${JSON.stringify(layout)}`);
      const echo = page.locator('.echo');
      await echo.fill('typed🙂');
      await page.waitForFunction(() => document.querySelector('.counts').textContent.includes('changes:1'));
      assert(await echo.inputValue() === 'typed🙂', 'controlled input did not accept edit');
      await echo.evaluate(element => {
        window.savedInput = element;
        element.value = 'A🙂B';
        element.setSelectionRange(1, 3, 'backward');
        element.dispatchEvent(new InputEvent('input', { bubbles: true, data: '🙂' }));
      });
      await page.waitForFunction(() => document.querySelector('.counts').textContent.includes('changes:2'));
      assert(await echo.evaluate(element => element === window.savedInput && element.selectionStart === 1 && element.selectionEnd === 3 && element.selectionDirection === 'backward'), 'accepted echo reset identity/selection');
      await page.evaluate(() => window.demo.set_value('A🙂B'));
      assert(await echo.evaluate(element => element.selectionStart === 1 && element.selectionEnd === 3), 'equal program echo reset selection');
      await echo.evaluate(element => {
        for (const value of ['one', 'two', 'three']) {
          element.value = value;
          element.dispatchEvent(new InputEvent('input', { bubbles: true }));
        }
      });
      // Web coalesces uninterrupted native input to its final observation.
      // The Rust queued-host test separately checks non-coalesced same-tick edits.
      await page.waitForFunction(() => document.querySelector('.counts').textContent.includes('changes:3'));
      assert(await echo.inputValue() === 'three', 'host batch lost a later edit');
      await echo.press('Enter');
      await page.waitForFunction(() => document.querySelector('.counts').textContent.includes('commits:1'));

      await page.locator('.locked').fill('rejected');
      await page.waitForFunction(() => document.querySelector('.locked').value === 'locked');
      await page.locator('.accept-check input').check();
      await page.waitForFunction(() => document.querySelector('.check-state').textContent === 'checked:true');
      await page.locator('.reject-check input').click();
      await page.waitForFunction(() => !document.querySelector('.reject-check input').checked);
      await page.locator('.local-check input').check();
      await page.locator('.local-check input').focus();
      await page.keyboard.press('Space');
      await page.waitForFunction(() => !document.querySelector('.local-check input').checked);
      await page.locator('.local').fill('local\nnotes');
      assert(await page.locator('.local').inputValue() === 'local\nnotes', 'multiline local edit failed');

      await page.locator('.action').click();
      await page.waitForFunction(() => document.querySelector('.counts').textContent.includes('presses:1'));
      await page.evaluate(() => window.demo.set_loading(true));
      assert(await page.locator('.action').isDisabled(), 'loading failed to disable native button');
      await page.evaluate(() => window.demo.set_loading(false));
      await page.locator('.action').focus();
      await page.keyboard.press('Enter');
      await page.waitForFunction(() => document.querySelector('.counts').textContent.includes('presses:2'));

      // Synthetic composition validates the browser protocol, not OS IME.
      await echo.focus();
      await echo.evaluate(element => {
        element.dispatchEvent(new CompositionEvent('compositionstart', { bubbles: true }));
        element.value = '中';
        element.setSelectionRange(1, 1);
        element.dispatchEvent(new InputEvent('input', { bubbles: true, data: '中', isComposing: true }));
      });
      await page.waitForFunction(() => document.querySelector('.counts').textContent.includes('changes:4'));
      assert(await echo.inputValue() === '中', 'composition echo changed the draft');
      await page.evaluate(() => window.demo.set_value('program'));
      assert(await echo.inputValue() === '中', 'program write interrupted composition');
      await echo.evaluate(element => element.dispatchEvent(new CompositionEvent('compositionend', { bubbles: true, data: '中' })));
      await page.waitForFunction(() => document.querySelector('.echo').value === 'program');
      assert(await echo.evaluate(element => element === window.savedInput), 'input identity changed');
      await page.evaluate(() => window.demo.unmount());
      assert(await page.locator('#app').evaluate(element => element.childNodes.length === 0), 'unmount retained nodes');
      reports.push({ mode, result: 'passed' });
    }

    await page.goto('http://127.0.0.1:8794/hydrate.html?manual');
    await page.waitForFunction(() => window.moduleReady === true);
    await page.locator('.echo').evaluate(element => {
      window.beforeHydration = element;
      element.focus();
      element.value = 'pre-hydration🙂';
      element.setSelectionRange(2, 7, 'backward');
    });
    await page.locator('.accept-check input').evaluate(element => { window.beforeCheck = element; element.checked = true; });
    await page.evaluate(() => window.demo.mount(true));
    await page.waitForFunction(() => document.querySelector('.check-state').textContent === 'checked:true');
    assert(await page.locator('.echo').evaluate(element => element === window.beforeHydration && element.value === 'pre-hydration🙂' && element.selectionStart === 2 && element.selectionEnd === 7), 'hydration reset existing draft/selection');
    assert(await page.locator('.accept-check input').evaluate(element => element === window.beforeCheck && element.checked), 'hydration replaced checkbox or lost checked draft');
    reports.push({ mode: 'pre-hydration drafts', result: 'passed' });
    assert(errors.length === 0, `browser errors: ${errors.join('; ')}`);
    return { reports, errors };
  } catch (error) {
    const page = context.pages()[0];
    const state = page ? await page.evaluate(() => ({
      url: location.href,
      counts: document.querySelector('.counts')?.textContent,
      value: document.querySelector('.echo')?.value,
      checked: document.querySelector('.check-state')?.textContent,
    })).catch(() => ({ url: page.url(), navigation: true })) : null;
    throw new Error(`${error.message}\nState: ${JSON.stringify(state)}`, { cause: error });
  } finally {
    await context.close();
  }
}
