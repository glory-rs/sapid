async (hostPage, baseURL='http://127.0.0.1:8794') => {
  const context=await hostPage.context().browser().newContext({viewport:{width:1100,height:900}}),reports=[],errors=[];
  const assert=(condition,message)=>{if(!condition)throw new Error(message);};
  try {
    const page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));page.on('console',message=>{if(message.type()==='error')errors.push(message.text());});
    const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    const settle=async(ms=200)=>{await page.waitForTimeout(ms);await tick();};
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]) {
      const mode=`hints-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
      await page.goto(`${baseURL}/${hydrate?`hints-hydrate-${hue}-${scheme}.html`:'hints.html'}?theme=${hue}&scheme=${scheme}`);await page.waitForFunction(()=>window.ready===true);
      await page.mouse.move(1080,850);await settle(180);
      const save=page.getByRole('button',{name:'Save draft',exact:true}),about=page.getByRole('button',{name:'About Ada',exact:true}),outside=page.getByRole('button',{name:'Outside hints',exact:true});
      const tip=page.locator('#'+await save.getAttribute('aria-describedby')),card=page.getByRole('dialog',{name:'Ada profile',exact:true});
      assert(await tip.getAttribute('role')==='tooltip'&&!await tip.isVisible()&&await save.getAttribute('aria-haspopup')===null&&await save.getAttribute('aria-expanded')===null,`${mode}: real description relation without popup/expanded`);
      await page.evaluate(()=>{window.hintEnteredAt=null;window.hintShownAt=null;const target=document.querySelector('[id*=save-tip-target]'),panel=document.getElementById(target.getAttribute('aria-describedby'));const enter=()=>{if(window.hintEnteredAt===null)window.hintEnteredAt=performance.now();};target.addEventListener('pointerover',enter,{once:true});target.addEventListener('pointermove',enter,{once:true});const observer=new MutationObserver(()=>{if(!panel.hidden){window.hintShownAt=performance.now();observer.disconnect();}});observer.observe(panel,{attributes:true,attributeFilter:['hidden']});});
      await outside.focus();await save.hover();await page.waitForFunction(()=>window.hintShownAt!==null);assert(await page.evaluate(()=>window.hintShownAt-window.hintEnteredAt>=70),`${mode}: actual open delay duration`);await tick();
      assert(await tip.isVisible()&&await outside.evaluate(e=>e===document.activeElement),`${mode}: hovering preserves actual focus`);
      const tipCdp=await context.newCDPSession(page),tipAx=await tipCdp.send('Accessibility.getFullAXTree');await tipCdp.detach();
      assert(tipAx.nodes.some(node=>node.role?.value==='tooltip')&&tipAx.nodes.some(node=>node.role?.value==='button'&&node.name?.value==='Save draft'&&node.description?.value==='Your draft is saved privately until you publish it.'),`${mode}: actual AX Tooltip role and trigger description`);
      await save.focus();await page.mouse.move(1080,850);await settle();assert(await tip.isVisible(),`${mode}: focus keeps tooltip after pointer leaves`);
      await page.keyboard.press('Escape');await settle();assert(!await tip.isVisible()&&await save.evaluate(e=>e===document.activeElement),`${mode}: trigger Escape dismisses without focus move`);
      await about.focus();await settle();assert(await card.isVisible()&&await about.evaluate(e=>e===document.activeElement),`${mode}: HoverCard opens dialog while preserving trigger focus`);
      await page.keyboard.press('Tab');await tick();const profile=page.getByRole('link',{name:'Read Ada profile',exact:true}),follow=page.getByRole('button',{name:'Follow Ada',exact:true});
      assert(await profile.evaluate(e=>e===document.activeElement),`${mode}: Tab reaches actual interactive card content`);
      await page.keyboard.press('Tab');await page.keyboard.press('Enter');await settle();assert(await card.isVisible()&&await page.getByText('Followed: 1',{exact:true}).count()===1,`${mode}: descendant focus transitions and native action`);
      await outside.focus();await about.hover();await settle();const triggerBox=await about.boundingBox(),cardBox=await card.boundingBox();
      assert(triggerBox&&cardBox&&cardBox.x>=0&&cardBox.x+cardBox.width<=1101,`${mode}: actual anchored card`);
      await page.mouse.move(triggerBox.x+triggerBox.width/2,triggerBox.y+triggerBox.height+2);await profile.hover();await settle();
      assert(await card.isVisible()&&await outside.evaluate(e=>e===document.activeElement),`${mode}: cancellable close delay crosses trigger/card gap without grabbing focus`);
      await profile.focus();await page.mouse.move(1080,850);await settle();assert(await card.isVisible(),`${mode}: interactive focus survives pointer exit`);
      await outside.focus();await settle();assert(!await card.isVisible()&&await page.getByRole('button',{name:'Follow Ada',exact:true}).count()===0,`${mode}: closing unmounts child Owner`);
      await page.evaluate(()=>{demo.hints_state('reject',true);demo.hints_state('tip',true);});
      const unrelated=page.locator('#unrelated-hint-focus');await unrelated.focus();await page.evaluate(()=>{window.foreignEscape=null;document.querySelector('#unrelated-hint-focus').addEventListener('keydown',event=>{window.foreignEscape=event.defaultPrevented;});});
      await page.keyboard.press('Escape');await tick();assert(await tip.isVisible()&&await page.evaluate(()=>window.foreignEscape===false)&&await page.getByText('Dismissed tips: 1',{exact:true}).count()===1,`${mode}: unrelated document focus Escape remains unconsumed`);
      await save.focus();await page.keyboard.press('Escape');await settle();assert(await tip.isVisible()&&await page.getByText('Dismissed tips: 2',{exact:true}).count()===1,`${mode}: controlled Escape rejection keeps source truth`);
      await page.evaluate(()=>demo.hints_state('reject',false));await page.keyboard.press('Escape');await settle();assert(!await tip.isVisible(),`${mode}: accepted Escape close`);
      await page.evaluate(()=>demo.hints_state('disabled',true));await tick();assert(await save.isDisabled()&&await save.getAttribute('aria-describedby')===null,`${mode}: disabled tooltip clears its description relation`);await page.evaluate(()=>demo.hints_state('disabled',false));
      const search=page.getByRole('textbox',{name:'Search',exact:true});await search.focus();await tick();const searchTip=page.locator('#'+await search.getAttribute('aria-describedby'));await search.fill('edited query');await tick();
      assert(await searchTip.isVisible()&&await search.inputValue()==='edited query'&&await search.evaluate(e=>e===document.activeElement),`${mode}: Input target retains value and actual focus`);
      await page.keyboard.press('Escape');await tick();assert(!await searchTip.isVisible()&&await search.inputValue()==='edited query',`${mode}: description metadata does not rewrite editable value`);
      await outside.focus();await about.hover();await settle();await profile.hover();await settle();
      const cdp=await context.newCDPSession(page),ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();
      assert(ax.nodes.some(node=>node.role?.value==='dialog'&&node.name?.value==='Ada profile')&&ax.nodes.some(node=>node.role?.value==='link'&&node.name?.value==='Read Ada profile'),`${mode}: real AX dialog and interactive descendants`);
      await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});
      await page.setViewportSize({width:390,height:844});await tick();const narrow=await card.boundingBox();assert(narrow&&narrow.x>=0&&narrow.x+narrow.width<=391,`${mode}: narrow viewport collision`);await page.setViewportSize({width:1100,height:900});
      await page.mouse.move(1080,850);await outside.focus();await settle();await save.hover();await page.waitForTimeout(20);await page.evaluate(()=>demo.unmount_hints());await settle();assert(await page.locator('#app').evaluate(e=>e.childElementCount===0),`${mode}: Owner retirement cancels a pending timer`);
      reports.push({suite:mode,status:'passed',checks:'true Tooltip/description, no popup/expanded, hover+focus, open/close delay cancellation and gap, preserved actual focus, trigger/foreign Escape, controlled rejection, live descendant focus/Tab/action, Input value, AX, child/Owner cleanup, viewport collision'});
    }
    await page.goto(`${baseURL}/hints-hydrate-blue-light.html?manual=1`);await page.waitForFunction(()=>window.moduleReady===true);
    await page.evaluate(()=>{window.savedHintInput=document.querySelector('input');savedHintInput.value='preloaded query';savedHintInput.focus();savedHintInput.setSelectionRange(3,9);demo.mount_hints(true,'blue','light');});await tick();
    assert(await page.evaluate(()=>savedHintInput.isConnected&&savedHintInput===document.activeElement&&savedHintInput.value==='preloaded query'&&savedHintInput.selectionStart===3&&savedHintInput.selectionEnd===9&&!document.getElementById(savedHintInput.getAttribute('aria-describedby')).hidden),'dirty Input tooltip adoption retains DOM identity/value/UTF16 selection/focus');
    reports.push({suite:'hints-dirty-hydration',status:'passed',checks:'actual focused editable target preserves identity/value/selection and opens description without autofocus'});
    await page.goto(`${baseURL}/hints-stream.html`);await page.waitForFunction(()=>document.querySelector('#hints-stream-root')?.dataset.glyStreamState==='ready');
    assert(await page.locator('#stream-tip-target').getAttribute('aria-describedby')==='stream-tip'&&await page.locator('#stream-tip').getAttribute('role')==='tooltip'&&await page.locator('#stream-tip').textContent()==='Loaded tooltip description'&&!await page.locator('#stream-tip').evaluate(e=>e.hidden),'classic stream description/role/text/hidden Set/Clear');
    assert(await page.locator('#stream-tip-target').getAttribute('aria-haspopup')===null&&await page.locator('#stream-tip-target').getAttribute('aria-expanded')===null&&await page.evaluate(()=>GloryStream.active()===0&&document.querySelector('#hints-stream-root').firstChild.nodeValue==='gly:v:13'),'classic stream passive semantics and finite End rekey');
    reports.push({suite:'hints-classic-stream',status:'passed',checks:'actual Rust Shell/Patch/End description text/hidden reset, no popup/expanded, v3 rekey'});
    assert(errors.length===0,JSON.stringify(errors));return{reports,errors};
  } finally {await context.close();}
}
