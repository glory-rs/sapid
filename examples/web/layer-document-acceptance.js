async(hostPage,baseURL='http://127.0.0.1:8794')=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1100,height:850},hasTouch:true}),reports=[],errors=[];
  const assert=(value,message)=>{if(!value)throw new Error(message)};
  let checkpoint='initial';
  try{
    const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
    const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    const state=(key,kind,value)=>page.evaluate(args=>demo.layer_document_state(...args),[key,kind,value]);
    const dialog=key=>page.locator(`#app-${key} .sapid-dialog`);
    const opened=key=>page.waitForFunction(key=>document.querySelector(`#app-${key} .sapid-dialog`)?.matches(':popover-open'),key);
    const closed=key=>page.waitForFunction(key=>!document.querySelector(`#app-${key} .sapid-dialog`)?.matches(':popover-open'),key);
    const first=key=>page.getByRole('textbox',{name:`First ${key}`,exact:true});
    const second=key=>page.getByRole('textbox',{name:`Second ${key}`,exact:true});
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]){
      checkpoint=`${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
      await page.goto(`${baseURL}/${hydrate?`layer-document-hydrate-${hue}-${scheme}.html`:'layer-document.html'}?theme=${hue}&scheme=${scheme}`);await page.waitForFunction(()=>window.ready===true);
      await page.evaluate(()=>{
        const html=document.documentElement.style,body=document.body.style;
        html.setProperty('overflow-x','visible');html.setProperty('overflow-y','scroll','important');
        body.setProperty('overflow-x','scroll','important');body.setProperty('overflow-y','auto');body.setProperty('padding-right','7px','important');
        window.layerSaved=[document.documentElement,document.body].map(element=>['overflow-x','overflow-y','padding-right'].map(name=>[element.style.getPropertyValue(name),element.style.getPropertyPriority(name)]));
        window.scrollTo(0,320);
      });await tick();
      const originalY=await page.evaluate(()=>scrollY);
      await page.getByRole('button',{name:'Open a',exact:true}).click();await opened('a');await tick();
      assert(await first('a').evaluate(e=>e===document.activeElement),'first App initial focus');
      assert(await page.locator('#app-b').evaluate(e=>!!e.closest('[inert]')),'second App has document inert lease');
      assert(await page.evaluate(()=>document.documentElement.style.getPropertyValue('overflow-y')==='hidden'&&document.body.style.getPropertyPriority('overflow-y')==='important'),'document scrollbar lock');
      await state('a','reject',true);
      const bOutside=await page.locator('#app-b button').first().boundingBox();await page.mouse.click(bOutside.x+bOutside.width/2,bOutside.y+bOutside.height/2);await tick();
      assert(await page.locator('.document-clicks-b').textContent()==='clicks:0'&&await dialog('a').evaluate(e=>e.matches(':popover-open')),'foreign background activation blocked during controlled refusal');
      await state('a','reject',false);await second('a').focus();
      await state('b','open',true);await opened('b');await tick();
      assert(await first('b').evaluate(e=>e===document.activeElement),'later foreign App owns focus');
      assert(await page.locator('#app-a').evaluate(e=>!!e.closest('[inert]')),'older App becomes inert');
      const cdp=await context.newCDPSession(page),ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();
      assert(ax.nodes.some(n=>!n.ignored&&n.role?.value==='dialog'&&n.name?.value==='Modal b')&&!ax.nodes.some(n=>!n.ignored&&n.role?.value==='dialog'&&n.name?.value==='Modal a'),'AX exposes current document modal');
      for(let index=0;index<8;index++){await page.keyboard.press(index%2?'Shift+Tab':'Tab');assert(await dialog('b').evaluate(e=>e.contains(document.activeElement)),'Tab stays in current document modal');}
      await page.keyboard.press('Escape');await closed('b');await opened('a');await tick();
      assert(await second('a').evaluate(e=>e===document.activeElement),'closing foreign modal restores the actual prior focus');
      await state('b','open',true);await opened('b');await second('b').focus();await state('a','open',false);await tick();
      assert(await second('b').evaluate(e=>e===document.activeElement)&&await dialog('b').evaluate(e=>e.matches(':popover-open')),'closing the lower App does not steal upper focus');
      assert(await page.evaluate(()=>document.body.style.getPropertyValue('overflow-y')==='hidden'),'lower close preserves scroll lease');
      await page.keyboard.press('Escape');await closed('b');await tick();
      assert(await page.evaluate(()=>[document.documentElement,document.body].every((element,index)=>['overflow-x','overflow-y','padding-right'].every((name,j)=>element.style.getPropertyValue(name)===window.layerSaved[index][j][0]&&element.style.getPropertyPriority(name)===window.layerSaved[index][j][1]))),'last close restores original overflow/padding and priorities');
      assert(Math.abs(await page.evaluate(()=>scrollY)-originalY)<1,'last close restores original viewport position');
      assert(await page.locator('#author-inert').getAttribute('inert')!==null,'author inert state survives');
      reports.push({suite:`layer-document-${checkpoint}`,status:'passed',checks:'cross-App modality, AX, native pointer, Tab/Escape, focus transfer, out-of-order close, scroll and style restoration'});
    }
    checkpoint='reverse-open-and-owner-cleanup';
    await state('b','open',true);await opened('b');await second('b').focus();await state('a','open',true);await opened('a');await page.keyboard.press('Escape');await closed('a');await opened('b');await tick();
    assert(await second('b').evaluate(e=>e===document.activeElement),'Escape follows open order rather than listener registration order');
    await state('a','open',true);await opened('a');await page.evaluate(()=>demo.unmount_layer_document_app('b'));await tick();
    assert(await dialog('a').evaluate(e=>e.matches(':popover-open'))&&await first('a').evaluate(e=>e===document.activeElement),'lower Owner retirement leaves current modal intact');
    await page.evaluate(()=>demo.unmount_layer_document_app('a'));await tick();
    assert(await page.locator(':popover-open').count()===0&&await page.evaluate(()=>document.body.style.getPropertyValue('overflow-y')==='auto'),'last Owner retirement releases layers and scroll lock');
    reports.push({suite:'layer-document-reverse-order-owner-cleanup',status:'passed'});
    checkpoint='nested-independent-root-retirement';
    await page.goto(`${baseURL}/layer-document.html`);await page.waitForFunction(()=>window.ready===true);
    await page.getByRole('button',{name:'Open a',exact:true}).click();await opened('a');
    await page.evaluate(()=>document.querySelector('#app-a .sapid-dialog').append(document.querySelector('#app-b')));
    await state('b','open',true);await opened('b');await tick();
    assert(await first('b').evaluate(e=>e===document.activeElement),'nested independent App retains its actual focus scope');
    await page.evaluate(()=>demo.unmount_layer_document_app('a'));await tick();
    assert(await page.locator(':popover-open').count()===0&&await page.evaluate(()=>document.body.style.getPropertyValue('overflow-y')===''),'removing a containing root releases disconnected foreign layers immediately');
    await page.evaluate(()=>demo.unmount_layer_document_app('b'));reports.push({suite:'layer-document-nested-root-retirement',status:'passed'});
    checkpoint='native-scroll-isolation';
    await page.goto(`${baseURL}/layer-document.html`);await page.waitForFunction(()=>window.ready===true);await page.evaluate(()=>scrollTo(0,320));await tick();const savedY=await page.evaluate(()=>scrollY);
    await page.getByRole('button',{name:'Open a',exact:true}).click();await opened('a');await state('a','reject',true);
    await page.mouse.move(1040,720);await page.mouse.wheel(0,700);await tick();assert(Math.abs(await page.evaluate(()=>scrollY)-savedY)<1,'native background wheel cannot scroll document');
    const bounds=await dialog('a').boundingBox();await page.mouse.move(bounds.x+30,bounds.y+80);await page.mouse.wheel(0,400);await tick();
    assert(await dialog('a').evaluate(e=>e.scrollTop)>0&&Math.abs(await page.evaluate(()=>scrollY)-savedY)<1,'wheel scrolls the modal while preserving viewport');
    await dialog('a').evaluate(e=>{e.scrollTop=0;e.focus()});await page.keyboard.press('PageDown');await tick();assert(Math.abs(await page.evaluate(()=>scrollY)-savedY)<1&&await dialog('a').evaluate(e=>e.scrollTop)>0,'PageDown targets the actual modal scroller');
    const touch=await context.newCDPSession(page);await touch.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x:1040,y:730}]});await touch.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x:1040,y:350}]});await touch.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});await touch.detach();await tick();assert(Math.abs(await page.evaluate(()=>scrollY)-savedY)<1,'native background touch cannot scroll document');
    await state('a','reject',false);await state('a','open',false);await closed('a');reports.push({suite:'layer-document-native-scroll-isolation',status:'passed'});
    checkpoint='style-ownership-and-registration-budget';
    await state('a','open',true);await opened('a');await page.evaluate(()=>document.body.style.setProperty('padding-right','29px','important'));await state('a','open',false);await closed('a');
    assert(await page.evaluate(()=>document.body.style.getPropertyValue('padding-right')==='29px'&&document.body.style.getPropertyPriority('padding-right')==='important'),'cleanup preserves an intervening author style write');
    assert(await page.evaluate(()=>demo.layer_document_budget_probe()),'65th document App registration is rejected before root mutation and prior Apps retire cleanly');reports.push({suite:'layer-document-style-ownership-and-live-budget',status:'passed'});
    return{reports,errors};
  }catch(error){errors.push(`${checkpoint}: ${error.message}`);return{reports,errors}}finally{await context.close()}
}
