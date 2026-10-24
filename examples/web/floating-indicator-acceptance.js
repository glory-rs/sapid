async(hostPage)=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1440,height:1100},reducedMotion:'no-preference'});
  const reports=[],errors=[],base=process.env.SAPID_FLOATING_BASE||'http://127.0.0.1:8794';const assert=(value,message)=>{if(!value)throw new Error(message)};
  try{
    const page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));page.on('console',message=>{if(message.type()==='error')errors.push(message.text())});
    const turn=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    const path=()=>page.locator('.sapid-floating-indicator-shape path');const marker=()=>page.locator('.sapid-floating-indicator-marker');
    const target=key=>page.locator(`.floating-target-${key}`);const editor=()=>page.locator('.floating-draft');
    const command=async name=>{await page.getByRole('button',{name,exact:true}).evaluate(element=>element.click());await turn();await turn()};
    const settled=async()=>{await page.waitForFunction(()=>{const element=document.querySelector('.sapid-floating-indicator-shape path');return element&&element.getAnimations().length===0});await turn()};
    const aligned=async key=>page.evaluate(key=>{
      const shape=document.querySelector('.sapid-floating-indicator-shape path').getBoundingClientRect();const item=document.querySelector(`.floating-target-${key}`).getBoundingClientRect();
      return ['x','y','width','height'].every(property=>Math.abs(shape[property]-item[property])<1.1);
    },key);
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]){
      const mode=`floating-indicator-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
      await page.setViewportSize({width:1440,height:1100});await page.emulateMedia({reducedMotion:'no-preference'});
      await page.goto(`${base}/${hydrate?`floating-indicator-hydrate-${hue}-${scheme}.html`:`floating-indicator.html?theme=${hue}&scheme=${scheme}`}`);
      await page.waitForFunction(()=>window.ready===true);await page.waitForFunction(()=>!document.querySelector('.sapid-floating-indicator-marker')?.hidden);await settled();
      assert(await aligned('one'),`${mode}: first measured outline does not match its actual target`);
      assert(await path().evaluate(element=>element.getAnimations().length===0)&&await page.locator('.sapid-floating-indicator-shape').getAttribute('aria-hidden')==='true',`${mode}: initial animation or duplicate accessible image`);
      await editor().fill('Keep 😀 indicator focus');await editor().evaluate(element=>{window.savedFloatingEditor=element;element.focus();element.setSelectionRange(2,8)});
      await target('one').evaluate(element=>window.savedFloatingOne=element);await path().evaluate(element=>window.savedFloatingPath=element);
      const first=await path().evaluate(element=>getComputedStyle(element).d);
      await command('Select teams');await page.waitForTimeout(35);const middle=await path().evaluate(element=>getComputedStyle(element).d);
      assert(middle!==first,`${mode}: no measured path interpolation`);await settled();const second=await path().evaluate(element=>getComputedStyle(element).d);
      assert(second!==middle&&second!==first&&await aligned('two'),`${mode}: path jumped or ended at stale geometry`);
      assert(await editor().evaluate(element=>element===window.savedFloatingEditor&&element===document.activeElement&&element.selectionStart===2&&element.selectionEnd===8),`${mode}: decoration changed actual focus/selection`);
      await command('Select projects');await page.waitForTimeout(35);const reversing=await path().evaluate(element=>getComputedStyle(element).d);
      assert(reversing!==first&&reversing!==second,`${mode}: reverse transition missing`);
      await command('Select teams');const interrupted=await path().evaluate(element=>getComputedStyle(element).d);
      assert(interrupted!==first&&interrupted!==second,`${mode}: interrupted path restarted at an endpoint`);
      await page.emulateMedia({reducedMotion:'reduce'});await turn();await settled();assert(await aligned('two'),`${mode}: dynamic Reduce did not settle current measured target`);
      await page.emulateMedia({reducedMotion:'no-preference'});await command('Select projects');await page.waitForTimeout(35);await command('Toggle indicator motion');await settled();
      assert(await aligned('one')&&await path().evaluate(element=>getComputedStyle(element).transitionProperty==='none'),`${mode}: Clear retained old interpolation`);
      await command('Toggle indicator motion');await command('Set indicator duration zero');await command('Select draft');await settled();assert(await aligned('draft'),`${mode}: explicit zero duration did not immediately follow target`);
      await command('Toggle indicator direction');await settled();assert(await aligned('draft'),`${mode}: vertical direction used stale local origin`);
      await command('Reverse indicator targets');await settled();assert(await aligned('draft')&&await editor().evaluate(element=>element===window.savedFloatingEditor&&element===document.activeElement),`${mode}: keyed direction/reorder changed owner or actual focus`);
      await command('Select projects');await command('Toggle target label');await settled();assert(await aligned('one'),`${mode}: live intrinsic label Resize did not update outline size`);
      await command('Toggle indicator visibility');assert(await marker().getAttribute('hidden')!==null,`${mode}: explicit hidden decoration retained paint`);
      await command('Toggle indicator visibility');await settled();assert(await aligned('one')&&await path().evaluate(element=>element.getAnimations().length===0),`${mode}: reappearance animated from hidden stale geometry`);
      await command('Remove projects target');await page.waitForFunction(()=>document.querySelector('.sapid-floating-indicator-marker')?.hidden);assert(await target('one').count()===0,`${mode}: removed registration still mounted`);
      await command('Restore projects target');await page.waitForFunction(()=>!document.querySelector('.sapid-floating-indicator-marker')?.hidden);await settled();
      assert(await target('one').evaluate(element=>element!==window.savedFloatingOne)&&await aligned('one'),`${mode}: retired target identity/geometry was reused`);
      assert(await path().evaluate(element=>element===window.savedFloatingPath),`${mode}: updating decoration replaced SVG identity`);
      const cdp=await context.newCDPSession(page),ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();
      assert(!ax.nodes.some(node=>!node.ignored&&node.role?.value==='image')&&await page.locator('.demo-floating-indicator [aria-selected]').count()===0,`${mode}: decoration invented selection or image semantics`);
      await page.setViewportSize({width:390,height:900});await turn();await settled();assert(await aligned('one')&&await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${mode}: mobile geometry/overflow`);
      if(hydrate)await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});
      await page.evaluate(()=>window.demo.unmount_display());await turn();assert(await page.locator('#app').textContent()==='',`${mode}: owner retirement left visible content`);
      reports.push({mode,measuredTarget:true,intermediateMotion:true,interruption:true,midflightReduce:true,focusUnchanged:true,currentRegistrations:true,physicalDirections:true});
    }
    await page.goto(`${base}/floating-indicator-hydrate-blue-light.html?manual=1`);await page.waitForFunction(()=>window.moduleReady===true);
    assert(await marker().getAttribute('hidden')!==null,'SSR invented physical target geometry');await editor().evaluate(element=>{window.dirtyFloating=element;element.value='Dirty before indicator adoption';element.focus();element.setSelectionRange(1,7)});
    await page.evaluate(()=>window.demo.mount_floating_indicator(true,'blue','light'));await page.waitForFunction(()=>!document.querySelector('.sapid-floating-indicator-marker')?.hidden);await settled();
    assert(await editor().evaluate(element=>element===window.dirtyFloating&&element.value==='Dirty before indicator adoption'&&element===document.activeElement&&element.selectionStart===1&&element.selectionEnd===7),'adoption/first observation changed dirty editor or focus');
    await page.evaluate(()=>window.demo.unmount_display());reports.push({mode:'floating-indicator-dirty-adoption',deterministicUnmeasuredShape:true,dirtyFocus:true});
    assert(errors.length===0,errors.join('\n'));return{reports,errors};
  }finally{await context.close()}
}
