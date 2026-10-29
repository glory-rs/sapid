async(hostPage,baseURL='http://127.0.0.1:8794')=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1100,height:850},hasTouch:true}),reports=[],errors=[];
  const assert=(value,message)=>{if(!value)throw new Error(message)};
  const near=(actual,expected)=>Math.abs(actual-expected)<1.5;
  try{
    const page=await context.newPage();page.on('pageerror',e=>errors.push(e.message));page.on('console',e=>{if(e.type()==='error')errors.push(e.text())});
    const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    const trigger=()=>page.getByRole('button',{name:'Open placement',exact:true});
    const panel=()=>page.locator('.sapid-popover');
    const opened=()=>page.waitForFunction(()=>document.querySelector('.sapid-popover')?.matches(':popover-open'));
    const closed=()=>page.waitForFunction(()=>!document.querySelector('.sapid-popover')?.matches(':popover-open'));
    const state=(kind,value)=>page.evaluate(([k,v])=>demo.popover_placement_state(k,v),[kind,value]);
    const select=async(index,edge=false,flip=true,shift=true)=>{await page.evaluate(args=>demo.popover_placement_case(...args),[index,edge,flip,shift]);await tick()};
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]){
      await page.goto(`${baseURL}/${hydrate?`popover-placement-hydrate-${hue}-${scheme}.html`:'popover-placement.html'}?theme=${hue}&scheme=${scheme}`);
      await page.waitForFunction(()=>window.ready===true);
      for(let index=0;index<12;index++){
        await select(index);await trigger().click();await opened();await tick();
        const anchor=await trigger().boundingBox(),p=await panel().boundingBox(),side=['top','bottom','left','right'][Math.floor(index/3)],align=index%3;
        const vertical=index<6,cross=vertical?anchor.x:anchor.y,extent=vertical?anchor.width:anchor.height,size=vertical?p.width:p.height;
        const expectedCross=cross+(align===0?0:align===1?(extent-size)/2:extent-size);
        const expected=side==='top'?[expectedCross,anchor.y-p.height-12]:side==='bottom'?[expectedCross,anchor.y+anchor.height+12]:side==='left'?[anchor.x-p.width-12,expectedCross]:[anchor.x+anchor.width+12,expectedCross];
        assert(near(p.x,expected[0])&&near(p.y,expected[1]),`physical ${side}/${align} geometry: ${JSON.stringify({p,expected})}`);
        assert(await panel().getAttribute('data-gly-layer')==='anchor'&&await panel().getAttribute('data-gly-anchor-side')===side,'single typed placement shape');
        assert(await page.getByRole('textbox',{name:'Placement draft'}).evaluate(e=>e===document.activeElement),'real input initial focus');
        if(index===0){
          const cdp=await context.newCDPSession(page),ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();
          assert(ax.nodes.some(n=>n.role?.value==='dialog'&&n.name?.value==='Open placement'),'Popover retains named Dialog semantics');
          assert(await trigger().evaluate(e=>e.tagName==='BUTTON'&&e.textContent.includes('Visual trigger')),'trigger slot retains actual Button');
          assert(await page.getByRole('button',{name:'Close placement',exact:true}).evaluate(e=>e.tagName==='BUTTON'&&e.textContent.includes('Visual close')),'close slot retains actual Button');
        }
        await page.keyboard.press('Escape');await closed();assert(await trigger().evaluate(e=>e===document.activeElement),'focus returns to actual trigger');
      }
      reports.push({suite:`popover-placement-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`,status:'passed',checks:'twelve actual placements, custom gap, slots, focus and AX'});
    }
    await select(0,true);await trigger().click();await opened();await tick();
    let a=await trigger().boundingBox(),p=await panel().boundingBox();assert(near(p.y,a.y+a.height+12)&&p.x>=15,'edge flip and independent viewport shift');
    await select(0,true,false,false);await trigger().click();await opened();await tick();a=await trigger().boundingBox();p=await panel().boundingBox();assert(near(p.y,a.y-p.height-12)&&p.y<0&&near(p.x,a.x),'disabled collision policies preserve requested geometry');
    await page.keyboard.press('Escape');await closed();reports.push({suite:'popover-collision-policies',status:'passed'});
    await select(3);await trigger().click();await opened();const beforeLoss=Number((await page.locator('.placement-status').textContent()).split('dismissals:')[1]);await state('reject',true);await state('anchor-hidden',true);await closed();await tick();
    assert(await page.locator('.placement-status').textContent()===`open:true dismissals:${beforeLoss+1}`,'anchor loss proposes once while source refuses');
    await state('anchor-hidden',false);await opened();await page.getByRole('button',{name:'Close placement',exact:true}).click();await tick();
    assert(await panel().evaluate(e=>e.matches(':popover-open')),'controlled close slot refusal preserves actual panel');
    await state('reject',false);await state('anchor-hidden',true);await closed();await tick();assert((await page.locator('.placement-status').textContent()).startsWith('open:false'),'accepted anchor loss closes model');
    await state('anchor-hidden',false);reports.push({suite:'popover-anchor-lifetime-controlled-refusal',status:'passed'});
    await select(8);await trigger().tap();await opened();await page.getByRole('button',{name:'Close placement',exact:true}).tap();await closed();await trigger().click();await opened();await page.evaluate(()=>demo.unmount_popover_placement());
    assert(await page.locator(':popover-open').count()===0&&await page.locator('#app [data-gly-node]').count()===0,'Owner retirement releases top layer and slot nodes');reports.push({suite:'popover-touch-owner-cleanup',status:'passed'});
    await page.goto(`${baseURL}/popover-placement-hydrate-blue-light.html?manual`);await page.waitForFunction(()=>window.moduleReady===true);
    assert(await page.evaluate(()=>{const root=document.querySelector('#app'),original=root.innerHTML,panel=root.querySelector('[data-gly-anchor-gap]');panel.setAttribute('data-gly-anchor-gap','99');const modified=root.innerHTML;try{demo.mount_popover_placement(true,'blue','light');return false;}catch(_){if(root.innerHTML!==modified)return false;}root.innerHTML=original.replace('gly:v:14','gly:v:13').replace('gly:/v:14','gly:/v:13');const old=root.innerHTML;try{demo.mount_popover_placement(true,'blue','light');return false;}catch(_){return root.innerHTML===old;}finally{root.innerHTML=original;}}),'adoption rejects changed finite field and old version without root mutation');
    await page.evaluate(()=>demo.mount_popover_placement(true,'blue','light'));await trigger().click();await opened();reports.push({suite:'popover-adoption-atomic-rejection',status:'passed'});
    await page.goto(`${baseURL}/anchor-stream.html`);await page.waitForFunction(()=>document.querySelector('#anchor-stream-root')?.dataset.glyStreamState==='ready');
    assert(await page.locator('#stream-anchor-panel').evaluate(e=>e.getAttribute('data-gly-layer')==='anchor'&&e.getAttribute('data-gly-anchor-side')==='top'&&e.getAttribute('data-gly-anchor-align')==='end'&&e.getAttribute('data-gly-anchor-gap')==='17'&&e.getAttribute('data-gly-anchor-padding')==='23'&&e.getAttribute('data-gly-anchor-flip')==='false'&&e.getAttribute('data-gly-anchor-shift')==='false'),'classic patch preserves all six fields');
    assert(await page.evaluate(()=>{if(GloryStream.active()!==0||document.querySelector('#anchor-stream-root').firstChild.nodeValue!=='gly:v:14')return false;for(const version of [undefined,13]){const root=document.createElement('div');root.setAttribute('data-gly-stream-pending','');root.innerHTML='<i>untouched</i>';document.body.append(root);const before=root.outerHTML;try{GloryStream.start(root,{version:2,kind:'start',sequence:0,html_version:version,data_id:'rejected-anchor',nodes:[]});return false;}catch(_){if(root.outerHTML!==before)return false;}finally{root.remove();}}return true;}),'classic old Start refuses before DOM mutation and End hands off version14');
    reports.push({suite:'popover-classic-policy-and-old-wire',status:'passed'});
    assert(errors.length===0,errors.join('\n'));return{reports,errors};
  }finally{await context.close()}
}
