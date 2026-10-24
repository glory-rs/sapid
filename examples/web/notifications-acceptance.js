async(hostPage)=>{
 const context=await hostPage.context().browser().newContext({viewport:{width:1440,height:1000}}),reports=[],errors=[];
 const assert=(v,m)=>{if(!v)throw new Error(m)};
 try{const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',m=>{if(m.type()==='error')errors.push(m.text())});
 const tick=()=>page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
 const click=async name=>{await page.getByRole('button',{name,exact:true}).click();await tick()};
 const pending=async n=>page.waitForFunction(n=>document.querySelector('.notifications-pending')?.textContent===`Pending: ${n}`,n);
 for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]){
  const mode=`notifications-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
  await page.goto(`http://127.0.0.1:8794/${hydrate?`notifications-hydrate-${hue}-${scheme}.html`:`notifications.html?theme=${hue}&scheme=${scheme}`}`);await page.waitForFunction(()=>window.ready===true);
  const outside=page.getByRole('textbox',{name:'Outside notification editor',exact:true});await outside.fill('Accepted outside draft');await outside.evaluate(e=>window.notificationOutside=e);
  const opener=page.getByRole('button',{name:'Show persistent notification',exact:true});await opener.click();await pending(1);
  const saved=page.getByRole('status',{name:'Saved notification',exact:true});await saved.waitFor();await saved.evaluate(e=>window.notificationRow=e);
  assert(await opener.evaluate(e=>e===document.activeElement),`${mode}: message stole focus`);
  const cdp=await context.newCDPSession(page),ax=await cdp.send('Accessibility.getFullAXTree');assert(ax.nodes.some(n=>!n.ignored&&n.role?.value==='status'&&n.name?.value==='Saved notification'&&n.properties?.some(p=>p.name==='live'&&p.value?.value==='polite')),`${mode}: polite AX message`);await cdp.detach();
  await click('Update saved notification');const updated=page.getByRole('status',{name:'Updated notification',exact:true});await updated.waitFor();assert(await updated.evaluate(e=>e===window.notificationRow),`${mode}: metadata replaced row`);
  await click('Replace saved notification');const replacement=page.getByRole('status',{name:'Replacement notification',exact:true});await replacement.waitFor();await pending(1);assert(await replacement.evaluate(e=>e!==window.notificationRow),`${mode}: replacement kept revoked lifetime`);assert(await page.locator('.notifications-result').textContent()==='Replaced',`${mode}: replacement completion`);
  await replacement.getByRole('button',{name:'Undo save',exact:true}).click();await pending(0);assert(await page.locator('.notifications-actions').textContent()==='Actions: 1'&&await page.locator('.notifications-result').textContent()==='Action',`${mode}: action once`);
  await click('Show persistent notification');await page.getByRole('button',{name:'Dismiss notification',exact:true}).click();await pending(0);assert(await page.locator('.notifications-result').textContent()==='Dismissed',`${mode}: independent dismiss`);
  await click('Show persistent notification');await click('Clear notifications');await pending(0);assert(await outside.evaluate(e=>e===window.notificationOutside)&&await outside.inputValue()==='Accepted outside draft',`${mode}: outside editor retained`);
  await page.setViewportSize({width:390,height:844});await click('Show persistent notification');assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${mode}: mobile overflow`);await page.setViewportSize({width:1440,height:1000});await click('Clear notifications');
  if(hue==='green'&&scheme==='dark'&&hydrate){await click('Show persistent notification');await page.screenshot({path:'D:/BuildCache/sapid/notifications-green-dark-hydrate.png',fullPage:true});await click('Clear notifications')}
  await page.evaluate(()=>window.demo.unmount_display());assert(await page.locator('#app').evaluate(e=>e.childNodes.length===0),`${mode}: Owner retirement`);reports.push({suite:mode,status:'passed'});
 }
 await page.goto('http://127.0.0.1:8794/notifications.html');await page.waitForFunction(()=>window.ready===true);
 await click('Show timed notification');const timed=page.getByRole('status',{name:'Timed notification',exact:true});await timed.waitFor();await timed.hover();await page.waitForTimeout(1100);assert(await timed.isVisible(),'hover did not pause timeout');
 await timed.getByRole('button',{name:'Dismiss notification',exact:true}).focus();await page.mouse.move(8,8);await page.waitForTimeout(1100);assert(await timed.isVisible(),'focus did not retain pause after pointer leave');await page.getByRole('textbox',{name:'Outside notification editor',exact:true}).focus();await timed.waitFor({state:'hidden',timeout:2500});await pending(0);
 await click('Show timed notification');await click('Pause notifications');await page.waitForTimeout(1100);assert(await timed.isVisible(),'manual pause lost residual lifetime');await click('Resume notifications');await timed.waitFor({state:'hidden',timeout:2500});await pending(0);
 await click('Queue three timed notifications');await pending(3);assert(await page.locator('.sapid-notification-row').count()===2,'visible bound');await page.getByRole('textbox',{name:'Outside notification editor',exact:true}).focus();await page.waitForFunction(()=>document.querySelector('.notifications-pending')?.textContent==='Pending: 1');assert(await page.getByRole('status',{name:'Queued notification 3',exact:true}).isVisible(),'queue promotion');await pending(0);reports.push({suite:'notifications-real-clock-hover-focus-pause-and-promotion',status:'passed'});
 await click('Show timed notification');await page.evaluate(()=>window.demo.unmount_display());await page.waitForTimeout(1100);assert(await page.locator('#app').evaluate(e=>e.childNodes.length===0),'late timer resurrected retired Owner');reports.push({suite:'notifications-retirement-cancels-timer',status:'passed'});
 assert(errors.length===0,errors.join('\n'));return {reports,errors};
 }finally{await context.close()}
}
