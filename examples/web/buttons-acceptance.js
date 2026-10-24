async(hostPage)=>{
 const context=await hostPage.context().browser().newContext({viewport:{width:1440,height:1000}}),reports=[],errors=[];const assert=(value,message)=>{if(!value)throw new Error(message)};
 try{const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
 const tick=()=>page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
 for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]){
  const mode=`buttons-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;await page.goto(`http://127.0.0.1:8794/${hydrate?`buttons-hydrate-${hue}-${scheme}.html`:`buttons.html?theme=${hue}&scheme=${scheme}`}`);await page.waitForFunction(()=>window.ready===true);
  const button=page.getByRole('button',{name:'Save draft',exact:true});await button.evaluate(e=>window.savedButton={element:e,label:e.querySelector('.sapid-button-label'),left:e.querySelector('[data-gly-kind="text"]')});
  assert(await button.textContent()==='★Save draftCtrl+S',`${mode}: sections and label order`);
  const layout=await button.evaluate(e=>{const es=[...e.querySelectorAll('[data-gly-kind="text"]')].map(e=>e.getBoundingClientRect());return{display:e.style.display,gap:getComputedStyle(e).gap,left:es[1].left-es[0].right,right:es[2].left-es[1].right}});
  assert(layout.display==='inline-flex'&&layout.gap==='8px'&&Math.abs(layout.left-8)<.6&&Math.abs(layout.right-8)<.6,`${mode}: exactly one gap between each section ${JSON.stringify(layout)}`);
  assert((await page.locator('.button-xs').boundingBox()).height>=24&&(await page.locator('.button-xl').boundingBox()).height>=50,`${mode}: extra small and large actual boxes`);
  assert(await page.locator('.section-unstyled').evaluate(e=>e.style.display==='inline-flex'&&getComputedStyle(e).gap==='8px'),`${mode}: unstyled structural layout`);
  await button.focus();await page.keyboard.press('Space');await tick();assert(await page.locator('.button-order').innerText()==='typed;generic;',`${mode}: typed callback before generic observer`);
  await page.getByRole('button',{name:'Toggle button loading',exact:true}).click();await tick();assert(await button.isDisabled()&&await button.getAttribute('aria-busy')==='true',`${mode}: loading is busy and disabled`);
  const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');assert(ax.nodes.some(n=>!n.ignored&&n.role?.value==='button'&&n.name?.value==='Save draft'&&n.properties?.some(p=>p.name==='busy'&&p.value?.value===1)),`${mode}: actual AX busy button`);
  await button.click({force:true});await tick();assert(await page.locator('.button-order').innerText()==='typed;generic;',`${mode}: disabled does not activate`);
  await page.getByRole('button',{name:'Update button label',exact:true}).click();await tick();const updated=page.getByRole('button',{name:'Save changes',exact:true});assert(await updated.evaluate(e=>e===window.savedButton.element&&e.querySelector('.sapid-button-label')===window.savedButton.label&&e.querySelector('[data-gly-kind="text"]')===window.savedButton.left),`${mode}: live label and section identity`);
  await page.getByRole('button',{name:'Toggle button loading',exact:true}).click();await tick();assert(await updated.isEnabled()&&await updated.getAttribute('aria-busy')==='false',`${mode}: recover enabled`);await cdp.detach();
  await page.setViewportSize({width:390,height:844});assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${mode}: mobile overflow`);await page.setViewportSize({width:1440,height:1000});if(hydrate)await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});await page.evaluate(()=>window.demo.unmount_display());assert(await page.locator('#app').evaluate(e=>e.childNodes.length===0),`${mode}: teardown`);reports.push({suite:mode,status:'passed'});
 }assert(errors.length===0,JSON.stringify(errors));return{reports,errors};}finally{await context.close()}
}
