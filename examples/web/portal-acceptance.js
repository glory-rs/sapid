async(hostPage,baseURL='http://127.0.0.1:8794')=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1200,height:900}}),reports=[],errors=[];
  const assert=(value,message)=>{if(!value)throw new Error(message)};
  try{
    const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
    const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    const button=name=>page.getByRole('button',{name,exact:true});
    const draft=()=>page.getByRole('textbox',{name:'Portaled draft',exact:true});
    const status=()=>page.locator('.portal-status').textContent();
    const open=async(path='portal.html')=>{await page.goto(`${baseURL}/${path}`);await page.waitForFunction(()=>window.ready===true);await tick()};
    const command=async name=>{await button(name).evaluate(el=>el.click());await tick()};
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const mode of ['csr','hydrate']){
      await open(`${mode==='csr'?'portal':`portal-hydrate-${hue}-${scheme}`}.html?theme=${hue}&scheme=${scheme}`);
      assert(await page.locator('.portal-destination-a input').count()===1&&await page.locator('.portal-source-location input').count()===0,'content belongs to actual target');
      await button('Increment portal counter').click();await tick();await draft().fill('Keep selected portal draft');await draft().focus();
      await draft().evaluate(el=>{el.setSelectionRange(3,8);window.__portalDraft=el;window.__portalContent=el.closest('.portal-content')});
      await command('Move portal to B');
      assert(await page.locator('.portal-destination-b input').count()===1&&await page.locator('.portal-destination-a input').count()===0,'real parent changed');
      assert(await draft().evaluate(el=>el===window.__portalDraft&&el.closest('.portal-content')===window.__portalContent&&el===document.activeElement&&el.selectionStart===3&&el.selectionEnd===8),'connected move preserves real editor, focus and selection');
      assert(await draft().inputValue()==='Keep selected portal draft'&&(await page.locator('.portal-local-counter').textContent())==='source counter=1','source-owned state survived');
      assert((await status()).includes('factories=1'),'factory invoked once');
      await command('Toggle target B');assert(await draft().count()===0,'retired target parks current content');
      assert(await page.evaluate(()=>!window.__portalDraft.isConnected&&window.__portalDraft.value==='Keep selected portal draft'),'same parked editor retains value');
      await command('Toggle target B');assert(await draft().evaluate(el=>el===window.__portalDraft),'new target resumes existing editor');
      assert((await page.locator('.portal-local-counter').textContent())==='source counter=1'&&(await status()).includes('factories=1'),'Owner remains source owned');
      reports.push({suite:`portal-${hue}-${scheme}-${mode}`,status:'passed'});
    }
    await open();await draft().focus();await draft().evaluate(el=>{window.__imeEnd=0;window.__imePortal=el;el.addEventListener('compositionend',()=>window.__imeEnd++);el.dispatchEvent(new CompositionEvent('compositionstart',{data:'',bubbles:true}));el.setSelectionRange(2,7)});
    await command('Move portal to B');assert(await draft().evaluate(el=>el===window.__imePortal&&el===document.activeElement&&el.selectionStart===2&&el.selectionEnd===7&&window.__imeEnd===0),'range move preserves ongoing composition protocol');
    await draft().evaluate(el=>el.dispatchEvent(new CompositionEvent('compositionend',{data:'',bubbles:true})));reports.push({suite:'portal-focused-editor-composition-protocol',status:'passed'});
    await open();await button('Increment portal counter').click();await tick();await draft().evaluate(el=>window.__parkedPortal=el);
    await command('Toggle portal enabled');assert(await draft().count()===0,'disabled source parks without destruction');
    await command('Toggle portal enabled');assert(await draft().evaluate(el=>el===window.__parkedPortal),'enable resumes same editor');
    await command('Toggle portal source');assert(await draft().count()===0,'source retirement destroys moved descendants');
    await command('Toggle portal source');assert(await draft().evaluate(el=>el!==window.__parkedPortal),'new source has fresh identity');
    assert((await status()).includes('factories=2')&&(await page.locator('.portal-local-counter').textContent())==='source counter=0','new short Owner has fresh state');reports.push({suite:'portal-enable-and-source-owner-retirement',status:'passed'});
    await open();await command('Toggle target B');await draft().evaluate(el=>window.__waitingPortal=el);await command('Move portal to B');assert(await draft().count()===0,'absent destination retains intent and parks content');
    await command('Toggle target B');assert(await draft().evaluate(el=>el===window.__waitingPortal),'late registered destination adopts existing range');reports.push({suite:'portal-missing-destination-and-late-registration',status:'passed'});
    await page.goto(`${baseURL}/portal-hydrate-blue-light.html?manual`);await page.waitForFunction(()=>window.moduleReady===true);
    await draft().fill('Typed before adoption');await draft().evaluate(el=>{el.setSelectionRange(4,10);window.__earlyPortal=el});
    await page.evaluate(()=>demo.mount_portal(true,'blue','light'));await tick();
    assert(await draft().evaluate(el=>el===window.__earlyPortal&&el.value==='Typed before adoption'&&el.selectionStart===4&&el.selectionEnd===10),'dirty hydration preserves targeted native editor');
    reports.push({suite:'portal-dirty-adoption-keeps-native-editor',status:'passed'});
    for(const mode of ['csr','hydrate']){
      await open(`${mode==='csr'?'portal-missing':'portal-missing-hydrate-blue-light'}.html`);
      assert(await draft().count()===0&&(await status()).includes('factories=1'),'initial missing target keeps one suspended source');
      await command('Toggle target A');await draft().waitFor();assert((await status()).includes('factories=1'),'initial parked range resumes without reinvoking factory');
      reports.push({suite:`portal-initial-missing-${mode}`,status:'passed'});
    }
    await open();await page.evaluate(()=>{document.querySelector('input').dispatchEvent(new CompositionEvent('compositionstart',{data:'',bubbles:true}));demo.unmount_display()});await tick();assert(await page.locator('.portal-content').count()===0,'whole App retires target/source and pending input');
    await page.evaluate(()=>demo.mount_portal(false,'blue','light'));await tick();assert((await status()).includes('factories=1')&&(await status()).includes('losses=0'),'new App has no retired context history');reports.push({suite:'portal-app-retirement-and-fresh-context',status:'passed'});
  }catch(error){errors.push(String(error));}finally{await context.close()}
  return {reports,errors};
}
