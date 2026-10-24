async(hostPage,baseURL='http://127.0.0.1:8794')=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1100,height:850}}),reports=[],errors=[];
  const assert=(value,message)=>{if(!value)throw new Error(message)};
  try{
    const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
    const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    const button=name=>page.getByRole('button',{name,exact:true});
    const panel=()=>page.locator('.sapid-tour');
    const opened=()=>page.waitForFunction(()=>document.querySelector('.sapid-tour')?.matches(':popover-open'));
    const closed=()=>page.waitForFunction(()=>!document.querySelector('.sapid-tour')?.matches(':popover-open'));
    const status=()=>page.locator('.tour-status').textContent();
    const activate=async locator=>{await locator.focus();await locator.press('Enter');await tick()};
    const open=async(path='tour.html')=>{await page.goto(`${baseURL}/${path}`);await page.waitForFunction(()=>window.ready===true);await tick()};
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const mode of ['csr','hydrate']){
      await open(`${mode==='csr'?'tour':`tour-hydrate-${hue}-${scheme}`}.html?theme=${hue}&scheme=${scheme}`);
      assert(!(await panel().isVisible()),'closed initial shape');
      await page.getByRole('textbox',{name:'Tour draft',exact:true}).fill('Keep my selected draft');
      await page.getByRole('textbox',{name:'Tour draft',exact:true}).evaluate(el=>el.setSelectionRange(3,8));
      await page.evaluate(()=>{window.__tourPanel=document.querySelector('.sapid-tour');window.__tourEditor=document.querySelector('.sapid-tour-target input')});
      await button('Start tour').click();await opened();
      assert((await panel().ariaSnapshot()).includes('dialog "First step"'),'real Dialog title');
      assert(await button('Previous step').isDisabled(),'first previous is actually disabled');
      assert(await page.evaluate(()=>!!document.activeElement?.closest('.sapid-tour')),'focus entered current layer');
      await button('Next step').click();await page.getByRole('dialog',{name:'Second step',exact:true}).waitFor();await opened();await tick();
      assert((await status()).includes('step=1'),'next source accepted');
      const target=await button('Second real target').boundingBox();
      assert(target.y>=0&&target.y+target.height<=851,'current mounted target physically revealed');
      assert(await page.evaluate(()=>scrollY>300),'actual viewport scroll changed');
      assert(await page.evaluate(()=>window.__tourPanel===document.querySelector('.sapid-tour')&&window.__tourEditor===document.querySelector('.sapid-tour-target input')),'stable panel and editor identities');
      assert(await page.evaluate(()=>{const e=window.__tourEditor;return e.value==='Keep my selected draft'&&e.selectionStart===3&&e.selectionEnd===8}),'draft selection preserved through scroll and step');
      await button('Finish').click();await closed();
      assert((await status()).includes('completed=1'),'completion callback');
      assert(await page.evaluate(()=>document.activeElement?.classList.contains('sapid-tour-target')),'focus returned to actual step target');
      reports.push({suite:`tour-${hue}-${scheme}-${mode}`,status:'passed'});
    }
    await open();await button('Toggle step refusal').click();await button('Toggle close refusal').click();await button('Start tour').click();await opened();
    await button('Next step').click();await tick();assert((await status()).includes('step=0')&&(await status()).includes('changes=1'),'controlled current refusal');
    assert((await panel().ariaSnapshot()).includes('First step'),'no optimistic title/focus change');
    await button('Cancel tour').click();await tick();assert(await panel().isVisible(),'controlled close refusal keeps live layer');
    await activate(button('Toggle close refusal'));await button('Cancel tour').click();await closed();reports.push({suite:'tour-controlled-step-and-close-refusal',status:'passed'});
    await open();await button('Go to second target').click();await opened();const before=await panel().evaluate(el=>{window.__keptTour=el;return true});assert(before,'keep panel');
    await activate(button('Toggle target presence'));await closed();assert((await status()).includes('open=true')&&(await status()).includes('losses=1'),'target retirement hides guide without rewriting controlled open');
    await activate(button('Refresh target'));await tick();assert((await status()).includes('losses=1'),'missing target episode callback is bounded');
    await activate(button('Toggle target presence'));await opened();assert(await panel().evaluate(el=>el===window.__keptTour),'target re-registration recovers same guide');
    reports.push({suite:'tour-live-target-loss-and-recovery',status:'passed'});
    await open();await button('Start tour').click();await opened();await page.keyboard.press('Escape');await closed();assert((await status()).includes('open=false'),'Escape dismissal is a proposal accepted by source');
    await button('Go to second target').click();await opened();await button('Previous step').click();await page.getByRole('dialog',{name:'First step',exact:true}).waitFor();await opened();
    assert((await status()).includes('step=0'),'previous accepted');await button('Cancel tour').click();await closed();reports.push({suite:'tour-escape-previous-and-focus',status:'passed'});
    await open();await page.evaluate(()=>{document.querySelectorAll('button')[1].click();demo.unmount_display()});await tick();assert(await page.locator('.sapid-tour').count()===0,'retired Tour cannot accept late measurement');
    await page.evaluate(()=>demo.mount_tour(false,'blue','light'));await tick();await button('Start tour').click();await opened();assert((await status()).includes('step=0')&&(await status()).includes('losses=0'),'fresh Owner has no old replies or registry entries');
    reports.push({suite:'tour-owner-cancelled-task-query-and-remount',status:'passed'});
  }catch(error){errors.push(String(error));}finally{await context.close()}
  return {reports,errors};
}
