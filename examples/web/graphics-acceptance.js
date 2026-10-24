async(hostPage)=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1440,height:1000}});const reports=[],errors=[];const assert=(value,message)=>{if(!value)throw new Error(message)};
  try{const page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));page.on('console',message=>{if(message.type()==='error')errors.push(message.text())});
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]){
      const mode=`graphics-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
      await page.goto(`http://127.0.0.1:8794/${hydrate?`graphics-hydrate-${hue}-${scheme}.html`:`graphics.html?theme=${hue}&scheme=${scheme}`}`);await page.waitForFunction(()=>window.ready===true);
      const marker=page.locator('.graphics-indicator>.sapid-indicator-marker'),wrapper=page.locator('.graphics-indicator');
      assert(await marker.getAttribute('role')==='group'&&await marker.getAttribute('aria-label')==='Unread messages',`${mode}: marker name`);
      assert(await marker.textContent()==='3',`${mode}: marker label`);assert(await page.getByRole('button',{name:'Inbox',exact:true}).count()===1,`${mode}: independent control`);
      const [outer,inner]=await Promise.all([wrapper.boundingBox(),marker.boundingBox()]);assert(Math.abs(inner.x+10-outer.x-outer.width)<0.2&&Math.abs(inner.y+10-outer.y)<0.2&&Math.abs(inner.width-20)<0.2,`${mode}: actual marker anchor`);
      const dynamic=page.locator('.graphics-dynamic svg'),path=dynamic.locator('path');
      const geometry=await dynamic.evaluate(e=>({namespace:e.namespaceURI,pathNamespace:e.firstElementChild.namespaceURI,viewBox:e.getAttribute('viewBox'),length:e.firstElementChild.getTotalLength(),width:e.getBoundingClientRect().width,height:e.getBoundingClientRect().height}));
      assert(geometry.namespace==='http://www.w3.org/2000/svg'&&geometry.pathNamespace===geometry.namespace&&geometry.viewBox==='0 0 24 24'&&geometry.length>20&&Math.abs(geometry.width-21.6)<0.2&&Math.abs(geometry.height-21.6)<0.2,`${mode}: actual vector geometry`);
      assert(await dynamic.getAttribute('role')==='img'&&await dynamic.getAttribute('aria-label')==='Completed',`${mode}: image alternative`);
      assert(await page.locator('.graphics-decorative svg').getAttribute('aria-hidden')==='true',`${mode}: explicit decoration`);
      assert((await page.locator('.graphics-unstyled path').getAttribute('d')).length>0,`${mode}: unstyled lost content`);
      await path.evaluate(e=>{window.graphicPath=e;window.originalGraphicPath=e.getAttribute('d')});await marker.evaluate(e=>window.graphicMarker=e);
      await page.getByRole('button',{name:'Update markers',exact:true}).click();await page.waitForFunction(()=>document.querySelector('.graphics-dynamic svg')?.getAttribute('aria-label')==='Closed');
      assert(await path.evaluate(e=>e===window.graphicPath&&e.getAttribute('d')!==window.originalGraphicPath),`${mode}: drawing patch identity`);assert(await marker.evaluate(e=>e===window.graphicMarker&&e.textContent==='4'),`${mode}: label patch identity`);
      await page.getByRole('button',{name:'Toggle marker',exact:true}).click();await page.waitForFunction(()=>getComputedStyle(document.querySelector('.graphics-indicator>.sapid-indicator-marker')).display==='none');
      assert(await page.getByRole('button',{name:'Inbox',exact:true}).isVisible(),`${mode}: marker hide removed child control`);
      await page.getByRole('button',{name:'Toggle marker',exact:true}).click();await page.waitForFunction(()=>getComputedStyle(document.querySelector('.graphics-indicator>.sapid-indicator-marker')).display!=='none');
      const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();const images=ax.nodes.filter(node=>node.role?.value==='image'&&!node.ignored);assert(images.length===2&&images.some(node=>node.name?.value==='Closed')&&images.some(node=>node.name?.value==='Information'),`${mode}: image AX alternatives/decoration`);assert(!ax.nodes.some(node=>node.role?.value==='status'),`${mode}: passive indicator became live status`);
      await page.setViewportSize({width:390,height:844});assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${mode}: narrow overflow`);await page.setViewportSize({width:1440,height:1000});
      if(hydrate)await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});
      await page.evaluate(()=>window.demo.unmount_display());assert(await page.locator('#app').evaluate(e=>e.childElementCount===0),`${mode}: teardown`);reports.push({suite:mode,status:'passed',checks:'real SVG namespace/path geometry, named/decorative AX, anchored markers, reactive identity, independent content, themes/mobile, cleanup'});
    }
    await page.goto('http://127.0.0.1:8794/graphics-stream.html');await page.waitForFunction(()=>document.querySelector('#graphics-stream-root')?.dataset.glyStreamState==='ready');
    assert(await page.locator('svg').count()===2,'stream: SVG Create count');
    assert(await page.locator('.stream-current-graphic path').getAttribute('d')==='M2 2L22 22','stream: existing path patch');
    assert(await page.locator('.stream-created-graphic').evaluate(e=>e.namespaceURI==='http://www.w3.org/2000/svg'&&e.firstElementChild.namespaceURI===e.namespaceURI&&e.getAttribute('viewBox')==='0 0 24 24'&&e.getAttribute('preserveAspectRatio')==='xMidYMid meet'&&e.firstElementChild.getTotalLength()>60),'stream: real finite SVG namespace/attributes/path');
    assert(await page.getByRole('img',{name:'Created graphic',exact:true}).count()===1,'stream: actual image role');
    assert(await page.evaluate(()=>GloryStream.active()===0&&document.querySelector('#graphics-stream-root').firstChild.nodeValue==='gly:v:6'),'stream: End handoff and released controller');reports.push({suite:'graphics-classic-stream',status:'passed',checks:'actual SVG shell, Create namespace, drawing field patch, image alternative, finite End rekey and controller release'});
    assert(errors.length===0,JSON.stringify(errors));return{reports,errors};
  }finally{await context.close()}
}

