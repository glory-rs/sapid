async(hostPage,baseURL='http://127.0.0.1:8794')=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1300,height:1000}}),reports=[],errors=[];
  const assert=(value,message)=>{if(!value)throw new Error(message)};
  try{
    const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
    const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const mode of ['csr','hydrate']){
      await page.goto(`${baseURL}/${mode==='csr'?'responsive':`responsive-hydrate-${hue}-${scheme}`}.html?theme=${hue}&scheme=${scheme}`);await page.waitForFunction(()=>window.ready===true);await tick();
      assert(await page.locator('.responsive-tier').textContent()==='Base','explicit initial/actual 500px container resolves Base');
      const narrow=await page.locator('.responsive-space').boundingBox();assert(Math.abs(narrow.height-10)<0.2,'Space consumes narrow height');
      await page.locator('.responsive-fixture').evaluate(root=>{window.responsiveRoot=root;window.responsiveChild=root.querySelector('.responsive-center-child')});
      await page.getByRole('button',{name:'Wide container',exact:true}).click();await page.waitForFunction(()=>document.querySelector('.responsive-tier').textContent==='Large');await tick();
      const center=await page.locator('.responsive-center').boundingBox(),child=await page.locator('.responsive-center-child').boundingBox();assert(Math.abs(child.y-center.y-35)<0.2,'Center consumes breakpoint alignment');
      assert(Math.abs((await page.locator('.responsive-space').boundingBox()).height-40)<0.2,'Space consumes wide height');
      assert(await page.locator('.responsive-container').evaluate(element=>getComputedStyle(element).paddingLeft)==='16px','Container consumes breakpoint padding');
      assert(await page.locator('.responsive-grid').evaluate(element=>getComputedStyle(element).gap)==='16px','Grid consumes breakpoint gap');
      assert(await page.locator('.responsive-simple').evaluate(element=>getComputedStyle(element).gap)==='12px','SimpleGrid consumes breakpoint gap');
      assert(await page.locator('.responsive-fixture').evaluate(root=>root===window.responsiveRoot&&root.querySelector('.responsive-center-child')===window.responsiveChild),'responsive changes preserve actual child identities');
      await page.getByRole('button',{name:'Narrow container',exact:true}).click();await page.waitForFunction(()=>document.querySelector('.responsive-tier').textContent==='Base');await tick();
      assert(Math.abs((await page.locator('.responsive-space').boundingBox()).height-10)<0.2,'narrowing restores the accepted layout');
      reports.push(`responsive-${hue}-${scheme}-${mode}`);
    }
    await page.setViewportSize({width:800,height:1000});await tick();assert(await page.locator('.responsive-tier').textContent()==='Base','breakpoints measure declared container instead of guessing browser viewport');reports.push('responsive-container-not-viewport');
    await page.evaluate(()=>window.demo.unmount_display());await tick();assert(await page.locator('#app').evaluate(element=>element.childNodes.length===0),'Owner disposal removes responsive host subscription');reports.push('responsive-owner-retirement');
    return{reports,errors};
  }finally{await context.close()}
}
