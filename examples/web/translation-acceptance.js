async function runTranslationAcceptance(page) {
  const base=process.env.SAPID_TRANSLATION_BASE||process.env.SAPID_BASE_URL||'http://127.0.0.1:8794',reports=[],errors=[];
  page.on('pageerror',error=>errors.push(error.message));
  const assert=(value,message)=>{if(!value)throw new Error(message);};
  const turn=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
  const command=name=>page.getByRole('button',{name,exact:true}).evaluate(element=>element.click());
  const cube=()=>page.locator('.translation-cube');
  const x=()=>cube().evaluate(element=>element.getBoundingClientRect().x);
  const active=()=>cube().evaluate(element=>element.getAnimations().some(animation=>animation.playState==='running'));
  const focus=()=>page.locator('input[aria-label="Motion focus probe"]');
  for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]) {
    const mode=`translation-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
    await page.setViewportSize({width:960,height:860});await page.emulateMedia({reducedMotion:'no-preference'});
    await page.goto(`${base}/${hydrate?`translation-hydrate-${hue}-${scheme}.html`:`translation.html?theme=${hue}&scheme=${scheme}`}`);await page.waitForFunction(()=>window.ready===true);await turn();
    const start=await x();assert(!await active(),`${mode}: first appearance animated`);
    await focus().fill('Kept 🎯 motion selection');await focus().focus();await focus().evaluate(element=>element.setSelectionRange(2,9));const editor=await focus().elementHandle(),identity=await cube().elementHandle();
    await command('Move forty');await page.waitForTimeout(100);const middle=await x();assert(middle>start+1&&middle<start+39&&await active(),`${mode}: missing actual intermediate transform`);
    await command('Toggle translation pause');await turn();const paused=await x();await page.waitForTimeout(140);assert(Math.abs(await x()-paused)<0.05,`${mode}: pause lost phase`);
    await command('Observe translation bounds');await page.waitForFunction(()=>document.querySelector('.translation-query-status')?.textContent!=='Not observed');
    const query=Number((await page.locator('.translation-query-status').textContent()).split(',')[0]);assert(Math.abs(query-await x())<0.1,`${mode}: physical Bounds disagree with current translated ink`);
    await command('Reverse target');await turn();const interrupted=await x();assert(Math.abs(interrupted-paused)<8,`${mode}: interruption jumped to endpoint`);await page.waitForTimeout(80);assert(await x()<interrupted,`${mode}: reversed translation did not move`);
    await page.emulateMedia({reducedMotion:'reduce'});await turn();assert(Math.abs(await x()-(start-20))<0.1&&!await active(),`${mode}: live Reduce did not settle target`);
    await page.emulateMedia({reducedMotion:'no-preference'});await turn();await command('Move forty');await page.waitForTimeout(60);await command('Clear translation');await turn();assert(Math.abs(await x()-start)<0.1&&!await active(),`${mode}: Clear retained current animation`);
    await command('Repeat translation');await page.waitForTimeout(90);const looped=await x();await page.waitForTimeout(90);assert(Math.abs(await x()-looped)>3&&await active(),`${mode}: finite loop did not move`);
    await command('Toggle translation visibility');await turn();assert(await cube().getAttribute('data-gly-hidden')!==null&&!await active(),`${mode}: hidden loop kept running`);await command('Toggle translation visibility');await turn();assert(await active(),`${mode}: reappearance did not admit loop`);
    await page.emulateMedia({reducedMotion:'reduce'});await turn();assert(Math.abs(await x()-start)<0.1&&!await active(),`${mode}: reduced loop did not use static start`);
    const focusState=await editor.evaluate(element=>({same:element===document.querySelector('input[aria-label="Motion focus probe"]'),focused:element===document.activeElement,start:element.selectionStart,end:element.selectionEnd,value:element.value}));
    assert(focusState.same&&focusState.focused&&focusState.start===2&&focusState.end===9&&await identity.evaluate(element=>element===document.querySelector('.translation-cube')),`${mode}: motion replaced identity or changed actual editor focus ${JSON.stringify(focusState)}`);
    const session=await page.context().newCDPSession(page);const ax=await session.send('Accessibility.getFullAXTree');await session.detach();assert(ax.nodes.filter(node=>!node.ignored&&node.role?.value==='StaticText'&&node.name?.value==='Balance 123').length===1,`${mode}: duplicated accessible visual text`);
    if(hydrate)await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});
    await page.evaluate(()=>demo.unmount_display());await turn();assert(await identity.evaluate(element=>!element.isConnected&&element.getAnimations().length===0),`${mode}: retired animation survived Owner`);
    reports.push({mode,physicalBounds:true,intermediateTransform:true,pause:true,sourceInterrupt:true,clear:true,liveReduce:true,focusAndIdentity:true,oneAccessibleText:true});
  }
  await page.emulateMedia({reducedMotion:'no-preference'});await page.goto(`${base}/translation-stream.html`);await page.waitForFunction(()=>document.querySelector('#translation-stream-root')?.dataset.glyStreamState==='ready');await turn();
  const streamed=page.locator('.stream-translation'),handle=await streamed.elementHandle();
  assert(await page.evaluate(()=>GloryStream.active()===0&&document.querySelector('#translation-stream-root').firstChild.nodeValue==='gly:v:14'), 'translation classic: wrong End/released controller');
  const initial=await streamed.evaluate(element=>element.getBoundingClientRect().x);await page.waitForTimeout(100);assert(Math.abs(await streamed.evaluate(element=>element.getBoundingClientRect().x)-initial)>2&&await streamed.evaluate(element=>element.getAnimations().some(animation=>animation.playState==='running')),'translation classic: live loop did not survive End');
  await page.emulateMedia({reducedMotion:'reduce'});await turn();assert(await streamed.evaluate(element=>Math.abs(element.getBoundingClientRect().x-8)<0.1&&element.getAnimations().length===0),'translation classic: live Reduce failed');
  await page.emulateMedia({reducedMotion:'no-preference'});await turn();assert(await streamed.evaluate(element=>element.getAnimations().some(animation=>animation.playState==='running')),'translation classic: preference restart failed');
  await page.evaluate(()=>document.querySelector('#translation-stream-root').remove());await turn();assert(await handle.evaluate(element=>element.getAnimations().length===0),'translation classic: detached ready root retained animation');
  reports.push({mode:'translation-classic-stream',rustProducer:true,endHandoff:true,liveReduce:true,retirement:true});assert(errors.length===0,`translation page errors: ${errors.join(';')}`);return {reports,errors};
}
