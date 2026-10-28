async(hostPage)=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1440,height:1000}});
  const reports=[],errors=[];const assert=(value,message)=>{if(!value)throw new Error(message)};
  try{
    const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
    const tick=()=>page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
    for(const family of ['display','actions'])for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydration of [false,true]){
      const mode=`${family}-${hue}-${scheme}-${hydration?'hydrate':'csr'}`;
      await page.goto(`http://127.0.0.1:8794/${hydration?`${family}-hydrate-${hue}-${scheme}.html`:`${family}.html?theme=${hue}&scheme=${scheme}`}`);
      await page.waitForFunction(()=>window.ready===true);
      if(family==='display'){
        assert(await page.locator('.display-fixture').getByRole('heading',{name:'Display widgets',exact:true}).count()===1,`${mode}: card heading`);
        assert((await page.locator('.number-us').innerText()).trim()==='1,234.50',`${mode}: US number`);
        assert((await page.locator('.number-eur').innerText()).includes('1.234,50'),`${mode}: EU currency`);
        assert((await page.locator('.number-percent').innerText()).includes('12.5'),`${mode}: percentage`);
        assert(await page.locator('.sapid-paper').evaluate(e=>getComputedStyle(e).boxShadow.includes('12px')),`${mode}: typed paper shadow`);
        await page.getByRole('button',{name:'Remove local tag',exact:true}).click();await tick();
        assert(await page.locator('.local-pill').isHidden(),`${mode}: local removal`);
        assert(await page.locator('.sapid-paper').evaluate(e=>getComputedStyle(e).boxShadow)!=='none',`${mode}: typed surface shadow`);
        const controlled=page.locator('.controlled-pill');await controlled.evaluate(e=>window.savedPill=e);
        assert(await controlled.evaluate(e=>getComputedStyle(e).display==='inline-flex'&&e.getBoundingClientRect().width<e.parentElement.getBoundingClientRect().width),`${mode}: pill fits content`);
        await page.getByRole('button',{name:'Request controlled removal',exact:true}).focus();await page.keyboard.press('Space');await tick();
        assert(await controlled.isVisible()&&(await page.locator('.display-proposals').innerText()).includes('1'),`${mode}: controlled proposal`);
        await page.getByRole('button',{name:'Toggle controlled visibility',exact:true}).click();await tick();assert(await controlled.isHidden(),`${mode}: source hide`);
        await page.getByRole('button',{name:'Toggle controlled visibility',exact:true}).click();await tick();
        assert(await controlled.evaluate(e=>e===window.savedPill),`${mode}: shell identity`);
        assert(await page.getByRole('button',{name:'Remove disabled tag',exact:true}).isDisabled(),`${mode}: disabled removal`);
        assert(await page.getByRole('heading',{name:'No messages',exact:true}).count()===1,`${mode}: empty state`);
        assert(await page.locator('.sapid-badge').first().evaluate(e=>getComputedStyle(e).backgroundColor)!=='rgba(0, 0, 0, 0)',`${mode}: badge theme`);
      }else{
        const action=page.getByRole('button',{name:'Refresh preview',exact:true});
        assert(await action.count()===1&&await page.getByRole('button',{name:'Dismiss preview',exact:true}).count()===1,`${mode}: icon names`);
        const local=page.locator('.local-burger');await local.evaluate(e=>window.savedBurger=e);
        await local.click();await tick();assert(await local.getAttribute('aria-expanded')==='true',`${mode}: pointer expand`);
        await local.focus();await page.keyboard.press('Space');await tick();assert(await local.getAttribute('aria-expanded')==='false',`${mode}: keyboard collapse`);
        const controlled=page.locator('.controlled-burger');await controlled.click();await tick();
        assert(await controlled.getAttribute('aria-expanded')==='false'&&(await page.locator('.action-proposals').innerText()).includes('1'),`${mode}: controlled refusal`);
        await page.getByRole('button',{name:'Toggle disclosure source',exact:true}).click();await tick();assert(await controlled.getAttribute('aria-expanded')==='true',`${mode}: external disclosure`);
        const first=page.getByRole('button',{name:'Group first',exact:true}),last=page.getByRole('button',{name:'Group last',exact:true});
        assert(await page.getByRole('group',{name:'Horizontal actions',exact:true}).count()===1,`${mode}: group role`);
        await first.focus();await page.keyboard.press('Tab');assert(await last.evaluate(e=>document.activeElement===e),`${mode}: native Tab skipping disabled`);
        const borders=await page.getByRole('group',{name:'Horizontal actions',exact:true}).evaluate(e=>[...e.querySelectorAll('button')].map(e=>({left:getComputedStyle(e).borderLeftWidth,right:getComputedStyle(e).borderRightWidth,radius:getComputedStyle(e).borderRadius})));
        assert(borders.length===3&&borders[1].left==='0px'&&borders[1].radius==='0px',`${mode}: joined borders`);
        await page.getByRole('button',{name:'Toggle group enabled',exact:true}).click();await tick();assert(await first.isDisabled()&&await last.isDisabled(),`${mode}: group disable`);
        await page.getByRole('button',{name:'Toggle group enabled',exact:true}).click();await tick();assert(await first.isEnabled()&&await page.getByRole('button',{name:'Group disabled',exact:true}).isDisabled(),`${mode}: child disabled preserved`);
        const toolbar=page.getByRole('toolbar',{name:'RTL editing tools',exact:true}),tools=toolbar.locator('button');
        assert(await tools.evaluateAll(es=>es.filter(e=>e.tabIndex===0).length)===1&&await toolbar.getAttribute('aria-orientation')==='horizontal',`${mode}: actual toolbar and one tab entry`);
        await tools.first().focus();await page.keyboard.press('ArrowRight');await tick();assert(await tools.last().evaluate(e=>e===document.activeElement),`${mode}: RTL wraps and skips disabled`);
        await page.keyboard.press('ArrowLeft');await tick();assert(await tools.first().evaluate(e=>e===document.activeElement),`${mode}: RTL reverse`);
        await page.keyboard.press('End');await tick();assert(await tools.last().evaluate(e=>e===document.activeElement),`${mode}: toolbar End`);await page.keyboard.press('Home');await tick();assert(await tools.first().evaluate(e=>e===document.activeElement),`${mode}: toolbar Home`);
        assert((await page.locator('.toolbar-status').innerText()).endsWith('0'),`${mode}: arrows do not activate`);await page.keyboard.press('Space');await tick();assert((await page.locator('.toolbar-status').innerText()).endsWith('1'),`${mode}: toolbar native activation`);
        await page.keyboard.press('Tab');await tick();const above=page.getByRole('button',{name:'Toolbar above',exact:true}),below=page.getByRole('button',{name:'Toolbar below',exact:true});assert(await above.evaluate(e=>e===document.activeElement),`${mode}: Tab exits horizontal toolbar`);await page.keyboard.press('ArrowUp');await tick();assert(await above.evaluate(e=>e===document.activeElement),`${mode}: non-looping edge`);await page.keyboard.press('ArrowDown');await tick();assert(await below.evaluate(e=>e===document.activeElement),`${mode}: vertical navigation`);
        assert(await local.evaluate(e=>e===window.savedBurger),`${mode}: disclosure identity`);
      }
      const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();
      assert(ax.nodes.some(n=>n.role?.value==='button'&&n.name?.value===(family==='display'?'Compose':'Refresh preview')),`${mode}: AX action name`);
      await page.setViewportSize({width:390,height:844});await tick();
      assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${mode}: mobile overflow`);
      await page.setViewportSize({width:1440,height:1000});
      if(hydration){await page.screenshot({path:`D:/BuildCache/sapid/${family}-${hue}-${scheme}.png`,fullPage:true})}
      await page.evaluate(()=>window.demo.unmount_display());assert(await page.locator('#app').evaluate(e=>e.childElementCount===0),`${mode}: teardown`);
      reports.push({suite:mode,status:'passed'});
    }
    assert(errors.length===0,JSON.stringify(errors));return {reports,errors};
  }finally{await context.close()}
}
