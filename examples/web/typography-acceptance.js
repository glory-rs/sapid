async (hostPage) => {
 const context=await hostPage.context().browser().newContext({viewport:{width:1100,height:1000}});const reports=[],errors=[];
 const assert=(value,message)=>{if(!value)throw new Error(message);};
 try{
  const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text());});
  const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
  for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydration of [false,true]){
   const mode=`typography-${hue}-${scheme}-${hydration?'hydrate':'csr'}`;
   const url=`http://127.0.0.1:8794/${hydration?`typography-hydrate-${hue}-${scheme}.html`:`typography.html?theme=${hue}&scheme=${scheme}`}`;
   await page.goto(url);await page.waitForFunction(()=>window.ready===true);
   assert(await page.getByRole('heading',{name:'Readable by design',level:1}).count()===1,`${mode}: h1 semantics missing`);
   assert(await page.getByRole('heading',{name:'Typography and navigation',level:2}).count()===1,`${mode}: h2 semantics missing`);
   assert(await page.locator('.type-body').evaluate(e=>e.tagName)==='P',`${mode}: body is not paragraph`);
   const metrics=await page.locator('.typography-demo').evaluate(e=>Object.fromEntries(['type-lead','type-body','type-muted','type-small','type-caption'].map(c=>{const s=getComputedStyle(e.querySelector('.'+c));return[c,{size:parseFloat(s.fontSize),color:s.color}]})));
   assert(metrics['type-lead'].size>metrics['type-body'].size&&metrics['type-body'].size>metrics['type-caption'].size,`${mode}: text hierarchy missing`);
   assert(metrics['type-body'].color===(scheme==='dark'?'rgb(222, 226, 230)':'rgb(33, 37, 41)'),`${mode}: rich context theme inheritance failed`);
   const truncated=await page.locator('.type-truncate').evaluate(e=>({overflow:getComputedStyle(e).overflowX,space:getComputedStyle(e).whiteSpace,ellipsis:getComputedStyle(e).textOverflow,width:e.clientWidth,scroll:e.scrollWidth,text:e.textContent}));
   assert(truncated.overflow==='hidden'&&truncated.space==='nowrap'&&truncated.ellipsis==='ellipsis'&&truncated.scroll>truncated.width&&truncated.text.includes('full sentence remains accessible'),`${mode}: truncation is not real ellipsis or lost full content`);
   const request=page.getByRole('link',{name:'Open documentation',exact:true});
   assert(await request.getAttribute('target')==='_blank'&&await request.getAttribute('rel')==='noopener noreferrer',`${mode}: external context policy missing`);
   assert(await request.getAttribute('data-gly-link-request')!==null,`${mode}: typed navigation capture not declared`);
   const initialPages=context.pages().length;
   await request.click();await tick();assert(page.url()===url&&context.pages().length===initialPages,`${mode}: typed request performed default navigation`);
   assert((await page.locator('.type-status').textContent()).includes('requests=1; ordered=true; NewContext:https://example.invalid/documentation'),`${mode}: typed request or order wrong`);
   await request.focus();await page.keyboard.press('Enter');await tick();assert((await page.locator('.type-status').textContent()).includes('requests=2'),`${mode}: Enter link request missing`);
   await request.evaluate(e=>window.savedAnchor=e);await page.locator('.type-body').evaluate(e=>window.savedBody=e);
   await page.evaluate(()=>window.demo.typography_text('External <text> & update'));
   await page.evaluate(()=>window.demo.typography_href('/updated-docs'));
   await page.evaluate(()=>window.demo.typography_external(false));await tick();
   assert(await request.evaluate(e=>e===window.savedAnchor)&&await page.locator('.type-body').evaluate(e=>e===window.savedBody),`${mode}: external updates replaced identity`);
   assert(await page.locator('.type-body').textContent()==='External <text> & update',`${mode}: reactive text corrupted`);
   assert(await request.getAttribute('href')==='/updated-docs'&&await request.getAttribute('target')===null&&await request.getAttribute('rel')===null,`${mode}: context reset left target or rel`);
   assert((await page.locator('.type-status').textContent()).includes('requests=2'),`${mode}: external source updates emitted navigation`);
   await request.click();await tick();assert((await page.locator('.type-status').textContent()).includes('requests=3; ordered=true; Current:/updated-docs'),`${mode}: accepted live destination not used`);
   await page.getByRole('link',{name:'Native fragment navigation',exact:true}).click();await tick();assert(page.url().endsWith('#passive'),`${mode}: callback-free anchor blocked native fragment navigation`);
   assert(await page.locator('.type-unstyled').evaluate(e=>getComputedStyle(e).fontSize)==='17px',`${mode}: unstyled erased explicit size`);
   const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();
   assert(ax.nodes.some(n=>n.role?.value==='heading'&&n.name?.value==='Readable by design'),`${mode}: AX heading missing`);
   assert(ax.nodes.some(n=>n.role?.value==='link'&&n.name?.value==='Open documentation'),`${mode}: AX link missing`);
   await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});
   reports.push({suite:mode,status:'passed',checks:'true h1/h2/p semantics, rich theme context, actual typed ellipsis/full accessible text, controlled text/href/context identity, typed navigation no default/no popup, Enter, native passive fragment, target/rel reset, AX heading/link'});
  }
  await page.goto('http://127.0.0.1:8794/typography-hydrate-green-dark.html?manual');await page.waitForFunction(()=>window.moduleReady===true);
  await page.locator('.type-request').evaluate(e=>{window.preAnchor=e;e.focus()});
  await page.evaluate(()=>window.demo.mount_typography(true,'green','dark'));await tick();
  assert(await page.locator('.type-request').evaluate(e=>e===window.preAnchor&&document.activeElement===e),'typography hydration replaced focused anchor');
  await page.evaluate(()=>window.demo.unmount_typography());await tick();assert(await page.locator('.sapid-typography').count()===0,'typography retirement leaked mounted content');
  reports.push({suite:'typography-hydration-retirement',status:'passed'});return{reports,errors};
 }finally{await context.close();}
}
