async (hostPage) => {
  const context=await hostPage.context().browser().newContext({viewport:{width:1440,height:1000}});
  const reports=[],errors=[];
  const assert=(value,message)=>{if(!value)throw new Error(message);};
  try {
    const page=await context.newPage();
    page.on('pageerror',e=>errors.push(e.message));
    page.on('console',e=>{if(e.type()==='error')errors.push(e.text());});
    const tick=()=>page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydration of [false,true]) {
      const mode=`${hue}-${scheme}-${hydration?'hydrate':'csr'}`;
      await page.goto(`http://127.0.0.1:8794/${hydration?`fields-hydrate-${hue}-${scheme}.html`:`fields.html?theme=${hue}&scheme=${scheme}`}`);
      await page.waitForFunction(()=>window.ready===true);
      const input=page.locator('.field-email');
      assert(await page.getByRole('textbox',{name:'Email address',exact:true}).count()===1,`${mode}: label name association failed`);
      const meta=await input.evaluate(e=>({id:e.id,help:e.getAttribute('aria-describedby'),label:e.labels[0]?.textContent,required:e.required,bg:getComputedStyle(e).backgroundColor,fg:getComputedStyle(e).color}));
      assert(meta.id&&meta.help&&meta.required&&meta.label==='Email address',`${mode}: missing native field metadata`);
      assert(meta.bg===(scheme==='dark'?'rgb(37, 38, 43)':'rgb(255, 255, 255)'),`${mode}: Rust input theme differs from page`);
      await page.getByText('Email address',{exact:true}).click();
      assert(await input.evaluate(e=>document.activeElement===e),`${mode}: label click did not focus`);
      await input.fill('hello@studio.design'); await tick();
      await input.evaluate(e=>{window.savedField=e;e.setSelectionRange(2,7,'backward');});
      await page.evaluate(()=>window.demo.field_state('error',true));
      assert(await input.evaluate(e=>e===window.savedField&&e.selectionStart===2&&e.selectionEnd===7&&e.selectionDirection==='backward'),`${mode}: error update reset input/caret`);
      const errorId=await input.getAttribute('aria-errormessage');
      assert(errorId&&await input.getAttribute('aria-invalid')==='true',`${mode}: error state missing`);
      assert((await input.getAttribute('aria-describedby')).split(' ').length===2,`${mode}: error replaced help description`);
      assert(await page.locator(`#${errorId}`).isVisible(),`${mode}: error not visible`);
      const cdp=await context.newCDPSession(page); const ax=await cdp.send('Accessibility.getFullAXTree'); await cdp.detach();
      assert(ax.nodes.some(n=>n.role?.value==='textbox'&&n.name?.value==='Email address'&&n.description?.value.includes('work email')&&n.properties?.some(p=>p.name==='required'&&p.value.value===true)),`${mode}: AX field name/description/required missing`);
      await page.evaluate(()=>window.demo.field_state('readonly',true));
      assert(await input.evaluate(e=>e.readOnly),`${mode}: readonly not native`);
      await input.focus(); await page.keyboard.press('Control+a'); await page.keyboard.type('must not edit'); await tick();
      assert(await input.inputValue()==='hello@studio.design',`${mode}: readonly text changed`);
      const check=page.locator('.field-consent input'); const checked=await check.isChecked();
      await check.focus(); await page.keyboard.press('Space'); await tick();
      assert(await check.isChecked()===checked,`${mode}: readonly check changed by keyboard`);
      await check.click(); await tick(); assert(await check.isChecked()===checked,`${mode}: readonly check changed by pointer`);
      await page.evaluate(()=>window.demo.field_state('required',false));
      assert(await input.evaluate(e=>!e.required&&!e.hasAttribute('aria-required')),`${mode}: required reset failed`);
      await page.evaluate(()=>window.demo.field_state('readonly',false));
      await page.evaluate(()=>window.demo.field_state('error',false));
      assert(await input.getAttribute('aria-invalid')===null&&await input.getAttribute('aria-errormessage')===null,`${mode}: error association leaked`);
      assert(await page.locator(`#${errorId}`).isHidden(),`${mode}: empty error remained accessible`);
      await page.getByText('Public profile',{exact:true}).click(); await tick();
      assert(await page.locator('.fieldset-check input').isChecked(),`${mode}: external label did not toggle checkbox`);
      await page.evaluate(()=>window.demo.field_state('disabled',true));
      for(const selector of ['.fieldset-name','.fieldset-company','.fieldset-check input','.fieldset-button'])assert(await page.locator(selector).isDisabled(),`${mode}: fieldset did not disable ${selector}`);
      await input.focus(); await page.locator('.fieldset-name').evaluate(e=>e.focus());
      assert(await input.evaluate(e=>document.activeElement===e),`${mode}: disabled fieldset focus escaped`);
      await page.evaluate(()=>window.demo.field_state('disabled',false));
      await page.locator('.fieldset-name').fill('Changed profile');
      const standalone=page.locator('.standalone-input'); await page.getByText('Standalone control label',{exact:true}).click();
      assert(await standalone.evaluate(e=>document.activeElement===e),`${mode}: standalone Label failed`);
      // Capture the default gallery at the top; sticky navigation otherwise
      // appears halfway through a full-page image after the interaction checks.
      await page.reload(); await page.waitForFunction(()=>window.ready===true);
      await page.evaluate(()=>scrollTo(0,0)); await tick();
      await page.screenshot({path:`D:/BuildCache/sapid/fields-${hue}-${scheme}.png`,fullPage:true});
      reports.push({suite:mode,status:'passed',checks:'native labels, checkbox label toggle, required/readonly/invalid, help+error AX, caret identity, disabled fieldset, four-theme rendering'});
    }
    await page.goto('http://127.0.0.1:8794/fields-hydrate-green-dark.html?manual');
    await page.waitForFunction(()=>window.moduleReady===true);
    await page.locator('.field-email').fill('before@hydrate.design');
    await page.locator('.field-email').evaluate(e=>{window.preField=e;e.setSelectionRange(1,6,'backward');});
    await page.evaluate(()=>window.demo.mount_fields(true,'green','dark'));
    assert(await page.locator('.field-email').evaluate(e=>e===window.preField&&e.value==='before@hydrate.design'&&e.selectionStart===1&&e.selectionEnd===6), 'field hydration lost draft/identity/selection');
    reports.push({suite:'fields-hydration-draft',status:'passed'});
    await page.goto('http://127.0.0.1:8794/fields.html');await page.waitForFunction(()=>window.ready===true);
    await page.getByRole('button',{name:'Green theme',exact:true}).click();await page.waitForFunction(()=>window.ready===true&&document.documentElement.dataset.hue==='green');
    await page.getByRole('button',{name:'Toggle color scheme',exact:true}).click();await page.waitForFunction(()=>window.ready===true&&document.documentElement.dataset.mode==='dark');
    await page.getByRole('searchbox',{name:'Search components'}).fill('fieldset');
    assert(await page.locator('.sidebar a:visible').count()===1,'showcase search is not functional');
    await page.setViewportSize({width:390,height:844});await tick();
    assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),'mobile gallery overflows horizontally');
    reports.push({suite:'showcase-theme-navigation-mobile',status:'passed'});
    await page.setViewportSize({width:1440,height:1000});
    for(const family of ['index','composites','overlays'])for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydration of [false,true]) {
      const stem=family==='index'?'hydrate':`${family}-hydrate`;
      await page.goto(`http://127.0.0.1:8794/${hydration?`${stem}-${hue}-${scheme}.html`:`${family}.html?theme=${hue}&scheme=${scheme}`}`);
      await page.waitForFunction(()=>window.ready===true);
      if(family==='overlays')await page.getByRole('button',{name:'Open dialog',exact:true}).click();
      const control=page.locator(family==='index'?'.echo':family==='composites'?'.profile-input':'.dialog-input');
      await control.waitFor({state:'visible'});
      assert(await control.evaluate(e=>getComputedStyle(e).backgroundColor)===(scheme==='dark'?'rgb(37, 38, 43)':'rgb(255, 255, 255)'),`${family}/${hue}/${scheme}/${hydration}: gallery control theme mismatch`);
      if(family==='composites')assert(await page.locator('.auto-tabs [role=tab][aria-selected=true]').evaluate(e=>getComputedStyle(e).backgroundColor)===(hue==='green'?'rgb(35, 112, 50)':'rgb(25, 113, 194)'),'selected tab accent differs from theme');
      if(hue==='green'&&scheme==='dark'&&hydration){await page.evaluate(()=>scrollTo(0,0));await page.screenshot({path:`D:/BuildCache/sapid/gallery-${family}-green-dark.png`,fullPage:family!=='overlays'});}
    }
    await page.goto('http://127.0.0.1:8794/composites.html');await page.waitForFunction(()=>window.ready===true);
    await page.getByRole('button',{name:'Green theme',exact:true}).click();await page.waitForFunction(()=>window.ready===true&&document.documentElement.dataset.hue==='green');
    assert(new URL(page.url()).pathname==='/composites.html','changing hue left the current gallery');
    await page.getByRole('button',{name:'Toggle color scheme',exact:true}).click();await page.waitForFunction(()=>window.ready===true&&document.documentElement.dataset.mode==='dark');
    await page.locator('.sidebar a[data-page=fields]').click();await page.waitForFunction(()=>window.ready===true);
    assert(await page.evaluate(()=>document.documentElement.dataset.hue==='green'&&document.documentElement.dataset.mode==='dark'),'sidebar navigation lost theme');
    reports.push({suite:'all-gallery-themes-csr-hydration',status:'passed',checks:'24 themed basic/composite/overlay mounts, current-page theme switching and cross-page theme retention'});
    assert(errors.length===0,JSON.stringify(errors)); return {reports,errors};
  } finally {await context.close();}
}
