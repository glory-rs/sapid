async (hostPage,baseURL='http://127.0.0.1:8794')=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1100,height:1000}}),reports=[],errors=[];
  const assert=(ok,message)=>{if(!ok)throw new Error(message)};
  try{
    const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
    const tick=()=>page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
    const state=(kind,value)=>page.evaluate(([k,v])=>demo.json_input_state(k,v),[kind,value]);
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]){
      const mode=`json-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
      await page.goto(`${baseURL}/${hydrate?`json-input-hydrate-${hue}-${scheme}.html`:`json-input.html?theme=${hue}&scheme=${scheme}`}`);await page.waitForFunction(()=>window.ready===true);
      const editor=page.getByRole('textbox',{name:'Payload',exact:true}),format=page.getByRole('button',{name:'Format JSON',exact:true});
      assert(await editor.evaluate(e=>e.tagName==='TEXTAREA'&&e.required&&e.name==='payload'),`${mode}: true named multiline successful control`);
      const original=await editor.inputValue();await editor.evaluate(e=>{window.savedJson=e;e.focus();e.setSelectionRange(2,7,'backward')});
      await state('error','Server validation');await tick();
      assert(await editor.evaluate(e=>e===window.savedJson&&e.selectionStart===2&&e.selectionEnd===7&&document.getElementById(e.getAttribute('aria-errormessage'))?.textContent==='Server validation'),`${mode}: Field error preserves value and UTF16 selection`);
      await state('error','');await format.focus();await page.keyboard.press('Enter');await tick();
      const pretty=await editor.inputValue();assert(pretty.includes('\n  "z": 1e999,')&&pretty.includes('9007199254740993123456789')&&pretty.includes('"z": "😀"'),`${mode}: lossless formatting of duplicate keys and number lexemes`);
      assert(await editor.evaluate(e=>e===window.savedJson)&&await page.locator('.json-status').innerText().then(v=>v.includes('formats=1')),`${mode}: format callback and editor identity`);
      await state('value','{');await tick();assert(await editor.getAttribute('aria-invalid')==='true',`${mode}: malformed draft remains editable`);
      assert(await editor.evaluate(e=>!!document.getElementById(e.getAttribute('aria-errormessage'))?.textContent),`${mode}: parse error relation`);
      assert(await format.isDisabled(),`${mode}: invalid draft cannot format`);
      await editor.fill('{"a":1}');await tick();await state('reject','true');await format.click();await tick();
      assert(await editor.inputValue()==='{"a":1}',`${mode}: controlled format refusal`);
      await editor.fill('[]');await tick();assert(await editor.inputValue()==='{"a":1}',`${mode}: controlled edit refusal`);
      await state('reject','false');await editor.fill('["😀"]');await tick();
      await editor.evaluate(e=>{e.focus();e.dispatchEvent(new CompositionEvent('compositionstart',{bubbles:true}));e.value='["合成"]';e.dispatchEvent(new InputEvent('input',{bubbles:true,inputType:'insertCompositionText',data:'合成',isComposing:true}))});await tick();
      assert(await format.isDisabled()&&await editor.inputValue()==='["合成"]',`${mode}: preedit retained and formatter blocked`);
      await editor.evaluate(e=>e.dispatchEvent(new CompositionEvent('compositionend',{bubbles:true,data:'合成'})));await tick();assert(await format.isEnabled(),`${mode}: composition terminal restores formatter`);
      const limited=page.getByRole('textbox',{name:'Limited JSON',exact:true});await page.getByRole('button',{name:'Format limited',exact:true}).click();await tick();
      assert(await limited.inputValue()==='[1,2,3]'&&await limited.getAttribute('aria-invalid')!=='true'&&await page.locator('.limited-json .sapid-json-format-error').innerText().then(v=>v.includes('byte budget')),`${mode}: output limit keeps valid original draft`);
      await limited.fill('[]');await tick();assert(await page.locator('.limited-json .sapid-json-format-error').innerText()==='',`${mode}: next same-kind accepted draft clears format error`);
      await state('readonly','true');await tick();assert(await editor.evaluate(e=>e.readOnly)&&await format.isDisabled(),`${mode}: Field readonly`);
      await state('readonly','false');await state('enabled','false');await tick();assert(await editor.isDisabled()&&await format.isDisabled(),`${mode}: Field disabled`);await state('enabled','true');
      const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();assert(ax.nodes.some(n=>n.role?.value==='textbox'&&n.name?.value==='Payload'&&n.properties?.some(p=>p.name==='multiline'&&p.value.value===true)),`${mode}: actual multiline accessibility tree`);
      await page.setViewportSize({width:390,height:844});await tick();assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${mode}: mobile width`);await page.setViewportSize({width:1100,height:1000});
      if(hue==='green'&&scheme==='dark'&&hydrate)await page.screenshot({path:'D:/BuildCache/sapid/json-input-green-dark-hydrate.png',fullPage:true});
      await page.evaluate(()=>demo.unmount_json_input());assert(await page.locator('#app').evaluate(e=>e.childElementCount===0),`${mode}: Owner cleanup`);
      reports.push({suite:mode,status:'passed',checks:'true multiline editor/AX, Field metadata and caret, exact JSON tokens, controlled proposal/refusal, parse/output limits, IME protocol, readonly/disabled, mobile, retirement'});
    }
    await page.goto(`${baseURL}/json-input-hydrate-blue-light.html?manual=1`);await page.waitForFunction(()=>window.moduleReady===true);
    await page.getByRole('textbox',{name:'Payload',exact:true}).evaluate(e=>{window.preJson=e;e.value='{"dirty":"😀"}';e.focus();e.setSelectionRange(2,5);demo.mount_json_input(true,'blue','light')});await tick();
    assert(await page.getByRole('textbox',{name:'Payload',exact:true}).evaluate(e=>e===window.preJson&&e===document.activeElement&&e.value==='{"dirty":"😀"}'&&e.selectionStart===2&&e.selectionEnd===5),'JSON dirty adoption preserves editor/focus/UTF16 selection');reports.push({suite:'json-dirty-hydration',status:'passed'});
    assert(errors.length===0,JSON.stringify(errors));return{reports,errors};
  }finally{await context.close()}
}
