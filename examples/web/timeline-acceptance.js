async(hostPage)=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1440,height:1000}});const reports=[],errors=[];const assert=(condition,message)=>{if(!condition)throw new Error(message)};
  try{
    const page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));page.on('console',message=>{if(message.type()==='error')errors.push(message.text())});
    const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    for(const hue of['blue','green'])for(const scheme of['light','dark'])for(const hydration of[false,true]){
      const mode=`timeline-${hue}-${scheme}-${hydration?'hydrate':'csr'}`;
      await page.goto(`http://127.0.0.1:8794/${hydration?`timeline-hydrate-${hue}-${scheme}.html`:`timeline.html?theme=${hue}&scheme=${scheme}`}`);await page.waitForFunction(()=>window.ready===true).catch(error=>{throw new Error(`${mode}: mount failed: ${errors.join(" | ")} (${error.message})`)});
      assert(await page.getByRole('separator').count()===2,`${mode}: decorative separator entered semantics`);
      for(const[name,axis]of[['Stage boundary','horizontal'],['Vertical boundary','vertical']]){
        const line=page.getByRole('separator',{name,exact:true});assert(await line.getAttribute('aria-orientation')===axis,`${mode}: separator axis`);assert(await line.getAttribute('tabindex')===null,`${mode}: passive line is focusable`);
        const box=await line.boundingBox();assert(box.width>0&&box.height>0,`${mode}: physical line absent`);
      }
      assert(await page.locator('.decorative-boundary').getAttribute('role')===null,`${mode}: decorative role`);
      assert(await page.locator('ol.sapid-timeline').count()===1&&await page.locator('li.sapid-timeline-item').count()===3,`${mode}: real ordered structure`);
      const states=()=>page.locator('.sapid-timeline-state').allTextContents();assert((await states()).join('|')==='Completed|Current|Upcoming',`${mode}: initial status text`);
      const input=page.getByRole('textbox',{name:'Build notes',exact:true});await input.fill('Draft notes');await tick();await input.evaluate(element=>{window.timelineInput=element;element.setSelectionRange(1,5);window.timelineItems=[...document.querySelectorAll('.sapid-timeline-item')];});
      await page.getByRole('button',{name:'Advance timeline',exact:true}).click();await tick();assert((await states()).join('|')==='Completed|Completed|Current',`${mode}: active change`);
      assert(await input.evaluate(element=>element===window.timelineInput&&element.value==='Draft notes'&&element.selectionStart===1&&element.selectionEnd===5),`${mode}: progress reset embedded draft/caret`);
      assert(await page.locator('.sapid-timeline').evaluate(element=>[...element.children].every((item,index)=>item===window.timelineItems[index])),`${mode}: progress recreated item identities`);
      await page.getByRole('button',{name:'Complete timeline',exact:true}).click();await tick();assert((await states()).every(value=>value==='Completed'),`${mode}: completion`);
      await page.getByRole('button',{name:'Reset timeline',exact:true}).click();await tick();assert((await states()).every(value=>value==='Upcoming'),`${mode}: reset`);
      const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();assert(ax.nodes.filter(node=>String(node.role?.value).toLowerCase()==='separator').length===2,`${mode}: AX separators`);assert(ax.nodes.some(node=>node.role?.value==='list'&&node.name?.value==='Release progress'),`${mode}: AX ordered timeline`);
      await page.setViewportSize({width:390,height:844});await tick();assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${mode}: mobile overflow`);await page.setViewportSize({width:1440,height:1000});
      if(hydration)await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});
      await page.evaluate(()=>window.demo.unmount_display());assert(await page.locator('#app').evaluate(element=>element.childElementCount===0),`${mode}: teardown`);
      reports.push({suite:mode,status:'passed',checks:'passive/decorative separator role/orientation/geometry, chronological list/status text, controlled progress, item/draft/caret identity, AX, mobile, teardown'});
    }
    assert(errors.length===0,JSON.stringify(errors));return{reports,errors};
  }finally{await context.close();}
}

