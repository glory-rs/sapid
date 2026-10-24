async(hostPage)=>{
 const context=await hostPage.context().browser().newContext({viewport:{width:1440,height:1000}});const reports=[],errors=[];const assert=(ok,message)=>{if(!ok)throw new Error(message)};
 try {const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});const tick=()=>page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
 for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]) {
 const mode=`layout-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;await page.goto(`http://127.0.0.1:8794/${hydrate?`layout-hydrate-${hue}-${scheme}.html`:`layout.html?theme=${hue}&scheme=${scheme}`}`);await page.waitForFunction(()=>window.ready===true);
 const container=page.locator('.bounded-container'),child=page.locator('.container-child');await child.evaluate(e=>window.savedLayoutChild=e);
 let outer=await container.boundingBox(),inner=await child.boundingBox();assert(Math.abs(inner.width-480)<1,`${mode}: max width`);assert(Math.abs(inner.x+inner.width/2-outer.x-outer.width/2)<1,`${mode}: centered cap`);
 const boxes=await page.locator('.wrapping-group > div').evaluateAll(nodes=>nodes.map(e=>{const r=e.getBoundingClientRect();return {x:r.x,y:r.y,width:r.width,height:r.height}}));assert(boxes.length===3&&Math.abs(boxes[0].y-boxes[1].y)<1&&Math.abs(boxes[2].y-boxes[0].y-boxes[0].height-10)<1,`${mode}: actual wrap and gap`);
 const center=await page.locator('.centered-box').boundingBox(),centerChild=await page.locator('.center-child').boundingBox();assert(Math.abs(centerChild.y+centerChild.height/2-center.y-center.height/2)<1&&Math.abs(centerChild.x+centerChild.width/2-center.x-center.width/2)<1,`${mode}: two axis center`);
 assert((await page.locator('.explicit-space').boundingBox()).height===24,`${mode}: explicit space`);
 await page.getByRole('button',{name:'Toggle layout padding',exact:true}).click();await tick();assert(await child.evaluate(e=>e===window.savedLayoutChild),`${mode}: stable child after style patch`);
 await page.setViewportSize({width:390,height:844});await tick();outer=await container.boundingBox();inner=await child.boundingBox();assert(inner.width<=outer.width-39&&inner.width>0,`${mode}: responsive cap and changed padding`);assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${mode}: mobile overflow`);
 await page.setViewportSize({width:1440,height:1000});await tick();if(hydrate)await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});await page.evaluate(()=>window.demo.unmount_display());assert(await page.locator('#app').evaluate(e=>e.childElementCount===0),`${mode}: teardown`);reports.push({suite:mode,status:'passed'});
 }
 assert(errors.length===0,JSON.stringify(errors));return {reports,errors};}finally{await context.close()}
}