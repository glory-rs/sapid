async (hostPage) => {
 const context=await hostPage.context().browser().newContext({viewport:{width:1200,height:1100}});const reports=[],errors=[];
 const assert=(value,message)=>{if(!value)throw new Error(message)};
 try{
  const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
  const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
  for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydration of [false,true]){
   const mode=`stepper-${hue}-${scheme}-${hydration?'hydrate':'csr'}`;
   const url=`http://127.0.0.1:8794/${hydration?`stepper-hydrate-${hue}-${scheme}.html`:`stepper.html?theme=${hue}&scheme=${scheme}`}`;
   await page.goto(url);await page.waitForFunction(()=>window.ready===true);
   const workflow=page.locator('.main-stepper');const header=index=>workflow.locator('.sapid-stepper-trigger').nth(index);
   const email=workflow.getByRole('textbox',{name:'Account email',exact:true});const status=page.locator('.stepper-status');
   const current=()=>workflow.locator('[aria-current=step]').textContent();
   assert(await current()==='1. Account'&&await workflow.getByRole('list').count()===1,`${mode}: current/list semantics missing`);
   assert(await header(2).isDisabled(),`${mode}: forward skip enabled by default`);
   assert(await page.locator('.readonly-stepper .sapid-stepper-headers').evaluate(e=>getComputedStyle(e).flexDirection)==='column',`${mode}: vertical layout missing`);
   assert(await page.locator('.readonly-stepper .sapid-stepper-trigger:not(:disabled)').count()===0,`${mode}: default header selection is enabled`);
   await header(1).click();await tick();assert(await current()==='1. Account',`${mode}: failed form validation advanced workflow`);
   assert(await email.getAttribute('aria-invalid')==='true'&&(await workflow.locator('.sapid-stepper-description').first().textContent()).includes('Error'),`${mode}: form errors not reflected in step status`);
   await email.fill('alex@example.test');await email.evaluate(e=>window.firstStepInput=e);await email.press('Enter');await tick();
   assert(await current()==='2. Details',`${mode}: implicit form Enter did not pass validation barrier`);
   const note=workflow.getByRole('textbox',{name:'Review note'});await note.fill('Kept local note');await note.evaluate(e=>window.keptNote=e);
   await header(0).click();await tick();assert(await email.evaluate(e=>e===window.firstStepInput),`${mode}: kept first panel input replaced`);
   await page.evaluate(()=>window.demo.stepper_state(false,true,false,false));await header(1).focus();await page.keyboard.press('Space');await tick();assert(await current()==='1. Account',`${mode}: rejected controlled change advanced workflow`);
   await page.evaluate(()=>window.demo.stepper_state(true,false,false,false));await tick();assert(await header(1).isDisabled(),`${mode}: disabled header is not native-disabled`);
   await page.evaluate(()=>window.demo.stepper_state(true,true,true,false));await header(2).focus();await page.keyboard.press('Enter');await tick();assert(await current()==='3. Review',`${mode}: explicit skip request failed`);
   await workflow.getByRole('button',{name:'Finish workflow',exact:true}).click();await tick();assert(await workflow.locator('[aria-current=step]').count()===0&&await workflow.getByText('Your workflow is complete.',{exact:true}).isVisible(),`${mode}: completion stage still claims current header`);
   await page.evaluate(()=>window.demo.stepper_active(0));await email.focus();await page.evaluate(()=>window.demo.stepper_active(1));await tick();
   assert(await workflow.locator('.sapid-stepper-panel:visible').evaluate(e=>document.activeElement===e),`${mode}: source change did not restore actual focus to new panel`);
   assert(await note.evaluate(e=>e===window.keptNote)&&await note.inputValue()==='Kept local note',`${mode}: KeepMounted lost local input draft`);
   await page.evaluate(()=>window.demo.stepper_active(0));await page.evaluate(()=>window.demo.stepper_state(true,true,false,true));
   await header(1).click();await page.waitForFunction(()=>document.querySelector('.stepper-status').textContent.includes('pending=1'));
   assert(await current()==='1. Account',`${mode}: pending async validation advanced workflow`);
   await page.evaluate(()=>window.demo.stepper_finish_async(false));await tick();assert(await current()==='1. Account'&&(await workflow.locator('.sapid-stepper-description').first().textContent()).includes('Server rejected'),`${mode}: async error did not block progress`);
   await header(1).click();await page.waitForFunction(()=>document.querySelector('.stepper-status').textContent.includes('pending=1'));
   await email.fill('changed@example.test');await tick();await page.evaluate(()=>window.demo.stepper_finish_async(true));await tick();assert(await current()==='1. Account',`${mode}: cancelled late validation advanced workflow`);
   await header(1).click();await page.waitForFunction(()=>document.querySelector('.stepper-status').textContent.includes('pending=1'));await page.evaluate(()=>window.demo.stepper_finish_async(true));await tick();assert(await current()==='2. Details',`${mode}: valid async barrier did not advance workflow`);
   assert((await status.textContent()).includes('ordered=true'),`${mode}: generic observer preceded typed request`);
   assert(await workflow.locator('[role=tab],[role=tablist],[aria-selected]').count()===0,`${mode}: progress misrepresented as tab selection`);
   const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();assert(ax.nodes.some(n=>n.role?.value==='list'&&n.name?.value==='Steps'),`${mode}: AX steps list missing`);
   await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});reports.push({suite:mode,status:'passed',checks:'Current::Step/list/description, optional/error/completion, explicit skip, mouse/Enter/Space, controlled rejection, native disabled headers, sync/async FormState barriers, cancellation of late results, actual panel focus, KeepMounted input identity/draft, AX list'});
  }
  await page.goto('http://127.0.0.1:8794/stepper-hydrate-green-dark.html?manual');await page.waitForFunction(()=>window.moduleReady===true);
  await page.locator('.stepper-email').fill('before@hydrate.test');await page.locator('.stepper-email').evaluate(e=>{window.preStepInput=e;e.focus();e.setSelectionRange(2,8,'backward')});
  await page.evaluate(()=>window.demo.mount_stepper(true,'green','dark'));await tick();
  assert(await page.locator('.stepper-email').evaluate(e=>e===window.preStepInput&&e.value==='before@hydrate.test'&&e.selectionStart===2&&e.selectionEnd===8&&document.activeElement===e),'Stepper dirty hydration replaced draft/caret/focus');
  await page.locator('.main-stepper .sapid-stepper-trigger').nth(1).click();await tick();assert(await page.locator('.main-stepper [aria-current=step]').textContent()==='2. Details','hydrated email was not adopted into validation source');
  await page.evaluate(()=>window.demo.unmount_stepper());await tick();assert(await page.locator('.stepper-demo').count()===0,'Stepper Owner retirement leaked content');
  reports.push({suite:'stepper-dirty-hydration-retirement',status:'passed'});return{reports,errors};
 }finally{await context.close()}
}
