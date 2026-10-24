async(hostPage)=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1440,height:1000}}),reports=[],errors=[];
  const assert=(value,message)=>{if(!value)throw new Error(message)};
  try{const page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]){
      const mode=`navigation-progress-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
      await page.goto(`http://127.0.0.1:8794/${hydrate?`navigation-progress-hydrate-${hue}-${scheme}.html`:`navigation-progress.html?theme=${hue}&scheme=${scheme}`}`);await page.waitForFunction(()=>window.ready===true);
      const bar=page.getByRole('progressbar',{name:'Route loading',exact:true}),root=page.locator('.sapid-navigation-progress'),editor=page.getByRole('textbox',{name:'Navigation progress editor',exact:true});
      assert(await bar.count()===0,`${mode}: idle exposed progress`);
      await editor.fill('Retained choice');await editor.evaluate(e=>window.navEditor=e);
      await root.evaluate(e=>window.navRoot=e);
      await page.getByRole('button',{name:'Start navigation',exact:true}).click();await bar.waitFor();
      assert(await bar.getAttribute('aria-valuenow')===null&&await root.getAttribute('aria-busy')==='true',`${mode}: initial indeterminate`);
      await bar.evaluate(e=>window.navBar=e);
      await page.getByRole('button',{name:'Report 60 percent',exact:true}).click();await page.waitForFunction(()=>document.querySelector('.sapid-progress')?.getAttribute('aria-valuenow')==='60');
      const fill=await bar.locator('.sapid-progress-fill').boundingBox(),box=await bar.boundingBox();assert(Math.abs(fill.width/box.width-.6)<.002&&Math.abs(box.height-8)<.1,`${mode}: actual geometry`);
      await page.getByRole('button',{name:'Replace navigation',exact:true}).click();await page.waitForFunction(()=>document.querySelector('.sapid-progress')?.getAttribute('aria-valuenow')===null);
      await page.getByRole('button',{name:'Publish stale progress',exact:true}).click();await page.getByText('Revoked',{exact:true}).waitFor();assert(await bar.getAttribute('aria-valuenow')===null,`${mode}: stale publication`);
      await page.getByRole('button',{name:'Finish navigation',exact:true}).click();await bar.waitFor({state:'hidden'});assert(await root.getAttribute('aria-busy')==='false',`${mode}: completed busy state`);
      await page.getByRole('button',{name:'Start navigation',exact:true}).click();await bar.waitFor();assert(await bar.evaluate(e=>e===window.navBar)&&await root.evaluate(e=>e===window.navRoot),`${mode}: identity`);
      await page.getByRole('button',{name:'Cancel navigation',exact:true}).click();await bar.waitFor({state:'hidden'});assert(await editor.inputValue()==='Retained choice'&&await editor.evaluate(e=>e===window.navEditor),`${mode}: retained editor`);
      await page.setViewportSize({width:390,height:844});assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${mode}: narrow overflow`);await page.setViewportSize({width:1440,height:1000});
      await page.evaluate(()=>window.demo.unmount_display());assert(await page.locator('#app').evaluate(e=>e.childElementCount===0),`${mode}: owner cleanup`);reports.push({suite:mode,status:'passed',checks:'idle/indeterminate/numeric/completed, actual 60% width, replacement revokes stale result, cancel, stable bar/editor, mobile, Owner cleanup'});
    }assert(errors.length===0,JSON.stringify(errors));return{reports,errors};
  }finally{await context.close()}
}
