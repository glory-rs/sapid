async (hostPage) => {
 const context=await hostPage.context().browser().newContext({viewport:{width:1100,height:900}}),reports=[],errors=[];
 const assert=(value,message)=>{if(!value)throw new Error(message)};
 try {
  const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
  const settle=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))));
  const inset=element=>element.evaluate(e=>{const r=e.getBoundingClientRect();return{right:innerWidth-r.right,bottom:innerHeight-r.bottom,x:r.x,y:r.y}});
  for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydration of [false,true]) {
   const label=`affix-${hue}-${scheme}-${hydration?'hydrate':'csr'}`;
   await page.setViewportSize({width:1100,height:900});await page.goto(`http://127.0.0.1:8794/${hydration?`affix-hydrate-${hue}-${scheme}.html`:`affix.html?theme=${hue}&scheme=${scheme}`}`);await page.waitForFunction(()=>window.ready===true);await settle();
   const fixed=page.locator('.viewport-affix'),container=page.locator('.demo-affix-container'),sticky=page.locator('.sticky-affix'),absolute=page.locator('.absolute-affix'),draft=page.locator('.affix-draft');
   const initial=await inset(fixed);assert(Math.abs(initial.right-24)<1&&Math.abs(initial.bottom-24)<1,`${label}: actual viewport fixed offsets missing`);
   await draft.fill('Stable floating draft');await draft.evaluate(e=>{window.affixDraft=e;e.setSelectionRange(2,7,'backward')});
   await page.evaluate(()=>scrollTo(0,400));await settle();const scrolled=await inset(fixed);assert(Math.abs(scrolled.x-initial.x)<1&&Math.abs(scrolled.y-initial.y)<1,`${label}: fixed box moved with page scroll`);
   await page.evaluate(()=>scrollTo(0,0));await settle();const cb=await container.boundingBox(),ab=await absolute.boundingBox();assert(Math.abs(cb.x+cb.width-ab.x-ab.width-16)<1&&Math.abs(cb.y+cb.height-ab.y-ab.height-16)<1,`${label}: absolute affix is not anchored to the explicit relative container`);
   const before=await sticky.boundingBox();await container.evaluate(e=>e.scrollTop=180);await settle();const after=await sticky.boundingBox(),root=await container.boundingBox();assert(before.y>root.y+50&&Math.abs(after.y-root.y-12)<1,`${label}: actual sticky constraint did not hold during container scrolling`);
   assert(Math.abs((await inset(fixed)).y-initial.y)<1,`${label}: container scrolling moved viewport fixed box`);
   await page.evaluate(()=>window.demo.affix_visible(false));await settle();assert(!await fixed.isVisible(),'hidden affix remained visible');await page.evaluate(()=>window.demo.affix_visible(true));await settle();assert(await draft.evaluate(e=>e===window.affixDraft&&e.value==='Stable floating draft'),'visibility retired the live editor');
   await page.setViewportSize({width:600,height:700});await page.waitForFunction(()=>document.querySelector('.affix-status').textContent.includes('mobile=true'));await settle();const narrow=await inset(fixed);assert(Math.abs(narrow.right-8)<1&&Math.abs(narrow.bottom-8)<1,`${label}: explicit Resize-driven responsive offset failed`);
   await page.evaluate(()=>window.demo.affix_offset(31,37));await settle();const custom=await inset(fixed);assert(Math.abs(custom.right-31)<1&&Math.abs(custom.bottom-37)<1,`${label}: controlled offset was not applied`);
   const rejected=await page.evaluate(()=>{try{window.demo.affix_offset(NaN,0);return false}catch{return true}});assert(rejected,'invalid offset source was accepted');await settle();const accepted=await inset(fixed);assert(Math.abs(accepted.right-31)<1&&Math.abs(accepted.bottom-37)<1,`${label}: rejected offset changed accepted descriptor`);
   await page.evaluate(()=>window.demo.affix_offset(24,24));await page.setViewportSize({width:1100,height:900});await settle();await container.evaluate(e=>e.scrollTop=0);await page.screenshot({path:`D:/BuildCache/sapid/${label}.png`,fullPage:false});reports.push({suite:label,status:'passed',checks:'actual Fixed/right/bottom/page scroll, explicit relative container absolute positioning, real container Sticky, stable hidden editor, local Resize-responsive offset, rejected source accepted state'});
  }
  await page.goto('http://127.0.0.1:8794/affix-hydrate-green-dark.html?manual');await page.waitForFunction(()=>window.moduleReady===true);const draft=page.locator('.affix-draft');await draft.fill('Before affix hydration');await draft.evaluate(e=>{window.beforeAffix=e;e.focus();e.setSelectionRange(2,9,'backward')});
  await page.evaluate(()=>window.demo.mount_affix(true,'green','dark'));await settle();assert(await draft.evaluate(e=>e===window.beforeAffix&&e.value==='Before affix hydration'&&e.selectionStart===2&&e.selectionEnd===9),'affix dirty hydration lost editor identity/value/selection');await page.evaluate(()=>window.demo.unmount_affix());await settle();assert(await page.locator('.affix-demo').count()===0,'affix Owner retirement leaked content/context');reports.push({suite:'affix-dirty-hydration-retirement',status:'passed'});
  return{reports,errors};
 }finally{await context.close()}
}
