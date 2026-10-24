async(hostPage,baseURL='http://127.0.0.1:8794')=>{
  const browser=hostPage.context().browser(),context=await browser.newContext({viewport:{width:1100,height:900}}),reports=[],errors=[];
  const assert=(ok,message)=>{if(!ok)throw new Error(message)};
  try{
    const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
    const tick=()=>page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
    const state=(kind,value)=>page.evaluate(([k,v])=>demo.color_sliders_state(k,v),[kind,value]);
    const number=async locator=>Number(await locator.getAttribute('aria-valuenow'));
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]){
      const mode=`color-sliders-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
      await page.goto(`${baseURL}/${hydrate?`color-sliders-hydrate-${hue}-${scheme}.html`:`color-sliders.html?theme=${hue}&scheme=${scheme}`}`);await page.waitForFunction(()=>window.ready===true);
      const h=page.getByRole('slider',{name:'Hue channel',exact:true}),a=page.getByRole('slider',{name:'Alpha channel',exact:true}),rtl=page.getByRole('slider',{name:'RTL hue',exact:true}),notes=page.getByRole('textbox',{name:'Color notes',exact:true});
      assert(await number(h)===120.123456789&&await h.getAttribute('aria-valuemax')==='360'&&await a.getAttribute('aria-valuemax')==='1',`${mode}: exact independent color domains`);
      assert(await h.locator('.sapid-color-slider-segment').count()===120&&await a.locator('.sapid-color-slider-segment').count()===64&&await a.locator('.sapid-alpha-checker-cell').count()===64,`${mode}: fixed live trail/checker budgets`);
      const unstyled=await page.getByRole('slider',{name:'Unstyled alpha',exact:true}).boundingBox(),notesBox=await notes.boundingBox();
      assert(unstyled.height>=24&&unstyled.height<=50&&Math.abs(unstyled.width-240)<1&&unstyled.y+unstyled.height<=notesBox.y+1,`${mode}: unstyled retains finite default dimensions and does not overlap the following editor (${JSON.stringify(unstyled)})`);
      assert(await h.getAttribute('data-gly-control-name')==='hue'&&await a.getAttribute('data-gly-control-name')==='alpha',`${mode}: explicit Field snapshot names`);
      await notes.evaluate(e=>{window.savedColorNotes=e;e.focus();e.setSelectionRange(2,9,'backward')});await h.evaluate(e=>window.savedHue=e);await a.evaluate(e=>window.savedAlpha=e);
      await h.focus();await page.keyboard.press('Home');await page.keyboard.press('ArrowRight');await tick();assert(await number(h)===0.25,`${mode}: fractional hue keyboard step`);
      await page.keyboard.press('End');await page.keyboard.press('ArrowRight');await tick();assert(await number(h)===360,`${mode}: hue endpoints clamp`);
      await rtl.focus();await page.keyboard.press('ArrowRight');await tick();assert(await number(rtl)===179,`${mode}: RTL keyboard direction`);
      await a.focus();await page.keyboard.press('Home');await page.keyboard.press('ArrowRight');await tick();assert(Math.abs(await number(a)-0.01)<1e-12,`${mode}: alpha step`);
      await state('alpha','0.25');await state('accept','false');await a.focus();await page.keyboard.press('ArrowRight');await tick();assert(await number(a)===0.25,`${mode}: controlled refusal`);
      await state('commit_accept','true');await a.evaluate(e=>{e.focus();for(let i=0;i<2;i++)e.dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowRight',bubbles:true,cancelable:true}))});await tick();assert(Math.abs(await number(a)-0.27)<1e-12,`${mode}: same-tick commit-only acceptance`);
      await state('commit_accept','false');await state('accept','true');
      const hbox=await h.boundingBox();await page.mouse.click(hbox.x+hbox.width*0.75,hbox.y+hbox.height/2);await tick();assert(Math.abs(await number(h)-270)<1,`${mode}: real pointer hue geometry`);
      const abox=await a.boundingBox();await page.mouse.move(abox.x+abox.width*0.4,abox.y+abox.height/2);await page.mouse.down();await page.mouse.move(abox.x+abox.width+120,abox.y+abox.height+60);await page.mouse.up();await tick();assert(await number(a)===1,`${mode}: captured release outside alpha bounds`);
      await state('color','alternate');await tick();assert(await a.locator('.sapid-color-slider-segment').last().evaluate(e=>getComputedStyle(e).backgroundColor)==='rgb(200, 50, 100)',`${mode}: RGB update preserves independent alpha`);
      assert(await h.evaluate(e=>e===window.savedHue)&&await a.evaluate(e=>e===window.savedAlpha)&&await notes.evaluate(e=>e===window.savedColorNotes&&e.value==='Retain 😀 caret'&&e.selectionStart===2&&e.selectionEnd===9),`${mode}: control and unrelated editor identity/selection`);
      await state('readonly','true');await a.focus();await page.keyboard.press('ArrowLeft');await tick();assert(await number(a)===1&&await a.getAttribute('aria-readonly')==='true',`${mode}: readonly Field retains focus and refuses changes`);
      await state('readonly','false');await state('enabled','false');await tick();assert(await h.getAttribute('aria-disabled')==='true'&&await a.getAttribute('aria-disabled')==='true',`${mode}: disabled Field`);await state('enabled','true');
      await page.getByRole('button',{name:'Before colors',exact:true}).focus();await page.keyboard.press('Tab');assert(await h.evaluate(e=>e===document.activeElement),`${mode}: hue tab stop`);await page.keyboard.press('Tab');assert(await a.evaluate(e=>e===document.activeElement),`${mode}: alpha tab stop without passive trail focus`);
      const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();assert(ax.nodes.some(n=>n.role?.value==='slider'&&n.name?.value==='Hue channel')&&ax.nodes.some(n=>n.role?.value==='slider'&&n.name?.value==='Alpha channel'),`${mode}: actual two named AX sliders`);
      await page.setViewportSize({width:390,height:844});await tick();assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${mode}: mobile geometry`);await page.setViewportSize({width:1100,height:900});
      if(hue==='green'&&scheme==='dark'&&hydrate)await page.screenshot({path:'D:/BuildCache/sapid/color-sliders-green-dark-hydrate.png',fullPage:true});
      await page.evaluate(()=>demo.unmount_color_sliders());assert(await page.locator('#app').evaluate(e=>e.childElementCount===0),`${mode}: Owner cleanup`);
      reports.push({suite:mode,status:'passed',checks:'exact numeric channels, fixed color/checker trails, Field/AX, fractional/RTL keys, controlled refusal and same-tick commit acceptance, captured pointer, independent RGB updates, stable editor, readonly/disabled, mobile, retirement'});
    }
    await page.goto(`${baseURL}/color-sliders-hydrate-blue-light.html?manual=1`);await page.waitForFunction(()=>window.moduleReady===true);await page.getByRole('textbox',{name:'Color notes',exact:true}).evaluate(e=>{window.preColorNotes=e;e.value='Dirty 😀 notes';e.focus();e.setSelectionRange(1,6);demo.mount_color_sliders(true,'blue','light')});await tick();assert(await page.getByRole('textbox',{name:'Color notes',exact:true}).evaluate(e=>e===window.preColorNotes&&e===document.activeElement&&e.value==='Dirty 😀 notes'&&e.selectionStart===1&&e.selectionEnd===6),'color controls preserve dirty adjacent editor adoption');reports.push({suite:'color-sliders-dirty-hydration',status:'passed'});
    const touch=await browser.newContext({viewport:{width:390,height:844},hasTouch:true,isMobile:true});try{
      const p=await touch.newPage();p.on('pageerror',e=>errors.push(e.message));await p.goto(`${baseURL}/color-sliders.html`);await p.waitForFunction(()=>window.ready===true);const alpha=p.getByRole('slider',{name:'Alpha channel',exact:true});await alpha.scrollIntoViewIfNeeded();const box=await alpha.boundingBox(),cdp=await touch.newCDPSession(p);
      try{await cdp.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x:box.x+box.width*0.2,y:box.y+box.height/2}]});await cdp.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x:box.x+box.width*0.6,y:box.y+box.height/2}]});await cdp.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});await p.waitForFunction(()=>Math.abs(Number(document.querySelector('.main-alpha').getAttribute('aria-valuenow'))-0.6)<0.03);reports.push({suite:'color-sliders-touch-capture',status:'passed'});}finally{await cdp.detach()}
    }finally{await touch.close()}
    assert(errors.length===0,JSON.stringify(errors));return{reports,errors};
  }finally{await context.close()}
}
