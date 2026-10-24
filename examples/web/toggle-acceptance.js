async (hostPage) => {
  const context=await hostPage.context().browser().newContext({viewport:{width:1100,height:900}});
  const reports=[],errors=[];
  const assert=(value,message)=>{if(!value)throw new Error(message);};
  try {
    const page=await context.newPage();
    page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text());});
    const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    const pressed=locator=>locator.getAttribute('aria-pressed');
    const changes=async()=>Number((await page.locator('.toggle-status').textContent()).match(/changes=(\d+)/)[1]);
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydration of [false,true]) {
      const mode=`toggles-${hue}-${scheme}-${hydration?'hydrate':'csr'}`;
      await page.goto(`http://127.0.0.1:8794/${hydration?`toggles-hydrate-${hue}-${scheme}.html`:`toggles.html?theme=${hue}&scheme=${scheme}`}`);
      await page.waitForFunction(()=>window.ready===true);
      const bold=page.getByRole('button',{name:'Bold text',exact:true}),left=page.getByRole('button',{name:'Align left',exact:true}),center=page.getByRole('button',{name:'Align center',exact:true}),right=page.getByRole('button',{name:'Align right',exact:true});
      const italic=page.getByRole('button',{name:'Italic text',exact:true}),underline=page.getByRole('button',{name:'Underline text',exact:true});
      assert(await page.getByRole('group',{name:'Text alignment',exact:true}).count()===1,`${mode}: group name missing`);
      assert(await bold.getAttribute('type')==='button'&&await pressed(bold)==='false',`${mode}: actual button/initial pressed missing`);
      await bold.click();await tick();assert(await pressed(bold)==='true',`${mode}: pointer toggle failed`);
      await page.waitForFunction(expected=>getComputedStyle(document.querySelector('.toggle-bold')).backgroundColor===expected,hue==='green'?'rgb(35, 112, 50)':'rgb(25, 113, 194)');
      assert(await bold.evaluate(e=>getComputedStyle(e).backgroundColor)===(hue==='green'?'rgb(35, 112, 50)':'rgb(25, 113, 194)'),`${mode}: pressed theme missing`);
      await bold.focus();await page.keyboard.press('Space');await tick();assert(await pressed(bold)==='false',`${mode}: Space toggle failed`);
      await page.keyboard.press('Enter');await tick();assert(await pressed(bold)==='true',`${mode}: Enter toggle failed`);
      await left.focus();await page.keyboard.press('ArrowRight');await tick();
      assert(await center.evaluate(e=>document.activeElement===e),`${mode}: roving did not skip disabled item`);
      assert(await pressed(left)==='true'&&await pressed(center)==='false',`${mode}: arrows changed single value`);
      assert(await center.getAttribute('tabindex')==='0'&&await left.getAttribute('tabindex')==='-1',`${mode}: roving tab stop did not follow actual focus`);
      await page.keyboard.press('Space');await tick();assert(await pressed(center)==='true'&&await pressed(left)==='false',`${mode}: single selection failed`);
      await page.keyboard.press('Space');await tick();assert(await pressed(center)==='false',`${mode}: selected toggle did not clear`);
      await page.keyboard.press('End');await tick();assert(await right.evaluate(e=>document.activeElement===e),`${mode}: End navigation failed`);
      await page.keyboard.press('Home');await tick();assert(await left.evaluate(e=>document.activeElement===e),`${mode}: Home navigation failed`);
      await page.keyboard.press('ArrowLeft');await tick();assert(await right.evaluate(e=>document.activeElement===e),`${mode}: reverse loop navigation failed`);
      await page.keyboard.press('ArrowRight');await tick();assert(await left.evaluate(e=>document.activeElement===e),`${mode}: loop navigation failed`);
      await italic.focus();await page.keyboard.press('ArrowDown');await tick();
      assert(await underline.evaluate(e=>document.activeElement===e),`${mode}: vertical navigation failed`);
      assert(await pressed(underline)==='false',`${mode}: focus changed multiple value`);
      await page.keyboard.press('Space');await tick();await italic.click();await tick();
      assert(await pressed(underline)==='true'&&await pressed(italic)==='true',`${mode}: multiple values not independent`);
      assert(await page.getByRole('button',{name:'Inherited style',exact:true}).isDisabled()&&await pressed(page.getByRole('button',{name:'Inherited style',exact:true}))==='true',`${mode}: disabled selection was lost`);
      await page.evaluate(()=>window.demo.toggle_state('reject',true));
      await bold.click();await center.click();await tick();
      assert(await pressed(bold)==='true'&&await pressed(center)==='false',`${mode}: controlled rejection changed presented values`);
      await page.evaluate(()=>window.demo.toggle_state('reject',false));
      const beforeExternal=await changes();await right.evaluate(e=>window.savedToggle=e);
      await page.evaluate(()=>window.demo.toggle_value('single','right'));await tick();
      assert(await right.evaluate(e=>e===window.savedToggle)&&await pressed(right)==='true',`${mode}: external selection replaced member`);
      assert(await changes()===beforeExternal,`${mode}: external update emitted change callback`);
      await page.evaluate(()=>window.demo.toggle_state('disabled',true));
      for(const selector of ['.toggle-bold','.toggle-left','.toggle-center','.toggle-right','.toggle-italic','.toggle-underline'])assert(await page.locator(selector).isDisabled(),`${mode}: ${selector} not disabled`);
      await bold.evaluate(e=>e.click());await tick();assert(await changes()===beforeExternal,`${mode}: disabled button activated`);
      await page.evaluate(()=>window.demo.toggle_state('disabled',false));
      const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();
      assert(ax.nodes.some(n=>n.name?.value==='Bold text'&&n.properties?.some(p=>p.name==='pressed'&&p.value.value==='true')),`${mode}: AX pressed state missing`);
      await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});
      reports.push({suite:mode,status:'passed',checks:'native pressed button, pointer/Enter/Space, single clear, multiple values, roving arrows/Home/End/disabled skip, external silence/identity, controlled rejection, four-theme style, AX pressed'});
    }
    await page.goto('http://127.0.0.1:8794/toggles-hydrate-green-dark.html?manual');await page.waitForFunction(()=>window.moduleReady===true);
    await page.locator('.toggle-bold').evaluate(e=>{window.preToggle=e;e.focus();});
    await page.evaluate(()=>window.demo.mount_toggles(true,'green','dark'));await tick();
    assert(await page.locator('.toggle-bold').evaluate(e=>e===window.preToggle&&document.activeElement===e),'toggle hydration replaced focused button');
    await page.evaluate(()=>window.demo.unmount_toggles());await tick();assert(await page.locator('.sapid-toggle').count()===0,'toggle Owner retirement left mounted buttons');
    reports.push({suite:'toggles-hydration-retirement',status:'passed'});
    return {reports,errors};
  } finally {await context.close();}
}
