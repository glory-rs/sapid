async(hostPage,baseURL='http://127.0.0.1:8794')=>{
 const context=await hostPage.context().browser().newContext({viewport:{width:1100,height:1000}}),reports=[],errors=[];
 const assert=(value,message)=>{if(!value)throw Error(message)};
 try{
  const page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));page.on('console',event=>{if(event.type()==='error')errors.push(event.text())});
  const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
  const status=()=>page.locator('.questionnaire-status').textContent();
  const step=async(index)=>{await page.waitForFunction(index=>document.querySelector('.questionnaire-status')?.textContent.startsWith(`Step ${index};`),index);await tick()};
  const next=()=>page.getByRole('button',{name:'Next question step',exact:true});const submit=()=>page.getByRole('button',{name:'Submit answers',exact:true});
  const name=()=>page.locator('.questionnaire-primary input[name="name"]');const consent=()=>page.getByRole('checkbox',{name:'Permission',exact:true});
  const load=async(path='questionnaire.html')=>{await page.goto(`${baseURL}/${path}`);await page.waitForFunction(()=>window.ready===true);await tick()};
  for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const mode of ['csr','hydrate']){
   await load(`${mode==='csr'?'questionnaire':`questionnaire-hydrate-${hue}-${scheme}`}.html?theme=${hue}&scheme=${scheme}`);
   assert(await page.getByRole('heading',{name:'Application questionnaire',exact:true}).count()===1,'questionnaire has a real heading');
   assert(await name().evaluate(e=>e.form?.tagName==='FORM'&&e.name==='name'&&e.labels?.[0]?.textContent==='Your name'),'typed question uses a real labelled successful control');
   await next().click();await page.waitForFunction(()=>document.body.textContent.includes('Name is required'));assert((await status()).startsWith('Step 0;'),'invalid typed answer cannot advance');
   await name().fill('Accepted 🙂');await name().evaluate(e=>window.questionnaireEditor=e);await next().click();await step(1);
   assert(!await name().isVisible()&&await name().isDisabled(),'inactive step is kept mounted, hidden and really disabled');
   await consent().check();await submit().click();await page.waitForFunction(()=>document.querySelector('.questionnaire-result')?.textContent==='Accepted [Accepted 🙂]; permission true');
   assert((await status()).endsWith('completed 1'),'only validated answers reach the application');reports.push(`questionnaire-${hue}-${scheme}-${mode}`);
  }
  await load();await name().fill('Earlier valid answer');await name().evaluate(e=>window.questionnaireEditor=e);await next().click();await step(1);
  await submit().click();await page.waitForFunction(()=>document.body.textContent.includes('Permission is required'));assert((await status()).endsWith('completed 0'),'bool question validator blocks completion');
  await consent().check();await page.getByRole('button',{name:'Invalidate previous answer',exact:true}).click();await submit().click();await step(0);
  assert(await name().evaluate(e=>e===window.questionnaireEditor&&e.value===''),'earlier invalid answer returns to the same mounted editor');assert(await page.getByText('Name is required',{exact:true}).isVisible(),'fresh final validation exposes earlier error');
  await name().fill('Repaired answer');await next().click();await step(1);assert(await consent().isChecked(),'typed later answer survives returning to earlier step');await submit().click();await page.waitForFunction(()=>document.querySelector('.questionnaire-result')?.textContent==='Accepted [Repaired answer]; permission true');reports.push('questionnaire-final-revalidates-all-typed-answers');
  await load();await name().fill('Controlled answer');await page.getByRole('button',{name:'Refuse step changes',exact:true}).click();await next().click();await page.waitForFunction(()=>document.querySelector('.questionnaire-status')?.textContent==='Step 0; proposals 1; completed 0');assert(await name().isVisible()&&!await name().isDisabled(),'refused active source retains the accepted panel');
  await page.getByRole('button',{name:'Allow step changes',exact:true}).click();await next().click();await step(1);await page.getByRole('button',{name:'Previous question step',exact:true}).click();await step(0);assert(await name().inputValue()==='Controlled answer','back navigation retains accepted answer');reports.push('questionnaire-controlled-refusal-and-back');
  await page.setViewportSize({width:390,height:1000});await tick();assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),'questionnaire fits a narrow viewport');const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();assert(ax.nodes.some(n=>!n.ignored&&n.role?.value==='textbox'&&n.name?.value==='Your name')&&!ax.nodes.some(n=>!n.ignored&&n.role?.value==='checkbox'&&n.name?.value==='Permission'),'inactive answer is absent from the actual AX tree');reports.push('questionnaire-mobile-and-current-step-ax');
  await page.evaluate(()=>demo.unmount_display());assert(await page.locator('.sapid-questionnaire').count()===0,'Owner retirement removes schema controls and form state');await page.evaluate(()=>demo.mount_questionnaire(false,'blue','light'));await step(0);assert(await name().inputValue()===''&&(await status()).endsWith('completed 0'),'replacement Owner begins with fresh accepted application state');reports.push('questionnaire-owner-retirement-and-remount');
  assert(errors.length===0,`questionnaire errors ${errors.join('; ')}`);return{reports,errors};
 }finally{await context.close()}
}
