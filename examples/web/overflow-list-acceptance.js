async (hostPage) => {
  const context=await hostPage.context().browser().newContext({viewport:{width:1440,height:1100}});
  const reports=[],errors=[],base=process.env.SAPID_OVERFLOW_BASE||'http://127.0.0.1:8794';
  const assert=(value,message)=>{if(!value)throw new Error(message)};
  try {
    const page=await context.newPage();
    page.on('pageerror',error=>errors.push(error.message));
    page.on('console',message=>{if(message.type()==='error')errors.push(message.text())});
    const settle=()=>page.evaluate(()=>new Promise((resolve,reject)=>{
      let previous='',stable=0,ticks=0;
      const turn=()=>{
        const text=document.querySelector('.overflow-list-status')?.textContent||'';
        stable=text===previous&&text.includes('pending: 0;')?stable+1:0;previous=text;
        if(stable>=4)return resolve();if(++ticks>240)return reject(new Error(`overflow did not settle: ${text}`));requestAnimationFrame(turn);
      };requestAnimationFrame(turn);
    }));
    const status=async()=>{
      const text=await page.locator('.overflow-list-status').textContent();
      const match=text.match(/Visible: (\d+); hidden: (\d+); pending: (\d+); factories: (\d+); cleaned: (\d+); proposals: (\d+); clicks: (\d+)/);
      assert(match,`malformed overflow status: ${text}`);
      const [visible,hidden,pending,factories,cleaned,proposals,clicks]=match.slice(1).map(Number);return{visible,hidden,pending,factories,cleaned,proposals,clicks};
    };
    const control=name=>page.getByRole('button',{name,exact:true});
    const change=async name=>{await control(name).evaluate(element=>element.click());await settle()};
    const more=()=>page.locator('.sapid-overflow-more .sapid-popover-trigger');
    const draft=()=>page.locator('.overflow-draft');
    const panel=()=>page.locator('.sapid-overflow-more .sapid-popover');
    const fits=async()=>page.locator('.sapid-overflow-line').evaluate(line=>{
      const box=line.getBoundingClientRect();const strip=line.querySelector('.sapid-overflow-strip');
      const items=[...strip.children].map(item=>item.getBoundingClientRect());const more=line.querySelector('.sapid-overflow-more');
      return items.every(item=>item.left>=box.left-1&&item.right<=box.right+1)&&(!more||more.hasAttribute('inert')||more.getBoundingClientRect().right<=box.right+1);
    });
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]){
      const mode=`overflow-list-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
      await page.setViewportSize({width:1440,height:1100});
      await page.goto(`${base}/${hydrate?`overflow-list-hydrate-${hue}-${scheme}.html`:`overflow-list.html?theme=${hue}&scheme=${scheme}`}`);
      await page.waitForFunction(()=>window.ready===true);await settle();
      const initial=await status();assert(initial.factories===5&&initial.cleaned===0&&initial.pending===0,`${mode}: initial Owners or pending work`);
      assert(await fits(),`${mode}: initial packing exceeds measured content width`);
      assert(await page.locator('.sapid-overflow-measure').getAttribute('inert')!==null&&await page.locator('.sapid-overflow-measure').evaluate(element=>getComputedStyle(element).opacity==='0'&&!element.hidden),`${mode}: measurement lane lost layout or inertness`);
      await draft().evaluate(element=>{window.savedOverflowEditor=element;element.value='Retained 😀 overflow draft';element.setSelectionRange(2,8)});
      await change('Narrow list');const narrow=await status();assert(narrow.visible<initial.visible&&narrow.hidden>0&&await fits(),`${mode}: narrowing failed to pack a smaller prefix`);
      await more().focus();await page.keyboard.press('Enter');await settle();
      assert(await panel().evaluate(element=>element.matches(':popover-open'))&&await panel().getAttribute('role')==='dialog',`${mode}: overflow must be the existing real Popover`);
      assert(await page.getByRole('textbox',{name:'Overflow draft',exact:true}).count()===1,`${mode}: overflow editor missing from accessibility exposure`);
      await draft().focus();await draft().evaluate(element=>element.setSelectionRange(2,8));
      await change('Widen list');const wide=await status();
      assert(wide.visible>narrow.visible&&wide.factories===5&&wide.cleaned===0&&await fits(),`${mode}: growing capacity reconstructed current items`);
      assert(await draft().evaluate(element=>element===window.savedOverflowEditor&&element.value==='Retained 😀 overflow draft'&&element===document.activeElement&&element.selectionStart===2&&element.selectionEnd===8),`${mode}: Popover to strip move lost identity/value/selection/actual focus`);
      await change('Narrow list');
      assert(await more().evaluate(element=>element===document.activeElement),`${mode}: moved focused item did not hand focus to the sole overflow entry`);
      await page.keyboard.press('Enter');await settle();await draft().focus();
      await page.keyboard.press('Escape');await settle();
      assert(await more().evaluate(element=>element===document.activeElement)&&!(await panel().evaluate(element=>element.matches(':popover-open'))),`${mode}: Escape did not restore the live overflow entry`);
      const cdp=await context.newCDPSession(page),ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();
      assert(!ax.nodes.some(node=>!node.ignored&&node.role?.value==='textbox'&&node.name?.value==='Overflow draft'),`${mode}: closed overflow/measurement editor remained in Chrome AX`);
      await change('Toggle action label');assert((await status()).factories===5&&await fits(),`${mode}: hidden label refresh cloned widgets or packed stale widths`);
      await change('Reverse items');assert((await status()).factories===5&&await draft().evaluate(element=>element===window.savedOverflowEditor),`${mode}: reorder replaced a retained key`);
      await change('Remove draft');assert(await draft().count()===0&&(await status()).cleaned===1,`${mode}: removed member did not retire its Owner exactly once`);
      await change('Restore draft');assert(await draft().evaluate(element=>element!==window.savedOverflowEditor)&&(await status()).factories===6,`${mode}: reinsert reused a retired identity`);
      await change('Widen list');await change('Toggle capacity acceptance');const refused=await status();await change('Narrow list');const result=await status();
      assert(result.visible===refused.visible&&result.proposals>refused.proposals,`${mode}: controlled capacity refusal changed accepted prefix`);
      await change('Toggle capacity acceptance');await change('Widen list');await change('Narrow list');assert(await fits(),`${mode}: resumed acceptance failed to remeasure`);
      await change('Toggle list enabled');assert(await page.locator('.demo-overflow-list').getAttribute('inert')!==null,`${mode}: disabled component did not block its whole subtree`);
      await change('Toggle list enabled');await page.setViewportSize({width:390,height:900});await settle();
      assert(await fits()&&await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${mode}: mobile overflow`);
      if(hydrate)await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});
      await page.evaluate(()=>window.demo.unmount_display());await page.waitForTimeout(80);
      assert(await page.locator('#app').textContent()==='',`${mode}: Owner cleanup left physical nodes`);
      reports.push({mode,initial:initial.visible,narrow:narrow.visible,wide:wide.visible,currentOwners:true,focusHandoff:true,dirtyEditor:true,closedAX:true,controlledRefusal:true});
    }
    await page.goto(`${base}/overflow-list-hydrate-blue-light.html?manual=1`);await page.waitForFunction(()=>window.moduleReady===true);
    const before=await page.locator('.overflow-list-status').textContent();assert(before.includes('Visible: 2; hidden: 3; pending: 0; factories: 5'),'SSR did not preserve explicit initial prefix');
    await draft().evaluate(element=>{window.beforeOverflowHydrate=element;element.value='Dirty before adoption';element.setSelectionRange(1,7)});
    await page.evaluate(()=>window.demo.mount_overflow_list(true,'blue','light'));await settle();
    assert(await draft().evaluate(element=>element===window.beforeOverflowHydrate&&element.value==='Dirty before adoption'&&element.selectionStart===1&&element.selectionEnd===7),'adoption/measurement replaced dirty editor');
    await page.evaluate(()=>window.demo.unmount_display());reports.push({mode:'overflow-list-dirty-adoption',deterministicPrefix:true,dirtyIdentity:true});
    assert(errors.length===0,errors.join('\n'));return{reports,errors};
  }finally{await context.close()}
}
