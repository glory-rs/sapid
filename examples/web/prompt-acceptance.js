async(hostPage,baseURL='http://127.0.0.1:8794')=>{
 const context=await hostPage.context().browser().newContext({viewport:{width:1100,height:1000}}),reports=[],errors=[];
 const assert=(value,message)=>{if(!value)throw Error(message)};
 try{
  const page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));page.on('console',event=>{if(event.type()==='error')errors.push(event.text())});
  const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
  const primary=()=>page.locator('.prompt-primary textarea'),controlled=()=>page.locator('.prompt-controlled textarea');
  const navigate=async(path='prompt.html')=>{await page.goto(`${baseURL}/${path}`);await page.waitForFunction(()=>window.ready===true);await tick()};
  const sent=async(text)=>{await page.waitForFunction(text=>document.querySelector('.prompt-result')?.textContent===text,text);await tick()};
  for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const mode of ['csr','hydrate']){
   await navigate(`${mode==='csr'?'prompt':`prompt-hydrate-${hue}-${scheme}`}.html?theme=${hue}&scheme=${scheme}`);
   const input=primary();assert(await input.getAttribute('name')==='prompt'&&await input.evaluate(e=>{const s=getComputedStyle(e),content=e.getBoundingClientRect().height-parseFloat(s.paddingTop)-parseFloat(s.paddingBottom)-parseFloat(s.borderTopWidth)-parseFloat(s.borderBottomWidth);return Math.abs(content-3*parseFloat(s.lineHeight))<0.2}),'prompt has a real named multiline successful control');
   assert(await input.evaluate(e=>e.labels?.[0]?.textContent==='Message'),'Field names the actual textarea');
   await input.fill('Actual accepted 🙂\nSecond line');await tick();await input.focus();await input.evaluate(e=>{window.promptEditor=e;e.setSelectionRange(2,6,'backward')});
   await input.press('Enter');await sent('Text [Actual accepted 🙂\nSecond line]; files 0');
   assert(await input.evaluate(e=>e===window.promptEditor&&e===document.activeElement&&e.selectionStart===2&&e.selectionEnd===6&&e.value==='Actual accepted 🙂\nSecond line'),'sending preserves actual editor identity, focus, UTF16 selection and text');
   assert(await input.evaluate(e=>new FormData(e.form).get('prompt'))==='Actual accepted 🙂\nSecond line','actual FormData keeps accepted LF content');
   const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();assert(ax.nodes.some(n=>!n.ignored&&n.role?.value==='textbox'&&n.name?.value==='Message'&&n.properties?.some(p=>p.name==='multiline'&&p.value?.value===true)),'actual AX exposes the labelled multiline editor');
   await page.setViewportSize({width:390,height:1000});await tick();assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),'prompt fits a narrow viewport');await page.setViewportSize({width:1100,height:1000});
   reports.push(`prompt-${hue}-${scheme}-${mode}`);
  }
  await navigate();const accepted='Accepted controlled draft';await controlled().fill('Refused proposal');await tick();assert(await controlled().inputValue()===accepted,'controlled edit refusal restores accepted text');
  await page.getByRole('button',{name:'Send controlled',exact:true}).click();await sent(`Controlled [${accepted}]`);
  await page.getByRole('button',{name:'Accept controlled edits',exact:true}).click();await controlled().fill('Now accepted');await tick();await controlled().press('End');await controlled().press('Enter');await tick();assert(await controlled().inputValue()==='Now accepted\n','ButtonOnly preserves native multiline Enter');
  await page.getByRole('button',{name:'Send controlled',exact:true}).click();await sent('Controlled [Now accepted\n]');reports.push('prompt-controlled-refusal-and-button-only');
  await primary().fill('Shift');await primary().press('End');await primary().press('Shift+Enter');await tick();assert(await primary().inputValue()==='Shift\n','Shift+Enter remains a native line break');
  const cdp=await context.newCDPSession(page);try{
   await primary().focus();await cdp.send('Input.imeSetComposition',{text:'公開',selectionStart:2,selectionEnd:2});await tick();const preedit=await primary().inputValue();
   await primary().evaluate(e=>e.dispatchEvent(new KeyboardEvent('keydown',{key:'Enter',isComposing:true,bubbles:true,cancelable:true})));await tick();assert(await primary().inputValue()===preedit&&await page.locator('.prompt-sends').textContent()==='Sent 0; cancelled 0; read 0','composing Enter neither sends nor replaces preedit');
   await cdp.send('Input.insertText',{text:'公開'});await tick();await primary().press('Enter');await sent(`Text [${await primary().inputValue()}]; files 0`);
  }finally{await cdp.detach()}
  reports.push('prompt-shift-enter-and-browser-ime');
  await page.getByRole('button',{name:'Mark request busy',exact:true}).click();await tick();assert(await primary().isDisabled(),'busy editor is actually disabled');
  await page.locator('.prompt-primary').getByRole('button',{name:'Cancel request',exact:true}).click();await tick();assert(await primary().isDisabled()&&await page.locator('.prompt-sends').textContent()==='Sent 1; cancelled 1; read 0','cancel only calls application handler and does not clear busy');
  await page.getByRole('button',{name:'Release request busy',exact:true}).click();await page.getByRole('button',{name:'Toggle prompt readonly',exact:true}).click();await tick();assert(await primary().getAttribute('readonly')!==null&&!await primary().isEditable(),'readonly is a real textarea property');assert(await page.locator('.prompt-primary').getByRole('button',{name:'Send message',exact:true}).isDisabled(),'readonly disables submission');await page.getByRole('button',{name:'Toggle prompt readonly',exact:true}).click();reports.push('prompt-busy-readonly-and-explicit-cancel');
  await page.getByRole('button',{name:'Explain this source',exact:true}).click();await tick();assert(await primary().inputValue()==='Please explain the application source.\nUse public information.','suggestion inserts an accepted text proposal');assert(await page.locator('.prompt-sends').textContent()==='Sent 1; cancelled 1; read 0','suggestion does not send');reports.push('prompt-suggestion-is-not-submit');
  const fs=await import('node:fs');const path='D:/BuildCache/sapid/tmp/prompt-actual.txt';fs.writeFileSync(path,'Actual opaque file\n');const chooserPromise=page.waitForEvent('filechooser');await page.locator('.prompt-primary').getByRole('button',{name:'Attach files',exact:true}).click();const chooser=await chooserPromise;assert(chooser.isMultiple(),'declared multiple chooser policy');await chooser.setFiles([path]);await tick();
  await page.waitForFunction(()=>document.querySelector('.prompt-primary .sapid-file-attachment')!==null);await page.locator('.prompt-primary').getByRole('button',{name:'Send message',exact:true}).click();await sent('Text [Please explain the application source.\nUse public information.]; files 1');await page.waitForFunction(()=>document.querySelector('.prompt-sends')?.textContent==='Sent 2; cancelled 1; read 4');
  await page.getByRole('button',{name:'Remove prompt-actual.txt',exact:true}).click();await tick();assert(await page.locator('.prompt-primary .sapid-file-attachment').count()===0,'removal retires current opaque attachment');reports.push('prompt-real-picker-and-query-from-send');
  await page.goto(`${baseURL}/prompt-hydrate-blue-light.html?manual=1`);await page.waitForFunction(()=>window.moduleReady===true);const dirty=primary();await dirty.fill('Dirty pre-adoption draft 🙂');await dirty.focus();await dirty.evaluate(e=>{window.promptBefore=e;e.setSelectionRange(3,8,'backward')});
  await page.evaluate(()=>demo.mount_prompt(true,'blue','light'));await tick();assert(await dirty.evaluate(e=>e===window.promptBefore&&e.value==='Dirty pre-adoption draft 🙂'&&e===document.activeElement&&e.selectionStart===3&&e.selectionEnd===8),'adoption keeps dirty accepted content and editor selection');
  await dirty.press('Enter');await sent('Text [Dirty pre-adoption draft 🙂]; files 0');await page.evaluate(()=>demo.unmount_display());assert(await page.locator('.sapid-prompt-input').count()===0,'Owner disposal removes forms, listeners and picker');reports.push('prompt-dirty-adoption-and-owner-cleanup');
  assert(errors.length===0,`prompt errors: ${errors.join('; ')}`);return{reports,errors};
 }finally{await context.close()}
}
