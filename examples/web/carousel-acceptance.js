async (hostPage, suppliedBase) => {
  const base=suppliedBase||process.env.SAPID_CAROUSEL_BASE||'http://127.0.0.1:8794';
  const context=await hostPage.context().browser().newContext({viewport:{width:1440,height:1100},hasTouch:true});
  const reports=[],errors=[];const assert=(value,message)=>{if(!value)throw Error(message)};
  try {
    const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
    const settle=async()=>{await page.evaluate(()=>new Promise(resolve=>{let frames=0;const next=()=>++frames===8?resolve():requestAnimationFrame(next);requestAnimationFrame(next)}));};
    const button=name=>page.getByRole('button',{name,exact:true});
    const change=async name=>{await button(name).evaluate(e=>e.click());await settle()};
    const status=async()=>{const text=await page.locator('.carousel-fixture-status').textContent();const m=text.match(/Current: (\d+); proposals: (\d+); factories: (\d+); cleaned: (\d+); reject: (true|false); order: (.*)/);assert(m,`bad carousel fixture status: ${text}`);return{current:+m[1],proposals:+m[2],factories:+m[3],cleaned:+m[4],reject:m[5]==='true',order:m[6]};};
    const position=selector=>page.locator(`${selector} .sapid-carousel-position`).textContent();
    const mouseSwipe=async(selector,dx,dy=0)=>{
      await page.locator(selector).scrollIntoViewIfNeeded();const b=await page.locator(selector).boundingBox();assert(b,'missing swipe box');
      const x=b.x+b.width/2,y=b.y+Math.min(b.height-55,130);
      await page.mouse.move(x,y);await page.mouse.down();await page.mouse.move(x+dx,y+dy,{steps:6});await page.mouse.up();await settle();
    };
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]){
      const mode=`carousel-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
      await page.setViewportSize({width:1440,height:1100});
      await page.goto(`${base}/${hydrate?`carousel-hydrate-${hue}-${scheme}`:'carousel'}.html?manual=1`);await page.waitForFunction(()=>window.moduleReady===true);
      if(hydrate)assert(await page.locator('.carousel-main .sapid-carousel-slide').count()===7,`${mode}: deterministic SSR window`);
      await page.evaluate(({hydrate,hue,scheme})=>window.demo.mount_carousel(hydrate,hue,scheme),{hydrate,hue,scheme});await settle();
      let s=await status();assert(s.current===5&&s.factories===7&&s.cleaned===0,`${mode}: bounded initial window ${JSON.stringify(s)}`);
      assert(await page.locator('.carousel-main .sapid-carousel-slide').count()===7&&await page.locator('.carousel-main .sapid-carousel-slide:not([hidden])').count()===1,`${mode}: only one visible slide`);
      const main=page.locator('.carousel-main');assert(await main.getAttribute('role')==='group'&&await main.getAttribute('aria-label')==='Carousel'&&await main.getAttribute('tabindex')==='0',`${mode}: actual named focusable group`);
      assert(await main.evaluate(e=>getComputedStyle(e).touchAction.includes('pan-y')),`${mode}: native orthogonal scroll policy`);
      await page.locator('.carousel-editor-5').fill('Retained draft 😀');await page.locator('.carousel-editor-5').evaluate(e=>{e.setSelectionRange(2,7);window.carouselEditor=e});
      await change('Program next slide');s=await status();assert(s.current===6&&s.proposals===0&&s.factories===8&&s.cleaned===1,`${mode}: source silent / bounded retirement`);
      await main.locator('.sapid-carousel-previous').click();await settle();
      assert(await page.locator('.carousel-editor-5').evaluate(e=>e===window.carouselEditor&&e.value==='Retained draft 😀'&&e.selectionStart===2&&e.selectionEnd===7),`${mode}: retained editor identity and UTF16 selection`);
      await main.focus();await page.keyboard.press('ArrowRight');await settle();s=await status();assert(s.current===6&&s.order==='typed:6;generic',`${mode}: keyboard / typed before generic`);
      await change('Toggle controlled rejection');const before=await status();await main.focus();await page.keyboard.press('ArrowRight');await settle();s=await status();assert(s.current===6&&s.proposals===before.proposals+1&&s.order==='typed:7;generic',`${mode}: controlled rejection`);
      await change('Toggle controlled rejection');const proposalBefore=(await status()).proposals;
      await mouseSwipe('.carousel-main',-130);s=await status();assert(s.current===7&&s.proposals===proposalBefore+1,`${mode}: actual captured mouse swipe`);
      const editor=page.locator('.carousel-editor-7');await editor.focus();await page.keyboard.press('Home');await page.keyboard.press('ArrowRight');await settle();assert((await status()).current===7,`${mode}: editor keyboard remains native`);
      await page.locator('.carousel-rtl').focus();await page.keyboard.press('ArrowLeft');await settle();assert(await position('.carousel-rtl')==='2 / 3',`${mode}: RTL arrow direction`);
      await mouseSwipe('.carousel-rtl',110);assert(await position('.carousel-rtl')==='3 / 3',`${mode}: RTL swipe direction`);
      const initialAuto=await position('.carousel-auto');await change('Toggle autoplay source pause');await page.mouse.move(0,0);await page.waitForTimeout(260);assert(await position('.carousel-auto')!==initialAuto,`${mode}: actual injected autoplay timer`);
      await page.locator('.carousel-auto').hover();await settle();const hovered=await position('.carousel-auto');await page.waitForTimeout(280);assert(await position('.carousel-auto')===hovered,`${mode}: hover cancels current autoplay wait`);
      await page.locator('.carousel-auto').focus();await page.mouse.move(0,0);await settle();const focused=await position('.carousel-auto');await page.waitForTimeout(260);assert(await position('.carousel-auto')===focused,`${mode}: focus pauses autoplay`);
      await page.locator('.carousel-auto .sapid-carousel-pause').evaluate(e=>e.click());await button('Toggle autoplay source pause').focus();await settle();const manualPause=await position('.carousel-auto');await page.waitForTimeout(260);assert(await position('.carousel-auto')===manualPause,`${mode}: explicit pause survives focus leaving`);
      await change('Toggle autoplay source pause');
      assert(await page.locator('.carousel-plain').evaluate(e=>e.classList.contains('sapid-unstyled')&&e.style.background===''),`${mode}: unstyled removes theme`);
      await change('Toggle carousel enabled');assert(await main.getAttribute('aria-disabled')==='true'&&await main.locator('.sapid-carousel-next').isDisabled(),`${mode}: disabled native controls`);
      await change('Toggle carousel enabled');await page.setViewportSize({width:390,height:900});await settle();assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${mode}: mobile width budget`);
      if(hydrate)await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});
      await page.evaluate(()=>window.demo.unmount_display());await settle();assert(await page.locator('#app').textContent()==='',`${mode}: Owner removes all physical content`);
      reports.push({mode,status:'passed',window:7,checks:'bounded million source, short Owners/editor selection, source silence, controlled refusal, actual keyboard/mouse swipe, RTL, autoplay timer/hover/focus/manual pause, four-theme/unstyled/disabled/mobile/teardown'});
    }
    // Trusted touch input, axis intent, multi-pointer and captured revision retirement.
    await page.setViewportSize({width:720,height:1000});await page.goto(`${base}/carousel.html?manual=1`);await page.waitForFunction(()=>window.moduleReady===true);await page.evaluate(()=>window.demo.mount_carousel(false,'blue','light'));await settle();
    const cdp=await context.newCDPSession(page);const touch=async(type,points)=>cdp.send('Input.dispatchTouchEvent',{type,touchPoints:points.map((p,index)=>({id:index+1,x:p[0],y:p[1],radiusX:2,radiusY:2,force:1}))});
    const probe=page.locator('.swipe-protocol');await probe.scrollIntoViewIfNeeded();let box=await probe.boundingBox();let x=box.x+box.width/2,y=box.y+80;
    await touch('touchStart',[[x,y]]);await touch('touchMove',[[x+65,y+2]]);await settle();let log=await page.locator('.swipe-protocol-log').textContent();assert(log.includes('Start:4;')&&log.includes('Move:4;'),'trusted touch axis capture');
    await touch('touchStart',[[x+65,y+2],[x+75,y+20]]);await touch('touchEnd',[]);await settle();log=await page.locator('.swipe-protocol-log').textContent();assert(log.includes('Cancel:4;')&&!log.includes('Drop:4;'),'second finger cancels capture without a drop');
    await probe.scrollIntoViewIfNeeded();box=await probe.boundingBox();x=box.x+box.width/2;y=box.y+80;
    await touch('touchStart',[[x,y]]);await touch('touchMove',[[x+70,y+2]]);await settle();
    await change('Set vertical swipe configuration');await touch('touchMove',[[x+100,y+2]]);await touch('touchEnd',[]);await settle();log=await page.locator('.swipe-protocol-log').textContent();assert(!log.includes('Drop:4;'),'changed configuration revokes old captured result');
    await change('Restore horizontal swipe configuration');const beforePan=await page.locator('.swipe-protocol-log').textContent();await probe.scrollIntoViewIfNeeded();box=await probe.boundingBox();x=box.x+box.width/2;y=box.y+60;
    await touch('touchStart',[[x,y]]);await touch('touchMove',[[x+2,y+75]]);await touch('touchEnd',[]);await settle();assert(await page.locator('.swipe-protocol-log').textContent()===beforePan,'orthogonal touch pan never becomes swipe');
    await change('Clear swipe configuration');assert(await probe.getAttribute('data-gly-swipe-axis')===null&&await probe.getAttribute('data-gly-swipe-revision')===null,'Clear removes both finite declarations');
    await page.evaluate(()=>window.demo.unmount_display());await settle();await cdp.detach();reports.push({mode:'carousel-trusted-touch-and-revision',status:'passed',checks:'real CDP touch, axis intent, second finger cancellation, source revision retirement, orthogonal pan, Clear'});
    assert(errors.length===0,errors.join('\n'));return{reports,errors};
  }finally{await context.close();}
}
