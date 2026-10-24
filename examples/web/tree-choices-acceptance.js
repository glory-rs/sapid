async (hostPage) => {
  const browser=hostPage.context().browser(),reports=[],errors=[];
  const base=process.env.SAPID_TREE_CHOICES_BASE||'http://127.0.0.1:8794';
  const assert=(ok,message)=>{if(!ok)throw new Error(message)};
  const context=await browser.newContext({viewport:{width:1440,height:1100}});
  try {
    const page=await context.newPage();
    page.on('pageerror',error=>errors.push(error.message));
    page.on('console',message=>{if(message.type()==='error')errors.push(message.text())});
    const turn=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))));
    const main=()=>page.getByRole('combobox',{name:'Project destination',exact:true});
    const popup=async trigger=>page.locator(`#${await trigger.getAttribute('aria-controls')}`);
    const item=(root,name)=>root.getByRole('treeitem',{name,exact:true});
    const current=()=>page.locator('.path-choice-value').textContent();
    const form=()=>page.locator('form').evaluate(form=>new FormData(form).getAll('destination'));
    for(const columns of [false,true]) {
      const slug=columns?'cascader':'tree-select',mount=columns?'mount_cascader':'mount_tree_select';
      for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]) {
        const mode=`${slug}-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
        await page.setViewportSize({width:1440,height:1100});
        await page.goto(`${base}/${hydrate?`${slug}-hydrate-${hue}-${scheme}.html`:`${slug}.html?theme=${hue}&scheme=${scheme}`}`);
        await page.waitForFunction(()=>window.ready===true);await turn();
        const root=await popup(main());
        assert(await main().getAttribute('aria-haspopup')==='tree'&&await root.getAttribute('role')==='tree',`${mode}: false combobox/tree popup`);
        assert(await main().getAttribute('aria-activedescendant')===null&&!await root.getByRole('option').count(),`${mode}: listbox substitution`);
        await main().focus();await page.keyboard.press('ArrowDown');await turn();
        assert(await root.evaluate(el=>el.matches(':popover-open'))&&await item(root,'Source').evaluate(el=>el===document.activeElement),`${mode}: actual first tree focus`);
        if(columns)await page.keyboard.press('ArrowRight');else await page.keyboard.press('ArrowDown');await turn();
        assert(await item(root,'Main file').evaluate(el=>el===document.activeElement),`${mode}: child navigation`);
        await page.keyboard.press('ArrowDown');await turn();
        assert(await item(root,'Library file').evaluate(el=>el===document.activeElement),`${mode}: disabled sibling not skipped`);
        if(columns) {
          const boxes=await Promise.all(['Source','Library file'].map(name=>item(root,name).boundingBox()));
          assert(boxes[1].x>boxes[0].x+150,`${mode}: columns only claimed, not laid out`);
        }
        await page.keyboard.press('Space');await turn();
        assert((await current()).includes('source/lib')&&JSON.stringify(await form())==='["source/lib"]',`${mode}: full path/FormData mismatch`);
        assert(await main().evaluate(el=>el===document.activeElement)&&await main().getAttribute('aria-expanded')==='false',`${mode}: selection did not close/restore focus`);
        await main().click();await turn();await page.keyboard.press('Tab');await turn();
        assert(await main().getAttribute('aria-expanded')==='false'&&await root.evaluate(el=>!el.contains(document.activeElement)),`${mode}: Tab trapped focus or retained popup`);
        await page.getByRole('checkbox',{name:'Refuse path selection',exact:true}).check();await main().click();await turn();
        const retained=await item(root,'Main file').elementHandle();await item(root,'Main file').click();await turn();
        assert((await current()).includes('source/lib')&&(await page.locator('.path-choice-proposals').textContent()).includes('2'),`${mode}: controlled refusal lost`);
        await page.getByRole('checkbox',{name:'Refuse path selection',exact:true}).uncheck();
        await page.getByRole('button',{name:'Reorder path items',exact:true}).click();await turn();
        assert(await retained.evaluate(el=>el.isConnected),`${mode}: source reorder replaced retained key`);
        await main().click();await turn();await root.getByRole('button',{name:'Toggle Remote files',exact:true}).click();await turn();
        assert((await page.locator('.path-choice-loads').textContent()).includes('1')&&await item(root,'Remote files').getAttribute('aria-expanded')==='true',`${mode}: lazy request`);
        await page.getByRole('button',{name:'Publish remote children',exact:true}).click();await turn();await main().click();await turn();
        await item(root,'Loaded child').click();await turn();assert((await current()).includes('remote/loaded'),`${mode}: explicit lazy publication`);
        const multi=page.getByRole('combobox',{name:'Checked destinations',exact:true}),checks=await popup(multi);
        await multi.click();await turn();await item(checks,'Main file').click();await turn();
        assert(await item(checks,'Source').getAttribute('aria-checked')==='mixed'&&await item(checks,'Source').getAttribute('aria-selected')===null,`${mode}: half-checked semantics`);
        await item(checks,'Source').click();await turn();assert(await item(checks,'Main file').getAttribute('aria-checked')==='true'&&await item(checks,'Library file').getAttribute('aria-checked')==='true'&&await item(checks,'Disabled file').getAttribute('aria-checked')==='false',`${mode}: loaded enabled leaf policy`);
        await page.keyboard.press('Escape');await turn();assert(await multi.evaluate(el=>el===document.activeElement),`${mode}: Escape return focus`);
        const rtl=page.getByRole('combobox',{name:'RTL destination',exact:true}),rtlTree=await popup(rtl);
        await rtl.click();await turn();await page.keyboard.press('ArrowLeft');await turn();await page.keyboard.press('ArrowLeft');await turn();
        assert(await item(rtlTree,'Main file').evaluate(el=>el===document.activeElement),`${mode}: RTL expansion/child`);
        await page.keyboard.press('Escape');await turn();
        await page.getByRole('checkbox',{name:'Enable path choice',exact:true}).uncheck();await turn();assert(await main().isDisabled(),`${mode}: disabled trigger`);
        await page.getByRole('checkbox',{name:'Enable path choice',exact:true}).check();await page.getByRole('checkbox',{name:'Readonly path choice',exact:true}).check();await turn();
        await main().click();await turn();assert(await main().getAttribute('aria-expanded')==='false'&&await main().getAttribute('aria-readonly')==='true',`${mode}: readonly changed open state`);
        await page.getByRole('button',{name:'Select library externally',exact:true}).click();await turn();assert((await current()).includes('source/lib'),`${mode}: readonly blocked external update`);
        await page.getByRole('checkbox',{name:'Readonly path choice',exact:true}).uncheck();await page.getByRole('button',{name:'Clear tree selection',exact:true}).click();await turn();assert((await form()).length===0,`${mode}: clear retained successful proxy`);
        await main().click();await turn();const cdp=await context.newCDPSession(page);const ax=await cdp.send('Accessibility.getFullAXTree');await cdp.detach();
        assert(ax.nodes.some(node=>!node.ignored&&node.role?.value==='tree')&&ax.nodes.some(node=>!node.ignored&&node.role?.value==='treeitem'&&node.name?.value==='Source'),`${mode}: actual AX Tree missing`);
        await page.keyboard.press('Escape');await turn();await page.setViewportSize({width:390,height:900});await main().click();await turn();
        assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${mode}: page horizontal overflow`);
        assert(await root.evaluate(el=>{const r=el.getBoundingClientRect();return r.left>=0&&r.right<=innerWidth+1}),`${mode}: popup clipped outside viewport`);
        await page.screenshot({path:`D:/BuildCache/sapid/${slug}-${hue}-${scheme}-mobile.png`,fullPage:true});
        await page.keyboard.press('Escape');await turn();await page.getByRole('button',{name:'Toggle path Owner',exact:true}).click();await turn();
        assert(await page.getByRole('combobox',{name:'Project destination',exact:true}).count()===0,`${mode}: Owner retained trigger`);
        await page.getByRole('button',{name:'Toggle path Owner',exact:true}).click();await turn();assert(await retained.evaluate(el=>!el.isConnected),`${mode}: retired identity reused`);
        reports.push({mode,realTreePopup:true,actualKeyboardFocus:true,disabledSkip:true,fullPathFormData:true,controlledRefusal:true,stableKeys:true,lazyPublication:true,parentHalfChecked:true,rtl:true,readonly:true,realAX:true,mobileContained:true,ownerRetirement:true,columns});
      }
      if(columns) {
        await page.goto(`${base}/cascader.html?theme=green&scheme=dark`);await page.waitForFunction(()=>window.ready===true);await main().click();await turn();
        const root=await popup(main());await root.getByRole('button',{name:'Toggle Documents',exact:true}).click();await turn();await root.getByRole('button',{name:'Toggle Guide',exact:true}).click();await turn();
        await item(root,'Deep guide').focus();await page.keyboard.press('Home');await turn();
        assert(await root.evaluate(el=>el.scrollWidth>el.clientWidth&&el.scrollLeft>0), 'Cascader deep column was not horizontally revealed');
        assert(await item(root,'Deep guide').evaluate(el=>{const r=el.getBoundingClientRect(),p=el.closest('[role=tree]').getBoundingClientRect();return r.left>=p.left-1&&r.right<=p.right+1}), 'Cascader focused deep column remains clipped');
        await page.keyboard.press('Space');await turn();assert((await current()).includes('docs/guide/deep'),'Cascader lost complete deep path');
        reports.push({mode:'cascader-three-columns-mobile',physicalColumns:true,actualHorizontalReveal:true,completeDeepPath:true});
      }
      await page.goto(`${base}/${slug}-hydrate-blue-light.html?manual=1`);await page.waitForFunction(()=>window.moduleReady===true);
      const trigger=await main().elementHandle();await main().focus();await page.evaluate(mount=>demo[mount](true,'blue','light'),mount);await turn();
      assert(await trigger.evaluate(el=>el.isConnected&&el===document.activeElement),`${slug}: dirty adoption replaced focus/identity`);
      await page.keyboard.press('ArrowDown');await turn();assert(await (await popup(main())).evaluate(el=>el.contains(document.activeElement)),`${slug}: adopted keyboard inactive`);
      reports.push({mode:`${slug}-focused-adoption`,actualFocusedIdentity:true,adoptedKeyboard:true});
      const touch=await browser.newContext({viewport:{width:390,height:900},isMobile:true,hasTouch:true});
      try {const mobile=await touch.newPage();mobile.on('pageerror',e=>errors.push(e.message));await mobile.goto(`${base}/${slug}.html?theme=green&scheme=dark`);await mobile.waitForFunction(()=>window.ready===true);
        await mobile.getByRole('combobox',{name:'Project destination',exact:true}).tap();await mobile.getByRole('treeitem',{name:'Main file',exact:true}).tap();
        await mobile.waitForFunction(()=>document.querySelector('.path-choice-value').textContent.includes('source/main'));
        reports.push({mode:`${slug}-touch`,realPointerActivation:true,fullPath:true});
      } finally {await touch.close();}
    }
    assert(errors.length===0,`Tree choices errors: ${errors.join(';')}`);return{reports,errors};
  } finally {await context.close();}
}
