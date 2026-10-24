async (hostPage) => {
 const context=await hostPage.context().browser().newContext({viewport:{width:1200,height:900}}),reports=[],errors=[];
 const assert=(value,message)=>{if(!value)throw new Error(message)};
 try{
  const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
  const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
  const status=()=>page.locator('.shell-status').textContent();
  for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydration of [false,true]){
   await page.setViewportSize({width:1200,height:900});
   const mode=`shell-${hue}-${scheme}-${hydration?'hydrate':'csr'}`;
   await page.goto(`http://127.0.0.1:8794/${hydration?`shell-hydrate-${hue}-${scheme}.html`:`shell.html?theme=${hue}&scheme=${scheme}`}`);
   await page.waitForFunction(()=>window.ready===true);await page.waitForFunction(()=>document.querySelector('.shell-status').textContent.includes('mobile=false'));
   assert(await page.getByRole('banner').count()===1&&await page.getByRole('contentinfo').count()===1&&await page.getByRole('main').count()===1&&await page.getByRole('navigation').count()===1,`${mode}: actual landmarks missing`);
   const nav=page.getByRole('navigation',{name:'Workspace navigation'}), draft=nav.getByRole('textbox',{name:'Sidebar draft'}),trigger=page.locator('.sapid-sidebar-trigger');
   await draft.fill('Kept across layout moves');await draft.evaluate(e=>window.sidebarDraft=e);await nav.evaluate(e=>window.sidebarNav=e);
   await trigger.click();await tick();assert((await status()).includes('collapsed=true'),`${mode}: collapse request failed`);
   assert(Math.abs((await page.locator('.sapid-sidebar-dock').boundingBox()).width-72)<1,`${mode}: collapsed rail is not 72px`);
   await page.evaluate(()=>window.demo.shell_policy(false));await trigger.click();await tick();assert((await status()).includes('collapsed=true'),`${mode}: rejected controlled collapse changed source`);
   const requests=(await status()).match(/requests=(\d+)/)[1];await page.evaluate(()=>window.demo.shell_controls(false,false));await tick();assert((await status()).includes('collapsed=false')&&(await status()).includes(`requests=${requests}`),`${mode}: external write emitted a request`);
   await page.evaluate(()=>window.demo.shell_policy(true));
   await page.getByRole('link',{name:'Skip to main content'}).click();await tick();assert(await page.getByRole('main').evaluate(e=>document.activeElement===e)&&!new URL(page.url()).hash,`${mode}: skip link did not use typed focus`);
   await nav.getByRole('link',{name:'Projects',exact:true}).click();await tick();assert(page.url().includes('shell')&&await nav.getByRole('link',{name:'Recent projects'}).count()===0,`${mode}: navigation callback routed or disclosure activated`);
   await nav.getByRole('button',{name:'Expand Projects'}).click();await tick();assert(await nav.getByRole('link',{name:'Recent projects'}).isVisible(),`${mode}: separate navigation disclosure failed`);
   await draft.focus();await page.setViewportSize({width:520,height:900});await page.waitForFunction(()=>document.querySelector('.shell-status').textContent.includes('mobile=true'));await tick();
   assert(await trigger.evaluate(e=>document.activeElement===e),`${mode}: hidden navigation did not restore actual focus to trigger`);
   assert(await page.locator('.sapid-sidebar-nav').evaluate(e=>e===window.sidebarNav)&&await page.locator('.shell-draft').evaluate(e=>e===window.sidebarDraft&&e.value==='Kept across layout moves'),`${mode}: responsive move replaced sidebar DOM or draft`);
   await trigger.press('Enter');await tick();const drawer=page.getByRole('dialog',{name:'Workspace navigation'});
   assert(await drawer.isVisible()&&await drawer.getByRole('button',{name:'Close navigation'}).evaluate(e=>document.activeElement===e),`${mode}: host layer initial close focus missing`);
   const focusable=drawer.locator('button,a,input');await focusable.last().focus();await page.keyboard.press('Tab');await tick();assert(await drawer.evaluate(e=>e.contains(document.activeElement)),`${mode}: host modal Tab escaped drawer`);
   await page.evaluate(()=>window.demo.shell_policy(false));await page.keyboard.press('Escape');await tick();assert(await drawer.isVisible(),`${mode}: rejected controlled dismissal closed drawer`);
   await page.evaluate(()=>window.demo.shell_policy(true));await page.keyboard.press('Escape');await tick();assert(await drawer.count()===0&&await trigger.evaluate(e=>document.activeElement===e),`${mode}: host Escape/return focus failed`);
   await trigger.click();await tick();await page.mouse.click(480,300);await tick();assert(await drawer.count()===0,`${mode}: outside dismissal failed`);
   await trigger.click();await tick();await page.locator('.shell-draft').focus();await page.setViewportSize({width:1200,height:900});await page.waitForFunction(()=>document.querySelector('.shell-status').textContent.includes('mobile=false'));await tick();
   assert(await draft.evaluate(e=>e===window.sidebarDraft&&document.activeElement===e&&e.value==='Kept across layout moves'),`${mode}: visible navigation move lost focused input identity/state`);
   assert((await status()).includes('creations=1')&&(await status()).includes('ordered=true')&&(await status()).includes('resize_ordered=true'),`${mode}: content recreated or generic callback preceded typed behavior`);
   assert(await page.locator('.sapid-sidebar-nav [aria-selected]').count()===0&&await page.locator('.sapid-sidebar-nav [aria-current=page]').count()===1,`${mode}: current navigation misrepresented as selection`);
   const cdp=await context.newCDPSession(page),ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();assert(ax.nodes.some(n=>n.role?.value==='navigation'&&n.name?.value==='Workspace navigation')&&ax.nodes.some(n=>n.role?.value==='main'),`${mode}: AX landmarks missing`);
   await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});reports.push({suite:mode,status:'passed',checks:'real landmarks, controlled collapse/rail, silent source writes, typed skip focus, separate route/disclosure, real Resize takeover, one navigation tree/draft/Owner, host modal Enter/Tab/Escape/outside/return focus, controlled dismissal, actual focus across moves, current-page AX, typed callback order'});
  }
  await page.setViewportSize({width:520,height:900});await page.goto('http://127.0.0.1:8794/shell-hydrate-green-dark.html?manual');await page.waitForFunction(()=>window.moduleReady===true);
  const draft=page.locator('.shell-draft');await draft.fill('Before mobile hydration');await draft.evaluate(e=>{window.preShell=e;e.focus();e.setSelectionRange(2,8,'backward')});
  await page.evaluate(()=>window.demo.mount_shell(true,'green','dark'));await page.waitForFunction(()=>document.querySelector('.shell-status').textContent.includes('mobile=true'));await tick();
  assert(await draft.evaluate(e=>e===window.preShell&&e.value==='Before mobile hydration'),'mobile hydration replaced the server navigation draft');
  await page.locator('.sapid-sidebar-trigger').click();await tick();assert(await draft.inputValue()==='Before mobile hydration'&&(await status()).includes('creations=1'),'mobile hydration recreated content');
  await page.evaluate(()=>window.demo.unmount_shell());await tick();assert(await page.locator('.shell-demo').count()===0,'shell Owner retirement leaked content');reports.push({suite:'shell-mobile-dirty-hydration-retirement',status:'passed'});
  return{reports,errors};
 }finally{await context.close()}
}
