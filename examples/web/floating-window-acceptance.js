async(hostPage,baseURL='http://127.0.0.1:8794')=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1200,height:950},hasTouch:true});
  const reports=[],errors=[];const assert=(value,message)=>{if(!value)throw new Error(message)};const near=(a,b)=>Math.abs(a-b)<1.1;
  try{
    const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
    const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    const working=()=>page.getByRole('dialog',{name:'Working window',exact:true});
    const secondary=()=>page.getByRole('dialog',{name:'Secondary window',exact:true});
    const draft=()=>page.getByRole('textbox',{name:'Window draft',exact:true});
    const button=name=>page.getByRole('button',{name,exact:true});
    const activate=async locator=>{await locator.focus();await locator.press('Enter');await tick()};
    const box=()=>working().boundingBox();
    const status=()=>page.locator('.window-bounds-status').textContent();
    const move=async(locator,dx,dy)=>{const r=await locator.boundingBox();await page.mouse.move(r.x+12,r.y+12);await page.mouse.down();await page.mouse.move(r.x+12+dx,r.y+12+dy,{steps:4});await page.mouse.up();await tick()};
    const open=async(path='floating-window.html')=>{await page.goto(`${baseURL}/${path}`);await page.waitForFunction(()=>window.ready===true);await tick()};
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const mode of ['csr','hydrate']){
      await open(`${mode==='csr'?'floating-window':`floating-window-hydrate-${hue}-${scheme}`}.html?theme=${hue}&scheme=${scheme}`);
      await secondary().getByRole('button',{name:'Close window',exact:true}).click();await tick();
      let r=await box();assert(near(r.x,100)&&near(r.y,120)&&near(r.width,560)&&near(r.height,420),`${hue}/${scheme}/${mode}: actual viewport bounds`);
      assert((await working().ariaSnapshot()).includes('dialog "Working window"'),'real Dialog name');
      await activate(working().getByRole('button',{name:'Move window right',exact:true}));
      await activate(working().getByRole('button',{name:'Make window wider',exact:true}));
      r=await box();assert(near(r.x,110)&&near(r.width,570),'real keyboard Buttons move and resize');
      await draft().fill('Window text');await draft().focus();await draft().evaluate(e=>{window.savedWindowDraft=e;e.setSelectionRange(2,5)});
      const before=await box();await move(working().locator('.sapid-window-caption'),70,35);r=await box();
      assert(near(r.x,before.x+70)&&near(r.y,before.y+35),'captured caption moves actual fixed box');
      assert(await draft().evaluate(e=>e===window.savedWindowDraft&&e===document.activeElement&&e.value==='Window text'&&e.selectionStart===2&&e.selectionEnd===5),'caption preserves actual editor identity/focus/caret');
      await move(working().locator('.sapid-window-resize-grip'),60,45);const resized=await box();
      assert(near(resized.width,r.width+60)&&near(resized.height,r.height+45),'corner capture resizes actual box');
      await working().getByRole('button',{name:'Close window',exact:true}).click();await tick();assert(await working().count()===0,'close hides layer');
      await activate(button('Toggle working window'));assert(await draft().evaluate(e=>e===window.savedWindowDraft&&e.value==='Window text'),'KeepMounted reopening retains draft');
      reports.push({suite:`floating-window-${hue}-${scheme}-${mode}`,status:'passed'});
    }
    await open();await secondary().getByRole('button',{name:'Close window',exact:true}).click();await tick();
    await activate(button('Toggle geometry refusal'));const held=await box();
    await activate(working().getByRole('button',{name:'Move window right',exact:true}));await move(working().locator('.sapid-window-caption'),80,40);
    let r=await box();assert(near(r.x,held.x)&&near(r.y,held.y)&&(await status()).includes('reject=true'),'controlled refusal never optimistically changes geometry');
    await activate(button('Toggle geometry refusal'));await activate(button('Toggle window enabled'));
    assert(await draft().isDisabled(),'real disabled Fieldset disables editor');const disabled=await box();await move(working().locator('.sapid-window-caption'),60,20);r=await box();
    assert(near(r.x,disabled.x)&&near(r.y,disabled.y),'disabled pointer cannot move window');
    await activate(button('Toggle window enabled'));assert(await draft().isEnabled(),'reenable restores native control');
    reports.push({suite:'floating-window-controlled-refusal-and-disabled',status:'passed'});

    await open();await draft().focus();await tick();await button('Raise secondary window').evaluate(e=>e.click());await tick();
    const overlap=await page.evaluate(()=>{const a=document.querySelectorAll('.sapid-floating-window')[0].getBoundingClientRect(),b=document.querySelectorAll('.sapid-floating-window')[1].getBoundingClientRect();return {x:Math.max(a.x,b.x)+30,y:Math.max(a.y,b.y)+30}});
    const hit=()=>page.evaluate(({x,y})=>document.elementFromPoint(x,y)?.closest('.sapid-floating-window')?.textContent.includes('Secondary window'),overlap);
    assert(await hit(),'real second top layer paints/hits first');
    await draft().evaluate(e=>{window.savedWindowDraft=e;window.windowCompositionEnds=0;e.addEventListener('compositionend',()=>window.windowCompositionEnds++);e.dispatchEvent(new CompositionEvent('compositionstart',{bubbles:true,data:'n'}));e.value='Keep this editorn';e.setSelectionRange(3,6);e.dispatchEvent(new InputEvent('input',{bubbles:true,data:'n',inputType:'insertCompositionText',isComposing:true}))});await tick();
    await button('Raise working window').evaluate(e=>e.click());await tick();assert(!await hit(),'Raise changes real host layer hit order');
    assert(await draft().evaluate(e=>e===window.savedWindowDraft&&e===document.activeElement&&e.selectionStart===3&&e.selectionEnd===6&&e.value==='Keep this editorn'&&window.windowCompositionEnds===0),'Raise preserves selection/preedit and editor identity');
    await draft().evaluate(e=>e.dispatchEvent(new CompositionEvent('compositionend',{bubbles:true,data:'n'})));await tick();
    await activate(button('Open modal blocker'));await page.getByRole('dialog',{name:'Modal blocker',exact:true}).getByRole('button',{name:'Try blocked window raise',exact:true}).click();await tick();
    assert(await page.getByRole('dialog').count()===1&&await page.getByRole('dialog',{name:'Modal blocker',exact:true}).evaluate(e=>e.contains(document.activeElement)),'Raise cannot cross current document modal');
    await page.keyboard.press('Escape');await tick();assert(await working().count()===1,'modal close restores live window');
    reports.push({suite:'floating-window-real-layer-order-editor-ime-and-modal-block',status:'passed'});

    await open();await secondary().getByRole('button',{name:'Close window',exact:true}).click();await tick();
    const client=await context.newCDPSession(page);const caption=await working().locator('.sapid-window-caption').boundingBox();const initial=await box();
    await client.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x:caption.x+12,y:caption.y+12,id:1}]});
    await client.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x:caption.x+92,y:caption.y+62,id:1}]});
    await client.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});await tick();r=await box();
    assert(near(r.x,initial.x+80)&&near(r.y,initial.y+50),'real touch capture moves window');await client.detach();
    reports.push({suite:'floating-window-touch-capture',status:'passed'});

    await open();await page.setViewportSize({width:600,height:500});await tick();r=await box();assert(near(r.x,32)&&near(r.y,72),'viewport collision clamps both coordinates');
    await page.setViewportSize({width:340,height:260});await tick();r=await box();assert(near(r.x,8)&&near(r.y,8),'oversized window remains at padded viewport start');
    await page.setViewportSize({width:1200,height:950});await tick();r=await box();assert(near(r.x,100)&&near(r.y,120)&&(await status()).includes('proposals=0'),'viewport changes preserve controlled source and do not emit proposals');
    reports.push({suite:'floating-window-viewport-collision-and-resize',status:'passed'});

    await open();await secondary().getByRole('button',{name:'Close window',exact:true}).click();await tick();await draft().evaluate(e=>window.savedWindowDraft=e);
    const retiring=await working().locator('.sapid-window-caption').boundingBox();await page.mouse.move(retiring.x+12,retiring.y+12);await page.mouse.down();await button('Retire windows').evaluate(e=>e.click());await tick();
    await page.mouse.move(retiring.x+120,retiring.y+70);await page.mouse.up();await tick();assert(await page.locator('.sapid-floating-window').count()===0,'Owner retirement removes layers and pending capture/query');
    await activate(button('Invoke retired window command'));assert((await status()).includes('retired denied'),'retired handle is rejected');await activate(button('Rebuild windows'));
    assert(await draft().evaluate(e=>e!==window.savedWindowDraft),'new Owner gets a fresh editor identity');
    reports.push({suite:'floating-window-owner-cancel-query-and-rebuild',status:'passed'});
    assert(errors.length===0,errors.join('\n'));return {reports,errors};
  }finally{await context.close()}
}
