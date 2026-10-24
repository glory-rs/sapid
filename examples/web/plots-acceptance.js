async hostPage=>{
 const browser=hostPage.context().browser(),reports=[],errors=[];
 const base=process.env.SAPID_PLOTS_BASE||'http://127.0.0.1:8794';
 const assert=(ok,m)=>{if(!ok)throw new Error(m)};
 const context=await browser.newContext({viewport:{width:1440,height:1100}});
 const pages=['polar-charts','point-charts','shape-charts','matrix-charts','range-charts','hierarchy-charts'];
 try{
  const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
  const turn=()=>page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(()=>requestAnimationFrame(r)))));
  const primary=()=>page.locator('.plot-primary');
  for(const slug of pages)for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]){
   const mode=`${slug}-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
   await page.goto(`${base}/${hydrate?`${slug}-hydrate-${hue}-${scheme}.html`:`${slug}.html?theme=${hue}&scheme=${scheme}`}`);await page.waitForFunction(()=>window.ready===true);await turn();
   assert(await primary().locator('.sapid-plot-point').count()>0,`${mode}: finite real point controls`);
   assert(await page.locator('.plot-secondary').count()===2,`${mode}: all three distinct chart kinds`);
   assert(await primary().getByRole('table').count()>=1,`${mode}: intrinsic exact value tables`);
   const geometry=await primary().locator('.sapid-plot-drawing').evaluate(e=>{const b=e.getBoundingClientRect();return{width:b.width,height:b.height,paths:[...e.querySelectorAll('svg path')].map(p=>p.getAttribute('d'))}});
   assert(geometry.width>250&&Math.abs(geometry.width/geometry.height-2)<.02&&geometry.paths.length>0&&geometry.paths.every(d=>d&&!/NaN|Infinity/.test(d)),`${mode}: real finite responsive vector geometry`);
   const cdp=await context.newCDPSession(page),ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();
   assert(ax.nodes.some(n=>!n.ignored&&n.role?.value==='table'),`${mode}: actual AX table`);
   if(slug==='range-charts')assert(ax.nodes.some(n=>!n.ignored&&n.role?.value==='progressbar'),`${mode}: read-only numeric progress range`);
   const next=primary().getByRole('button',{name:'Next data point',exact:true});await next.focus();await page.keyboard.press('Enter');await turn();
   assert(!(await primary().locator('.sapid-plot-detail').innerText()).includes('No data point selected'),`${mode}: real keyboard details`);
   await primary().getByRole('button',{name:'Clear data point',exact:true}).click();await turn();
   await page.getByRole('checkbox',{name:'Accept plot proposals',exact:true}).uncheck();await turn();await next.click();await turn();
   assert((await primary().locator('.sapid-plot-detail').innerText()).includes('No data point selected'),`${mode}: controlled refusal`);
   await page.getByRole('checkbox',{name:'Accept plot proposals',exact:true}).check();await turn();
   const mark=primary().locator('.sapid-plot-point').first();await mark.evaluate(e=>window.pointBefore=e);
   await page.getByRole('button',{name:'Replace current plot data',exact:true}).click();await turn();
   assert(await primary().locator('.sapid-plot-point').evaluateAll(es=>es.includes(window.pointBefore)),`${mode}: stable business key retains node`);
   assert(!(await primary().getByRole('table').innerText()).includes('NaN'),`${mode}: exact current numeric values`);
   if(slug==='polar-charts'||slug==='shape-charts'||slug==='range-charts'){
    const legend=primary().getByRole('button',{name:'Hide Updated North',exact:true});await legend.evaluate(e=>window.legendBefore=e);const count=await primary().locator('.sapid-plot-point').count();await legend.click();await turn();assert(await primary().locator('.sapid-plot-point').count()===count-1,`${mode}: actual legend visibility`);assert(await primary().getByRole('button',{name:'Show Updated North',exact:true}).evaluate(e=>e===window.legendBefore),`${mode}: retained legend identity`);await primary().getByRole('button',{name:'Show Updated North',exact:true}).click();await turn();
   }
   if(slug==='hierarchy-charts'){await next.click();await turn();await primary().getByRole('button',{name:'First child',exact:true}).click();await turn();assert((await primary().locator('.sapid-plot-detail').innerText()).includes('North:'),`${mode}: current child navigation`);await primary().getByRole('button',{name:'Parent node',exact:true}).click();await turn();assert((await primary().locator('.sapid-plot-detail').innerText()).includes('Updated Regions:'),`${mode}: current parent navigation`);}
   if(slug==='point-charts')assert((await page.locator('.plot-sample').innerText()).includes('16 of 1000000 sampled'),`${mode}: explicit source sampling disclosure`);
   await page.getByRole('checkbox',{name:'Enable plot controls',exact:true}).uncheck();await turn();assert(await next.isDisabled(),`${mode}: physical disabled controls`);
   await page.getByRole('checkbox',{name:'Enable plot controls',exact:true}).check();await turn();
   await page.getByRole('button',{name:'Empty plot data',exact:true}).click();await turn();assert(await primary().locator('.sapid-plot-point').count()===0&&(await primary().innerText()).includes('No visible plot data'),`${mode}: truthful empty data`);
   await page.getByRole('button',{name:'Restore plot data',exact:true}).click();await turn();assert(await primary().locator('.sapid-plot-point').count()>0,`${mode}: source replacement recovers`);
   await primary().locator('.sapid-plot-point').first().evaluate(e=>window.retiredPoint=e);await page.getByRole('button',{name:'Toggle plot owner',exact:true}).click();await turn();assert(await primary().count()===0&&await page.evaluate(()=>!window.retiredPoint.isConnected),`${mode}: owner retires controls`);
   await page.getByRole('button',{name:'Toggle plot owner',exact:true}).click();await turn();assert(await primary().locator('.sapid-plot-point').count()>0,`${mode}: fresh mounted owner`);
   reports.push({mode,vector:true,table:true,keyboard:true,controlled:true,stableKey:true,empty:true,cleanup:true});
  }
  for(const slug of pages){
   await page.goto(`${base}/${slug}-hydrate-blue-light.html?manual=1`);await page.waitForFunction(()=>window.moduleReady===true);const point=primary().locator('.sapid-plot-point').first();await point.evaluate(e=>{window.adoptedPoint=e;e.focus()});await page.evaluate(slug=>window.demo[`mount_${slug.replaceAll('-','_')}`](true,'blue','light'),slug);await turn();assert(await point.evaluate(e=>e===window.adoptedPoint&&document.activeElement===e),`${slug}: focused SSR point adopted`);reports.push({mode:`${slug}-focused-adoption`,identity:true,focus:true});
   await page.setViewportSize({width:390,height:844});await page.goto(`${base}/${slug}.html?theme=green&scheme=dark`);await page.waitForFunction(()=>window.ready===true);await turn();assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),`${slug}: no mobile overflow`);const box=await primary().locator('.sapid-plot-drawing').boundingBox();assert(box.width>200&&box.x>=0&&box.x+box.width<=390,`${slug}: mobile geometry`);await page.screenshot({path:`D:/BuildCache/sapid/${slug}-green-dark-mobile.png`,fullPage:true});reports.push({mode:`${slug}-mobile`,responsive:true});await page.setViewportSize({width:1440,height:1100});
  }
 }finally{await context.close()}
 return{reports,errors};
}
