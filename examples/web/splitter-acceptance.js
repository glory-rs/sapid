async (hostPage) => {
 const browser=hostPage.context().browser(),context=await browser.newContext({viewport:{width:1100,height:900}}),reports=[],errors=[];
 const assert=(value,message)=>{if(!value)throw new Error(message)};
 const install=page=>{page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())})};
 const settle=page=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))));
 const number=(element,attribute)=>element.getAttribute(attribute).then(Number);
 try {
  const page=await context.newPage();install(page);
  for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydration of [false,true]) {
   const label=`splitter-${hue}-${scheme}-${hydration?'hydrate':'csr'}`;
   await page.goto(`http://127.0.0.1:8794/${hydration?`splitter-hydrate-${hue}-${scheme}.html`:`splitter.html?theme=${hue}&scheme=${scheme}`}`);await page.waitForFunction(()=>window.ready===true);
   const handle=page.getByRole('separator',{name:'Workspace divider'}),vertical=page.getByRole('separator',{name:'Vertical divider'}),draft=page.getByRole('textbox',{name:'First draft'});
   assert(await handle.getAttribute('aria-orientation')==='vertical'&&await vertical.getAttribute('aria-orientation')==='horizontal',`${label}: separator line orientation missing`);
   assert(await number(handle,'aria-valuenow')===50&&await number(handle,'aria-valuemin')===0&&await number(handle,'aria-valuemax')===100,`${label}: finite range semantics missing`);
   await draft.fill('Stable splitter draft');await draft.evaluate(e=>{window.splitDraft=e;e.setSelectionRange(2,8,'backward')});
   await handle.focus();await page.keyboard.press('ArrowRight');await settle(page);assert(await number(handle,'aria-valuenow')===51,`${label}: perpendicular arrow did not adjust`);
   await page.evaluate(()=>window.demo.splitter_policy(false,true));await page.keyboard.press('ArrowRight');await settle(page);assert(await number(handle,'aria-valuenow')===51&&(await page.locator('.splitter-status').textContent()).includes('request=52.00'),`${label}: controlled rejection mutated source or proposal`);
   await page.evaluate(()=>window.demo.splitter_policy(true,true));await page.keyboard.press('Home');await settle(page);assert(await number(handle,'aria-valuenow')===0,`${label}: Home missed minimum`);
   await page.keyboard.press('End');await settle(page);assert(await number(handle,'aria-valuenow')===100,`${label}: End missed maximum`);
   await page.evaluate(()=>window.demo.splitter_value(50));await settle(page);
   const before=await handle.boundingBox();await page.mouse.move(before.x+before.width/2,before.y+40);await page.mouse.down();await page.mouse.move(before.x+before.width/2+120,before.y+40,{steps:5});await page.mouse.up();await settle(page);
   assert(Math.abs(await number(handle,'aria-valuenow')-70)<0.1,`${label}: captured release outside original handle lost measured drag`);
   const dimensions=await page.locator('.horizontal-splitter .sapid-splitter-frame').evaluate(e=>[e.getBoundingClientRect().width,...[...e.children].map(c=>c.getBoundingClientRect().width)]);
   assert(Math.abs(dimensions[0]-dimensions.slice(1).reduce((a,b)=>a+b,0))<1,`${label}: pane layout did not conserve available extent`);
   await vertical.focus();await page.keyboard.press('ArrowDown');await settle(page);assert(await number(vertical,'aria-valuenow')===51,`${label}: vertical local adjustment failed`);
   await page.evaluate(()=>window.demo.splitter_policy(true,false));await settle(page);assert(await handle.getAttribute('tabindex')==='-1'&&await handle.getAttribute('aria-disabled')==='true',`${label}: disabled handle remained tabbable`);
   await page.getByRole('textbox',{name:'Second draft'}).focus();await page.keyboard.press('Tab');await settle(page);assert(await vertical.evaluate(e=>e===document.activeElement),`${label}: Tab did not skip disabled handle`);
   await page.evaluate(()=>window.demo.splitter_policy(true,true));await draft.focus();await page.evaluate(()=>window.demo.splitter_value(0));await settle(page);assert(await handle.evaluate(e=>e===document.activeElement),`${label}: collapsing focused pane lost focus`);
   await page.evaluate(()=>window.demo.splitter_value(50));await settle(page);assert(await draft.evaluate(e=>e===window.splitDraft&&e.value==='Stable splitter draft'),`${label}: resize replaced pane editor`);
   assert((await page.locator('.splitter-status').textContent()).includes('ordered=true'),`${label}: generic event preceded typed callback`);
   await page.screenshot({path:`D:/BuildCache/sapid/${label}.png`,fullPage:true});reports.push({suite:label,status:'passed',checks:'range/line semantics, keyboard boundaries, controlled rejection, captured mouse release, layout conservation, vertical local state, disabled Tab skip, collapsed focus, stable editor and typed order'});
  }
  await page.goto('http://127.0.0.1:8794/splitter-hydrate-green-dark.html?manual');await page.waitForFunction(()=>window.moduleReady===true);
  const draft=page.locator('.first-draft');await draft.fill('Before splitter hydration');await draft.evaluate(e=>{window.beforeSplitter=e;e.focus();e.setSelectionRange(2,9,'backward')});
  await page.evaluate(()=>window.demo.mount_splitter(true,'green','dark'));await settle(page);assert(await draft.evaluate(e=>e===window.beforeSplitter&&e.value==='Before splitter hydration'&&e.selectionStart===2&&e.selectionEnd===9),'splitter dirty hydration replaced editor/draft/selection');
  await page.evaluate(()=>window.demo.unmount_splitter());await settle(page);assert(await page.locator('.splitter-demo').count()===0,'splitter Owner retirement leaked panes');reports.push({suite:'splitter-dirty-hydration-retirement',status:'passed'});
  const touchContext=await browser.newContext({viewport:{width:1100,height:900},hasTouch:true,isMobile:true});
  try {
   const touchPage=await touchContext.newPage();install(touchPage);await touchPage.goto('http://127.0.0.1:8794/splitter.html');await touchPage.waitForFunction(()=>window.ready===true);
   const handle=touchPage.getByRole('separator',{name:'Workspace divider'}),draft=touchPage.locator('.first-draft'),client=await touchContext.newCDPSession(touchPage);
   assert(await handle.evaluate(e=>getComputedStyle(e).touchAction)==='none','splitter touch capture default missing');
   await draft.evaluate(e=>e.setSelectionRange(2,7));const scroll=await touchPage.evaluate(()=>scrollY),box=await handle.boundingBox(),x=box.x+box.width/2,y=box.y+40;
   const send=(type,points)=>client.send('Input.dispatchTouchEvent',{type,touchPoints:points});const point=(px)=>[{x:px,y,id:7}];
   await send('touchStart',point(x));await send('touchMove',point(x+60));await send('touchEnd',[]);await settle(touchPage);
   assert(Math.abs(await number(handle,'aria-valuenow')-60)<0.1,'touch drag/end did not resize using original captured handle');
   assert(await touchPage.evaluate(()=>scrollY)===scroll&&await draft.evaluate(e=>e.selectionStart===2&&e.selectionEnd===7),'touch drag scrolled page or changed editor selection');
   const status=touchPage.locator('.splitter-status'),commits=Number((await status.textContent()).match(/commits=(\d+)/)[1]),next=await handle.boundingBox();
   await send('touchStart',[{x:next.x+next.width/2,y,id:8}]);await send('touchMove',[{x:next.x+next.width/2+60,y,id:8}]);await send('touchCancel',[]);await settle(touchPage);
   assert((await status.textContent()).includes('phase=Cancel')&&Number((await status.textContent()).match(/commits=(\d+)/)[1])===commits,'touch cancellation emitted a commit or failed cleanup');
   await touchPage.evaluate(()=>window.demo.unmount_splitter());await settle(touchPage);reports.push({suite:'splitter-touch-capture-cancel',status:'passed',checks:'Chromium CDP touch Start/Move/End/Cancel, intrinsic touch-action, captured original handle, page scroll/editor selection unchanged, no cancel commit'});
  } finally {await touchContext.close()}
  return {reports,errors};
 } finally {await context.close()}
}
