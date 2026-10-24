async(hostPage,baseURL='http://127.0.0.1:8794')=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1200,height:900}}),reports=[],errors=[];
  const assert=(value,message)=>{if(!value)throw new Error(message)};
  try{
    const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
    const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    const open=async(path='environment.html')=>{await page.goto(`${baseURL}/${path}`);await page.waitForFunction(()=>window.ready===true);await tick()};
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const mode of ['csr','hydrate']){
      await open(`${mode==='csr'?'environment':`environment-hydrate-${hue}-${scheme}`}.html?theme=${hue}&scheme=${scheme}`);
      const provider=page.locator('.environment-provider');assert(await provider.getAttribute('dir')==='rtl'&&await provider.getAttribute('lang')==='ar','outer environment is typed RTL Arabic');
      const first=await page.locator('.environment-first').boundingBox(),second=await page.locator('.environment-second').boundingBox();assert(first.x>second.x,'RTL changes actual flex row order');
      const styles=await page.locator('.environment-first').evaluate(element=>{const style=getComputedStyle(element);return{font:style.fontSize,radius:style.borderRadius,height:element.getBoundingClientRect().height}});
      assert(styles.font==='19px'&&styles.radius==='8px'&&styles.height>=48,'controls consume custom typography, radius and size tokens');
      assert(await page.locator('.environment-heading').evaluate(element=>getComputedStyle(element).fontSize)==='35px','heading consumes its role token');
      assert(await page.getByRole('button',{name:'关闭',exact:true}).count()===1,'nested locale supplies actual localized control text');
      assert(await page.locator('.environment-nested').getAttribute('dir')==='ltr'&&await page.locator('.environment-nested').getAttribute('lang')==='zh-CN','nested explicit context overrides direction and language');
      reports.push(`environment-${hue}-${scheme}-${mode}`);
    }
    await open();
    const radios=page.getByRole('radio');await radios.nth(0).focus();await page.keyboard.press('ArrowLeft');await tick();
    assert(await radios.nth(1).isChecked(),'explicit context direction controls actual RTL radio navigation');reports.push('environment-navigation-consumes-direction');
    const motion=await page.locator('[data-gly-collapse-duration]').getAttribute('data-gly-collapse-duration');
    assert(motion==='0ms','Reduce context disables the actual requested height transition');reports.push('environment-reduced-motion-consumer');
    await page.goto(`${baseURL}/environment-hydrate-blue-light.html?manual=1`);await page.waitForFunction(()=>window.moduleReady===true);
    await page.getByRole('textbox',{name:'Environment draft',exact:true}).fill('Dirty 🙂');
    await page.getByRole('textbox',{name:'Environment draft',exact:true}).evaluate(element=>{window.dirtyEditor=element;element.setSelectionRange(2,4)});
    await page.evaluate(()=>window.demo.mount_environment(true,'blue','light'));await tick();
    assert(await page.getByRole('textbox',{name:'Environment draft',exact:true}).evaluate(element=>element===window.dirtyEditor&&element.value==='Dirty 🙂'&&element.selectionStart===2&&element.selectionEnd===4),'environment adoption preserves dirty editor identity and UTF16 caret');
    reports.push('environment-dirty-adoption');
    return{reports,errors};
  }finally{await context.close()}
}
