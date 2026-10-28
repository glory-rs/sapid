async(hostPage,baseURL='http://127.0.0.1:8794')=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1440,height:1000}}),reports=[],errors=[];
  const assert=(value,message)=>{if(!value)throw Error(message)};
  try{
    const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const render of ['csr','hydrate']){
      await page.goto(`${baseURL}/gallery.html?theme=${hue}&scheme=${scheme}#P4-04`);await page.waitForFunction(()=>window.galleryReady===true);
      assert(await page.locator('tbody tr').count()===149,'gallery covers all canonical work IDs exactly once');
      await page.locator('#gallery-render').selectOption(render);
      await page.waitForFunction(()=>{const frame=document.querySelector('#gallery-preview');return frame.getAttribute('aria-busy')==='false'&&frame.contentWindow?.ready===true&&frame.contentWindow.location.href===frame.src});
      const preview=page.frameLocator('#gallery-preview'),select=preview.getByRole('combobox',{name:'Subscription plan',exact:true});await select.waitFor();
      await select.focus();await select.press('ArrowDown');await select.press('Enter');await select.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));assert(await select.inputValue()==='pro-value','gallery embeds an actual keyboard-operated fixture');
      assert(await page.locator('#gallery-preview').getAttribute('src').then(src=>src.includes(`theme=${hue}`)&&src.includes(`scheme=${scheme}`)&&(!render.includes('hydrate')||src.includes('hydrate'))),'gallery preserves palette/render mode in preview URL');
      assert(await page.locator('html').getAttribute('data-hue')===hue&&await page.locator('html').getAttribute('data-mode')===scheme,'gallery chrome has the same explicit palette');
      reports.push(`gallery-${hue}-${scheme}-${render}`);
    }
    await page.locator('#gallery-search').fill('P0-11');assert(await page.locator('tbody tr:visible').count()===1,'search filters actual work IDs');await page.locator('#gallery-search').fill('');
    await page.locator('#gallery-phase').selectOption('P4');assert(await page.locator('tbody tr:visible').count()===10,'phase filter retains all ten choice work items');reports.push('gallery-search-and-phase');
    await page.locator('button[data-select="P4-01"]').click();assert(await page.locator('#gallery-capabilities').textContent().then(text=>text.includes('native: model-only')&&text.includes('shared native')),'gallery distinguishes a shared native driver from current dedicated consumer evidence');reports.push('gallery-native-evidence-scope');
    await page.locator('#gallery-phase').selectOption('');
    await page.locator('button[data-select="P5-03"]').click();assert(await page.locator('#gallery-capabilities').textContent().then(text=>text.includes('native: rejected')&&text.includes('Tooltip Layer')),'gallery keeps the current unsupported Tooltip Layer boundary visible');reports.push('gallery-explicit-host-boundary');
    const withoutPreview=await page.evaluate(()=>{const rows=JSON.parse(document.querySelector('#gallery-data').textContent);return(rows.find(row=>row.capabilities.web.state==='pending'&&!row.pages.length)||rows.find(row=>!row.pages.length)).id});
    await page.locator(`button[data-select="${withoutPreview}"]`).click();assert(await page.locator('#gallery-preview').isHidden()&&await page.locator('#gallery-preview-note').textContent().then(text=>text.includes('No interactive fixture')),'work without a fixture does not invent a preview or acceptance claim');reports.push('gallery-no-synthetic-preview');
    await page.setViewportSize({width:390,height:844});assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),'mobile gallery has no document horizontal overflow');reports.push('gallery-mobile-layout');
    return{reports,errors};
  }finally{await context.close()}
}
