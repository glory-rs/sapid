async (hostPage) => {
  const context=await hostPage.context().browser().newContext({viewport:{width:1440,height:1100},reducedMotion:'no-preference'});
  const reports=[],errors=[];
  const assert=(value,message)=>{if(!value)throw new Error(message);};
  const base=process.env.SAPID_COLLAPSE_BASE||'http://127.0.0.1:8794';
  try {
    const page=await context.newPage();
    const turn=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    const pause=ms=>page.waitForTimeout(ms);
    // Observe a real intermediate frame at activation. Hold its animation clock
    // across external AX/protocol reads so they cannot outlast the phase tested.
    const motionFrame=(locator,name,full,freeze=false)=>locator.evaluate((element,{name,full,freeze})=>new Promise((resolve,reject)=>{
      let frames=0;
      const sample=()=>{
        const transition=element.getAnimations().find(animation=>animation.transitionProperty==='height'&&animation.playState==='running');
        const height=element.getBoundingClientRect().height;
        if(transition&&height>0&&height<full){
          if(freeze)transition.pause();
          const frame=transition.effect?.getKeyframes()[0]?.height||'',factor=frame.match(/([0-9.eE+-]+)\s*\*\s*size/);
          return resolve({height,start:factor?Number(factor[1])*full:parseFloat(frame),frame});
        }
        if(++frames===120)return reject(Error('missing actual intermediate collapse frame'));
        requestAnimationFrame(sample);
      };
      [...document.querySelectorAll('button')].find(button=>button.textContent===name||button.getAttribute('aria-label')===name).click();
      requestAnimationFrame(sample);
    }),{name,full,freeze});
    page.on('pageerror',error=>errors.push(error.message));
    page.on('console',message=>{if(message.type()==='error')errors.push(message.text());});
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]) {
      const mode=`collapse-motion-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
      await page.emulateMedia({reducedMotion:'no-preference'});
      await page.setViewportSize({width:1440,height:1100});
      await page.goto(`${base}/${hydrate?`collapse-motion-hydrate-${hue}-${scheme}.html`:`collapse-motion.html?theme=${hue}&scheme=${scheme}`}`);
      await page.waitForFunction(()=>window.ready===true);
      const root=page.locator('.animated-unmount-collapse'),panel=root.locator('.sapid-collapsible-panel');
      const kept=page.locator('.animated-kept-collapse .kept-collapse-editor');
      const editor=root.locator('.animated-collapse-editor');
      const control=name=>page.getByRole('button',{name,exact:true});
      const source=async name=>{await control(name).evaluate(element=>element.click());await turn();};
      const height=()=>panel.evaluate(element=>element.getBoundingClientRect().height);
      const status=async()=>{
        const match=(await page.locator('.collapse-motion-status').textContent()).match(/Open: (true|false); mounted: (\d+); cleaned: (\d+); proposals: (\d+); order: (.*)/);
        return {open:match[1]==='true',mounted:+match[2],cleaned:+match[3],proposals:+match[4],order:match[5]};
      };
      const settle=()=>page.waitForFunction(()=>{
        const element=document.querySelector('.animated-unmount-collapse .sapid-collapsible-panel');
        return element.getAnimations().length===0;
      });
      const waitBody=present=>page.waitForFunction(present=>!!document.querySelector('.animated-unmount-collapse .animated-collapse-editor')===present,present);
      assert(await height()===0&&await editor.count()===0,`${mode}: initial closed body/height`);
      await kept.evaluate(element=>window.savedKeptCollapse=element);
      const entering=(await motionFrame(panel,'Animated details',128)).height;
      assert(entering>0&&entering<128,`${mode}: enter jumped instead of interpolating (${entering})`);
      await settle();const full=await height();assert(full>=127&&full<=129,`${mode}: intrinsic target ${full}`);
      assert((await status()).order==='typed generic'&&(await status()).proposals===1,`${mode}: callback order`);
      await editor.fill('Reversal keeps this draft');await turn();
      await kept.fill('Persistent kept note');await turn();
      await editor.evaluate(element=>window.savedCollapseEditor=element);
      const closing=(await motionFrame(panel,'Close from source',full,true)).height;
      assert(closing>0&&closing<full&&await editor.count()===1&&(await status()).cleaned===0,`${mode}: closing owner retired before physical end (${closing})`);
      assert(await panel.getAttribute('aria-hidden')==='true'&&await panel.getAttribute('inert')!==null&&await panel.getAttribute('hidden')===null,`${mode}: closing logical state or UA hidden cut-off`);
      const cdp=await context.newCDPSession(page),ax=await cdp.send('Accessibility.getFullAXTree');
      assert(!ax.nodes.some(node=>!node.ignored&&node.role?.value==='textbox'&&node.name?.value==='Animated draft'),`${mode}: closing editor remained in Chrome AX`);
      await cdp.detach();
      const reversal=await motionFrame(panel,'Open from source',full,true);
      const reversed=reversal.height;assert(reversal.start>0&&reversal.start<full&&reversed>=reversal.start-1&&reversed<=full,`${mode}: reversal restarted from an endpoint ${JSON.stringify(reversal)}`);
      await panel.evaluate(element=>element.getAnimations().forEach(animation=>animation.play()));
      await settle();await pause(380);
      assert(await editor.evaluate(element=>element===window.savedCollapseEditor)&&await editor.inputValue()==='Reversal keeps this draft'&&(await status()).cleaned===0,`${mode}: stale close completed after reopen`);
      await editor.focus();await turn();await motionFrame(panel,'Close from source',full);
      assert(await root.getByRole('button',{name:'Animated details',exact:true}).evaluate(element=>element===document.activeElement),`${mode}: focus not restored from closing editor`);
      await waitBody(false);await settle();
      assert(await height()===0&&(await status()).cleaned===1,`${mode}: terminal close did not retire Owner`);
      await source('Open from source');await settle();
      assert(await editor.evaluate(element=>element!==window.savedCollapseEditor),`${mode}: Unmount reused retired body`);
      assert(await kept.evaluate(element=>element===window.savedKeptCollapse)&&await kept.inputValue()==='Persistent kept note',`${mode}: KeepMounted lost editor`);
      const beforeGrowth=await height();await source('Toggle content height');await settle();const grown=await height();
      assert(grown>beforeGrowth+70,`${mode}: intrinsic content height was cached ${beforeGrowth}/${grown}`);
      const beforeClear=(await motionFrame(panel,'Close from source',grown,true)).height;
      assert(beforeClear>0&&beforeClear<grown,`${mode}: clear prerequisite not in flight`);
      await source('Toggle motion');await waitBody(false);
      assert(await height()===0&&await panel.evaluate(element=>element.getAnimations().length===0),`${mode}: Clear left old transition/Owner alive`);
      await source('Set duration zero');await source('Toggle motion');await source('Open from source');await settle();
      assert(await height()===grown,`${mode}: duration-zero did not settle open`);
      await source('Close from source');await waitBody(false);assert(await height()===0,`${mode}: duration-zero did not settle close`);
      await source('Set duration normal');const beforeReduce=(await motionFrame(panel,'Open from source',grown,true)).height;assert(beforeReduce>0&&beforeReduce<grown,`${mode}: reduce prerequisite not in flight`);
      await page.emulateMedia({reducedMotion:'reduce'});await turn();await settle();
      assert(await height()===grown,`${mode}: dynamic Reduce did not immediately settle opening`);
      await source('Close from source');await waitBody(false);assert(await height()===0,`${mode}: reduced close retained Owner`);
      await page.emulateMedia({reducedMotion:'no-preference'});
      await source('Open from source');await settle();await motionFrame(panel,'Close from source',grown,true);
      assert(await height()>0,`${mode}: closing Reduce prerequisite`);
      await page.emulateMedia({reducedMotion:'reduce'});await turn();await waitBody(false);
      assert(await height()===0,`${mode}: mid-close Reduce did not release Owner`);
      await source('Open from source');await settle();await source('Toggle proposal acceptance');
      const refused=await status(),revision=await panel.getAttribute('data-gly-collapse-revision');
      await root.getByRole('button',{name:'Animated details',exact:true}).click();await turn();
      assert((await status()).open&&(await status()).proposals===refused.proposals+1&&await panel.getAttribute('data-gly-collapse-revision')===revision,`${mode}: controlled refusal animated or mutated source`);
      await page.setViewportSize({width:390,height:900});await turn();
      assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${mode}: mobile overflow`);
      if(hydrate)await page.screenshot({path:`D:/BuildCache/sapid/${mode}.png`,fullPage:true});
      await page.emulateMedia({reducedMotion:'no-preference'});await motionFrame(panel,'Close from source',grown,true);
      await page.evaluate(()=>window.demo.unmount_display());await turn();await pause(400);
      assert(await page.locator('#app').textContent()==='',`${mode}: retirement left DOM`);
      reports.push({mode,entering,full,closing,reversed,reversalStart:reversal.start,grown,beforeClear,beforeReduce,delayedUnmount:true,closedAX:true,controlledRefusal:true,midflightReduce:true,ownerRetirement:true});
    }
    await page.emulateMedia({reducedMotion:'no-preference'});
    await page.goto(`${base}/collapse-motion.html`);await page.waitForFunction(()=>window.ready===true);
    await page.addStyleTag({content:'[data-gly-collapse]{interpolate-size:numeric-only!important;transition:none!important}'});
    await page.getByRole('button',{name:'Open from source',exact:true}).click();await turn();
    await page.getByRole('button',{name:'Close from source',exact:true}).click();await turn();await turn();
    assert(await page.locator('.animated-unmount-collapse .animated-collapse-editor').count()===0,'unsupported interpolation fallback retained Owner');
    reports.push({mode:'collapse-motion-no-interpolation',immediateTerminal:true});
    await page.evaluate(()=>window.demo.unmount_display());await turn();
    await page.goto(`${base}/collapse-motion-hydrate-blue-light.html?manual=1`);await page.waitForFunction(()=>window.moduleReady===true);
    const rejected=await page.evaluate(()=>{
      const panel=document.querySelector('.animated-unmount-collapse .sapid-collapsible-panel');
      const later=document.querySelector('.animated-kept-collapse .kept-collapse-editor');
      panel.setAttribute('data-gly-collapse-revision','41');
      const original=later.getAttribute('aria-label');later.setAttribute('aria-label','Wrong later label');
      let rejected=false;try{window.demo.mount_collapse_motion(true,'blue','light')}catch(_){rejected=true}
      const unchanged=panel.getAttribute('data-gly-collapse-revision')==='41';
      later.setAttribute('aria-label',original);
      window.beforeRevisionPanel=panel;window.demo.mount_collapse_motion(true,'blue','light');
      return {rejected,unchanged,revision:panel.getAttribute('data-gly-collapse-revision'),same:panel===document.querySelector('.animated-unmount-collapse .sapid-collapsible-panel')};
    });await turn();
    assert(rejected.rejected&&rejected.unchanged&&rejected.revision==='0'&&rejected.same,'revision adoption mutated a failed subtree or replaced the live panel');
    reports.push({mode:'collapse-motion-transactional-adoption',failedSubtreeUnchanged:true,newLifetimeRevision:true});
    await page.evaluate(()=>window.demo.unmount_display());await turn();
    await page.goto(`${base}/collapse-stream.html`);
    await page.waitForFunction(()=>document.querySelector('.stream-collapse-panel')?.getAttribute('data-gly-collapse-revision')==='1');
    assert(await page.locator('.stream-collapse-panel').getAttribute('data-gly-collapse')==='open','classic stream did not apply the finite open config');
    await page.locator('.stream-collapse-editor').evaluate(element=>{window.beforeStreamEditor=element;window.beforeStreamPanel=element.closest('.stream-collapse-panel');element.value='Dirty streamed draft';element.focus();element.setSelectionRange(2,8)});
    await page.evaluate(async()=>{const demo=await import('./sapid_web_example.js');await demo.default();demo.mount_collapse_stream_probe();window.demo=demo});await turn();
    assert(await page.locator('.stream-collapse-panel').evaluate(element=>element===window.beforeStreamPanel&&element.getAttribute('data-gly-collapse-revision')==='0'),'stream adoption did not reset revision in place');
    assert(await page.locator('.stream-collapse-editor').evaluate(element=>element===window.beforeStreamEditor&&element.value==='Dirty streamed draft'&&element===document.activeElement&&element.selectionStart===2&&element.selectionEnd===8),'stream adoption lost dirty input, selection or focus');
    const streamHeight=(await motionFrame(page.locator('.stream-collapse-panel'),'Close streamed collapse',96)).height;
    assert(streamHeight>0&&streamHeight<96,'stream-adopted close did not interpolate');
    await page.waitForFunction(()=>document.querySelector('.stream-collapse-completed')?.textContent==='Completed: 1');
    assert(await page.locator('.stream-collapse-panel').evaluate(element=>element.getBoundingClientRect().height===0),'stream completion preceded settled height');
    await pause(280);assert(await page.locator('.stream-collapse-completed').textContent()==='Completed: 1','duplicate stream terminal event');
    await page.evaluate(()=>window.demo.unmount_display());await turn();
    reports.push({mode:'collapse-motion-classic-stream',capturedRevision:1,adoptedRevision:0,streamHeight,dirtyInput:true,terminalOnce:true});
    const fallback=await context.newPage();
    fallback.on('pageerror',error=>errors.push(error.message));
    await fallback.addInitScript(()=>{const original=CSS.supports.bind(CSS);CSS.supports=(property,value)=>property==='interpolate-size'?false:original(property,value)});
    await fallback.goto(`${base}/collapse-motion.html`);await fallback.waitForFunction(()=>window.ready===true);
    assert(await fallback.locator('#app').getAttribute('data-gly-collapse-static')!==null,'unsupported host did not disable physical transition');
    await fallback.getByRole('button',{name:'Open from source',exact:true}).click();
    await fallback.getByRole('button',{name:'Close from source',exact:true}).click();
    await fallback.waitForFunction(()=>!document.querySelector('.animated-unmount-collapse .animated-collapse-editor'));
    assert(await fallback.locator('.animated-unmount-collapse .sapid-collapsible-panel').evaluate(element=>element.getAnimations().length===0&&element.getBoundingClientRect().height===0),'unsupported host retained an unobserved physical animation');
    await fallback.evaluate(()=>window.demo.unmount_display());await fallback.close();
    reports.push({mode:'collapse-motion-unsupported-host',physicalMotionDisabled:true,immediateTerminal:true});
    assert(errors.length===0,errors.join('\n'));
    return {reports,errors};
  } finally {await context.close();}
}
