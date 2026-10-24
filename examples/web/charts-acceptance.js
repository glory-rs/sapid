async hostPage => {
 const browser=hostPage.context().browser(),reports=[],errors=[];
 const base=process.env.SAPID_CHARTS_BASE||'http://127.0.0.1:8794';
 const assert=(ok,message)=>{if(!ok)throw new Error(message)};
 const context=await browser.newContext({viewport:{width:1440,height:1100}});
 try {
  const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
  const turn=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))));
  const primary=()=>page.locator('.chart-primary');
  for(const slug of ['charts','cartesian'])for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]) {
   const mode=`${slug}-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
   await page.goto(`${base}/${hydrate?`${slug}-hydrate-${hue}-${scheme}.html`:`${slug}.html?theme=${hue}&scheme=${scheme}`}`);
   await page.waitForFunction(()=>window.ready===true);await turn();
   assert(await primary().locator('.sapid-chart-point').count()===11,`${mode}: gaps or current points`);
   assert(await primary().getByRole('table',{name:'Chart data'}).count()===1,`${mode}: actual intrinsic accessible table`);
   assert(await primary().getByRole('columnheader',{name:'Revenue',exact:true}).count()===1,`${mode}: real column headers`);
   assert((await primary().getByRole('table').innerText()).includes('No value'),`${mode}: null is not zero`);
   assert(await primary().locator('.sapid-chart-axis-tick').count()===5,`${mode}: finite visible scale labels`);
   const geometry=await primary().locator('.sapid-chart-plot').evaluate(plot=>{
    const box=plot.getBoundingClientRect();return {width:box.width,height:box.height,paths:[...plot.querySelectorAll('svg path')].map(p=>p.getAttribute('d'))};
   });
   assert(geometry.width>250&&Math.abs(geometry.width/geometry.height-2)<.02&&geometry.paths.length>=3&&geometry.paths.every(d=>d&&!/NaN|Infinity/.test(d)),`${mode}: responsive real finite SVG`);
   const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();
   assert(ax.nodes.some(n=>!n.ignored&&n.role?.value==='table'&&n.name?.value==='Chart data'),`${mode}: actual AX table`);
   const next=primary().getByRole('button',{name:'Next data point',exact:true});await next.focus();await page.keyboard.press('Enter');await turn();
   assert((await primary().locator('.sapid-chart-point-detail').innerText()).includes('January · Revenue: 12'),`${mode}: keyboard point access`);
   await page.getByRole('button',{name:'Inspect chart point',exact:true}).hover();await turn();
   assert((await page.getByRole('tooltip').innerText()).includes('January · Revenue: 12'),`${mode}: genuine tooltip content`);
   await next.focus();await turn();
   await primary().getByRole('button',{name:'Clear data point',exact:true}).click();await turn();
   await page.getByRole('checkbox',{name:'Accept chart proposals',exact:true}).uncheck();await turn();
   await next.click();await turn();assert((await primary().locator('.sapid-chart-point-detail').innerText()).includes('No data point selected'),`${mode}: controlled selection refusal`);
   const legend=primary().getByRole('button',{name:'Hide Revenue',exact:true});await legend.evaluate(e=>window.legendBefore=e);
   await legend.click();await turn();assert(await primary().locator('.sapid-chart-point').count()===11,`${mode}: controlled legend refusal`);
   await page.getByRole('checkbox',{name:'Accept chart proposals',exact:true}).check();await legend.click();await turn();
   assert(await primary().locator('.sapid-chart-point').count()===6,`${mode}: legend actual layer removal`);
   assert(await primary().getByRole('button',{name:'Show Revenue',exact:true}).evaluate(e=>e===window.legendBefore),`${mode}: stable legend identity`);
   await primary().getByRole('button',{name:'Show Revenue',exact:true}).click();await turn();
   await page.getByRole('button',{name:'Brush middle categories',exact:true}).click();await turn();
   assert(await primary().locator('.sapid-chart-point').count()===5,`${mode}: inclusive finite brush`);
   assert((await primary().getByRole('table').innerText()).includes('January'),`${mode}: table retains full source values beyond brush`);
   await page.getByRole('button',{name:'Clip chart scale',exact:true}).click();await turn();
   assert((await primary().locator('.sapid-chart-clipping').innerText()).includes('2 values clipped'),`${mode}: explicit clipping disclosure`);
   await page.getByRole('button',{name:'Reset chart window',exact:true}).click();await turn();
   const point=primary().getByRole('button',{name:'January · Revenue: 12',exact:true});await point.evaluate(e=>window.pointBefore=e);
   await page.getByRole('button',{name:'Replace current chart data',exact:true}).click();await turn();
   assert(await primary().getByRole('button',{name:'Updated January · Revenue: 7',exact:true}).evaluate(e=>e===window.pointBefore),`${mode}: stable full-key point identity and fresh numeric label`);
   await page.getByRole('button',{name:'Export chart SVG',exact:true}).click();await turn();
   assert(await page.locator('.chart-export').evaluate(e=>{const d=new DOMParser().parseFromString(e.textContent,'image/svg+xml');return !d.querySelector('parsererror')&&d.querySelectorAll('path').length>=3&&!d.querySelector('script');}),`${mode}: valid geometry export`);
   await page.getByRole('checkbox',{name:'Enable chart controls',exact:true}).uncheck();await turn();assert(await next.isDisabled(),`${mode}: physically disabled navigation`);
   await page.getByRole('button',{name:'Toggle chart owner',exact:true}).click();await turn();assert(await primary().count()===0&&await page.evaluate(()=>!window.pointBefore.isConnected),`${mode}: owner cleanup`);
   await page.getByRole('button',{name:'Toggle chart owner',exact:true}).click();await turn();assert(await primary().locator('.sapid-chart-point').count()===11,`${mode}: fresh owner finite source`);
   if(slug==='cartesian'){
    assert(await page.locator('.chart-mode-Area svg path[fill]:not([fill="none"])').count()>0,`${mode}: actual filled area`);
    assert(await page.locator('.chart-mode-Bar svg path[fill]:not([fill="none"])').count()>0,`${mode}: actual bars`);
    assert(await page.locator('.chart-stacked-area .sapid-chart-point').count()===11&&await page.locator('.chart-stacked-bar .sapid-chart-point').count()===11,`${mode}: both signed bar and positive area stacks`);
   }
   reports.push({mode,points:11,table:true,keyboard:true,tooltip:true,controlled:true,brush:true,export:true,cleanup:true});
  }
  for(const slug of ['charts','cartesian']) {
   await page.goto(`${base}/${slug}-hydrate-blue-light.html?manual=1`);await page.waitForFunction(()=>window.moduleReady===true);
   const point=primary().getByRole('button',{name:'January · Revenue: 12',exact:true});await point.evaluate(e=>{window.adoptedPoint=e;e.focus()});
   await page.evaluate(slug=>window.demo[slug==='charts'?'mount_charts':'mount_cartesian'](true,'blue','light'),slug);await turn();
   assert(await point.evaluate(e=>e===window.adoptedPoint&&document.activeElement===e),`${slug}: focused SSR control adopted intact`);
   reports.push({mode:`${slug}-focused-adoption`,identity:true,focus:true});
   await page.setViewportSize({width:390,height:844});await page.goto(`${base}/${slug}.html?theme=green&scheme=dark`);await page.waitForFunction(()=>window.ready===true);await turn();
   assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),`${slug}: narrow viewport overflow`);
   const rect=await primary().locator('.sapid-chart-plot').boundingBox();assert(rect.width>200&&rect.x>=0&&rect.x+rect.width<=390,`${slug}: actual mobile plot bounds`);
   await page.screenshot({path:`D:/BuildCache/sapid/${slug}-green-dark-mobile.png`,fullPage:true});reports.push({mode:`${slug}-mobile`,responsive:true});
   await page.setViewportSize({width:1440,height:1100});
  }
 } finally {await context.close()}
 return {reports,errors};
}
