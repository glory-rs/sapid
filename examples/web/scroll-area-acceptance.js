async (hostPage) => {
 const context=await hostPage.context().browser().newContext({viewport:{width:1200,height:1000}}),reports=[],errors=[];
 const assert=(value,message)=>{if(!value)throw new Error(message)};
 try{
  const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
  const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))));
  const offset=area=>area.evaluate(e=>({x:e.scrollLeft,y:e.scrollTop}));
  for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydration of [false,true]){
   const mode=`scroll-area-${hue}-${scheme}-${hydration?'hydrate':'csr'}`;
   await page.goto(`http://127.0.0.1:8794/${hydration?`scroll-area-hydrate-${hue}-${scheme}.html`:`scroll-area.html?theme=${hue}&scheme=${scheme}`}`);
   await page.waitForFunction(()=>window.ready===true);await page.waitForFunction(()=>document.querySelector('.scroll-status').textContent.includes('content=1400x1600'));
   const area=page.locator('.main-scroll-area'),nested=page.locator('.nested-scroll-area'),draft=page.getByRole('textbox',{name:'Document draft'}),status=page.locator('.scroll-status');
   assert(await area.getAttribute('data-gly-scroll-keyboard')==='both'&&await area.getAttribute('tabindex')==='0',`${mode}: opted-in viewport keyboard missing`);
   assert(await area.evaluate(e=>e.scrollHeight===1600&&e.scrollWidth===1400&&e.clientHeight===260),`${mode}: actual content/client dimensions disagree`);
   await draft.fill('Preserved scroll draft');await draft.evaluate(e=>window.scrollDraft=e);
   await area.focus();await page.keyboard.press('ArrowDown');await tick();assert((await offset(area)).y===40,`${mode}: arrow command did not use queued step`);
   await page.keyboard.press('PageDown');await tick();assert((await offset(area)).y===300,`${mode}: page command did not use measured client viewport`);
   await page.keyboard.press('End');await tick();assert((await offset(area)).y===1340,`${mode}: End did not use actual scroll extent`);
   await page.keyboard.press('Home');await tick();assert((await offset(area)).y===0,`${mode}: Home did not return to start`);
   await page.evaluate(()=>window.demo.scroll_area_batch());await tick();assert((await offset(area)).y===50,`${mode}: same-batch opposite directions lost ordered edge clamping`);
   await page.evaluate(()=>window.demo.scroll_area_to(0,0));await tick();await nested.focus();const beforeNested=await offset(area);
   await page.keyboard.press('ArrowDown');await tick();assert((await offset(nested)).y===40&&(await offset(area)).y===beforeNested.y,`${mode}: nested viewport key reached ancestor`);
   const bounds=await nested.boundingBox();await page.mouse.move(bounds.x+80,bounds.y+50);await page.mouse.wheel(0,80);await tick();assert((await offset(nested)).y>40&&(await offset(area)).y===beforeNested.y,`${mode}: ordinary nested wheel scrolling failed`);
   await draft.focus();const beforeEdit=await offset(area);await page.keyboard.press('ArrowRight');await page.keyboard.press('End');await tick();assert((await offset(area)).x===beforeEdit.x&&(await offset(area)).y===beforeEdit.y,`${mode}: descendant editing was consumed as viewport scrolling`);
   const strip=page.locator('.horizontal-scroll-area'),previous=page.getByRole('button',{name:'Previous scroll page'}),next=page.getByRole('button',{name:'Next scroll page'});
   assert(await previous.isDisabled()&&!await next.isDisabled(),`${mode}: initial native page boundaries missing`);
   await next.click();await page.waitForFunction(()=>document.querySelector('.horizontal-scroll-area').scrollLeft===480&&[...document.querySelectorAll('button')].some(e=>e.textContent==='Previous scroll page'&&!e.disabled));assert((await offset(strip)).x===480&&!await previous.isDisabled(),`${mode}: Scroller page button did not scroll measured viewport`);
   await strip.focus();await page.keyboard.press('End');await page.waitForFunction(()=>document.querySelector('.horizontal-scroll-area').scrollLeft===1920&&[...document.querySelectorAll('button')].some(e=>e.textContent==='Next scroll page'&&e.disabled));assert((await offset(strip)).x===1920&&await next.isDisabled(),`${mode}: horizontal End or native disabled end boundary failed`);
   await previous.click();await tick();assert((await offset(strip)).x===1440,`${mode}: Scroller previous page did not restore boundary state`);
   await page.evaluate(()=>window.demo.scroll_area_to(0,0));await tick();
   assert(await draft.evaluate(e=>e===window.scrollDraft&&e.value==='Preserved scroll draft')&&(await status.textContent()).includes('ordered=true'),`${mode}: scrolling replaced editor or generic event preceded typed callback`);
   await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});reports.push({suite:mode,status:'passed',checks:'real content/client metrics, queued Arrow/Page/Home/End, ordered clamp batch, nested nearest viewport keyboard/wheel, descendant editor native keys, native horizontal Scroller page buttons and disabled boundaries, preserved DOM/draft, typed event order'});
  }
  await page.goto('http://127.0.0.1:8794/scroll-area-hydrate-green-dark.html?manual');await page.waitForFunction(()=>window.moduleReady===true);
  const area=page.locator('.main-scroll-area'),draft=page.locator('.scroll-draft');await draft.fill('Before scroll hydration');await draft.evaluate(e=>{window.preScroll=e;e.focus();e.setSelectionRange(2,8,'backward')});await area.evaluate(e=>e.scrollTop=500);
  await page.evaluate(()=>window.demo.mount_scroll_area(true,'green','dark'));await tick();assert(await draft.evaluate(e=>e===window.preScroll&&e.value==='Before scroll hydration'&&e.selectionStart===2&&e.selectionEnd===8),'scroll hydration lost editor identity/draft/selection');assert((await offset(area)).y===500,'scroll hydration reset actual viewport offset');
  await page.evaluate(()=>window.demo.unmount_scroll_area());await tick();assert(await page.locator('.scroll-demo').count()===0,'scroll Owner retirement leaked viewport/content');reports.push({suite:'scroll-area-dirty-hydration-retirement',status:'passed'});
  return{reports,errors};
 }finally{await context.close()}
}
