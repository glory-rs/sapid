async (hostPage) => {
 const context=await hostPage.context().browser().newContext({viewport:{width:1250,height:1000}});const reports=[],errors=[];
 const assert=(value,message)=>{if(!value)throw new Error(message)};
 try{
  const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
  const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
  for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydration of [false,true]){
   const mode=`navigation-${hue}-${scheme}-${hydration?'hydrate':'csr'}`;
   const url=`http://127.0.0.1:8794/${hydration?`navigation-hydrate-${hue}-${scheme}.html`:`navigation.html?theme=${hue}&scheme=${scheme}`}`;
   await page.goto(url);await page.waitForFunction(()=>window.ready===true);
   const crumbs=page.getByRole('navigation',{name:'Breadcrumbs',exact:true});const pages=page.locator('.main-pages');const unknown=page.locator('.unknown-pages');
   assert(await crumbs.getByRole('list').count()===1&&await crumbs.getByRole('listitem').count()===4,`${mode}: bounded breadcrumb list missing`);
   assert(await crumbs.locator('[aria-current=page]').textContent()==='Current article',`${mode}: current breadcrumb missing`);
   assert(await crumbs.getByRole('link',{name:'Library',exact:true}).count()===0,`${mode}: middle path not folded`);
   await crumbs.getByRole('button',{name:'Expand breadcrumb path'}).focus();await page.keyboard.press('Space');await tick();
   assert(await crumbs.getByRole('link',{name:'Library',exact:true}).count()===1,`${mode}: Space did not unfold breadcrumb links`);
   await crumbs.getByRole('link',{name:'Library',exact:true}).focus();
   await crumbs.getByRole('button',{name:'Collapse breadcrumb path'}).evaluate(e=>e.click());await tick();
   assert(await crumbs.getByRole('button',{name:'Expand breadcrumb path'}).evaluate(e=>document.activeElement===e),`${mode}: collapsing path did not restore focus`);
   assert(await pages.getByRole('button').count()===9&&await pages.locator('[aria-current=page]').textContent()==='Page 5',`${mode}: huge total scanned or window incorrect`);
   await pages.locator('[aria-current=page]').evaluate(e=>window.initialPageButton=e);
   await pages.getByRole('button',{name:'Next page',exact:true}).click();await tick();
   assert(await pages.locator('[aria-current=page]').textContent()==='Page 6',`${mode}: mouse next request missing`);
   assert((await page.locator('.navigation-status').textContent()).includes('requests=1; ordered=true'),`${mode}: typed request ordering failed`);
   await page.evaluate(()=>window.demo.navigation_state(false,true));
   await pages.getByRole('button',{name:'Next page',exact:true}).focus();await page.keyboard.press('Enter');await tick();
   assert(await pages.locator('[aria-current=page]').textContent()==='Page 6',`${mode}: rejected controlled proposal changed UI`);
   await page.evaluate(()=>window.demo.navigation_state(true,false));await tick();assert(await pages.getByRole('button',{name:'Next page',exact:true}).isDisabled(),`${mode}: disabled root did not disable native button`);
   await page.evaluate(()=>window.demo.navigation_state(true,true));
   await page.evaluate(()=>window.demo.navigation_page(999,1000000000000));await tick();
   assert(await pages.getByRole('button').count()===9&&await pages.locator('[aria-current=page]').textContent()==='Page 999',`${mode}: source update changed bounds`);
   assert(await unknown.getByRole('button',{name:'Last page',exact:true}).isDisabled()&&await unknown.getByRole('button').count()===5,`${mode}: unknown total inferred pages`);
   const docs=page.locator('.docs-nav');const link=docs.getByRole('link',{name:'Documentation',exact:true});
   assert(await link.getAttribute('aria-current')==='page'&&await link.getAttribute('aria-selected')===null,`${mode}: current confused with selected`);
   await link.click();await tick();assert(page.url()===url,`${mode}: nav link callback performed default routing`);
   await docs.getByRole('button',{name:'Expand Documentation'}).focus();await page.keyboard.press('Enter');await tick();
   const child=docs.getByRole('textbox',{name:'Nested note'});assert(await child.count()===1,`${mode}: disclosure did not mount children`);await child.fill('Local draft');await child.focus();
   await page.evaluate(()=>window.demo.navigation_open(false));await tick();
   assert(await child.count()===0&&await docs.getByRole('button',{name:'Expand Documentation'}).evaluate(e=>document.activeElement===e),`${mode}: collapsed child Owner/focus not retired`);
   await docs.getByRole('button',{name:'Expand Documentation'}).click();await tick();assert(await child.inputValue()==='Retired with its owner',`${mode}: retired child state survived remount`);
   assert((await page.locator('.navigation-status').textContent()).includes('created=2; retired=1'),`${mode}: child lifetime evidence missing`);
   const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();
   assert(ax.nodes.filter(n=>n.role?.value==='navigation').length===3,`${mode}: navigation landmarks missing`);
   await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});reports.push({suite:mode,status:'passed',checks:'bounded trillion-page window, unknown total, current metadata, mouse/Enter/Space, controlled rejection, native disabled, folded focus restoration, actual link no implicit route, disclosure Owner reset, AX navigation'});
  }
  await page.goto('http://127.0.0.1:8794/navigation-hydrate-blue-light.html?manual');await page.waitForFunction(()=>window.moduleReady===true);
  await page.locator('.main-pages').getByRole('button',{name:'Next page',exact:true}).evaluate(e=>{window.preNext=e;e.focus()});
  await page.evaluate(()=>window.demo.mount_navigation(true,'blue','light'));await tick();
  assert(await page.locator('.main-pages').getByRole('button',{name:'Next page',exact:true}).evaluate(e=>e===window.preNext&&document.activeElement===e),'navigation hydration replaced focused button');
  await page.evaluate(()=>window.demo.unmount_navigation());await tick();assert(await page.locator('.navigation-demo').count()===0,'navigation Owner retirement leaked nodes');
  reports.push({suite:'navigation-hydration-retirement',status:'passed'});return{reports,errors};
 }finally{await context.close()}
}
