async function(page,base='http://127.0.0.1:8794') {
  const reports=[],errors=[];page.on('pageerror',error=>errors.push(String(error)));
  const assert=(value,message)=>{if(!value)throw Error(message)};
  const turn=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
  const root=()=>page.locator('.sapid-json-viewer:not(.json-tree-demo)');
  const branch=()=>root().getByRole('button',{name:'branch: Object 2',exact:true});
  const list=()=>root().getByRole('button',{name:'list: Array 2',exact:true});
  const values=()=>root().locator('.sapid-json-value').allTextContents();
  for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]) {
    const mode=`json-viewer-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;
    await page.setViewportSize({width:1100,height:900});await page.goto(`${base}/${hydrate?`code-hydrate-${hue}-${scheme}.html`:`code.html?theme=${hue}&scheme=${scheme}`}`);await page.waitForFunction(()=>window.ready===true);
    assert(JSON.stringify(await values())===JSON.stringify(['version: 1']),`${mode}: closed descendant mounted`);
    await branch().focus();await page.keyboard.press('Space');await turn();
    assert(await branch().getAttribute('aria-expanded')==='true',`${mode}: keyboard expansion`);
    assert((await values()).includes('needle: "😀 <script>"'),`${mode}: escaped leaf text missing`);
    assert(await root().locator('script').count()===0,`${mode}: source interpreted`);
    await list().click();await turn();assert((await values()).includes('0: 1')&&(await values()).includes('1: 2'),`${mode}: array indices/values missing`);
    const retained=await branch().elementHandle(),version=await root().locator('.sapid-json-value').filter({hasText:'version: 1'}).elementHandle();
    await page.getByRole('button',{name:'Reorder JSON properties',exact:true}).click();await turn();
    assert(await retained.evaluate(el=>el.isConnected&&el.textContent==='branch: Object 2'&&el===[...document.querySelector('.sapid-json-viewer:not(.json-tree-demo)').querySelectorAll('button')].find(button=>button.textContent==='branch: Object 2')),`${mode}: retained container retired`);
    assert(await version.evaluate(el=>el.isConnected&&el.textContent==='version: 2'),`${mode}: stable path identity lost on reorder`);
    await page.getByRole('button',{name:'Reject or accept JSON expansion',exact:true}).click();await branch().click();await turn();assert(await branch().getAttribute('aria-expanded')==='true',`${mode}: controlled rejected close applied`);
    await page.getByRole('button',{name:'Reject or accept JSON expansion',exact:true}).click();await branch().click();await turn();assert(await branch().getAttribute('aria-expanded')==='false'&&!await root().locator('.sapid-json-value').filter({hasText:'needle:'}).count(),`${mode}: close kept child Owner`);
    const query=page.getByRole('textbox',{name:'Search JSON structure',exact:true});await query.fill('needle');await turn();assert((await values()).includes('needle: "😀 <script>"'),`${mode}: search did not reveal ancestors`);
    assert(await branch().isDisabled(),`${mode}: forced search expansion can be contradicted`);const highlighted=await root().locator('.sapid-json-value').filter({hasText:'needle:'}).evaluate(el=>getComputedStyle(el).backgroundColor);assert(highlighted!=='rgba(0, 0, 0, 0)',`${mode}: match highlight absent`);
    await query.fill('');await turn();assert(await branch().getAttribute('aria-expanded')==='false',`${mode}: search mutated accepted expansion`);
    const session=await page.context().newCDPSession(page);const ax=await session.send('Accessibility.getFullAXTree');const documentNode=await session.send('DOM.getDocument');const domRoot=await session.send('DOM.querySelector',{nodeId:documentNode.root.nodeId,selector:'.sapid-json-viewer:not(.json-tree-demo)'});const described=await session.send('DOM.describeNode',{nodeId:domRoot.nodeId});const axRoot=ax.nodes.find(node=>node.backendDOMNodeId===described.node.backendNodeId);assert(axRoot,`${mode}: document viewer missing from AX`);const byId=new Map(ax.nodes.map(node=>[node.nodeId,node]));const own=[];const pending=[axRoot];while(pending.length){const node=pending.pop();own.push(node);for(const id of node.childIds||[]){const child=byId.get(id);if(child)pending.push(child)}}await session.detach();assert(own.some(n=>!n.ignored&&n.role?.value==='list'),`${mode}: missing real document list`);assert(!own.some(n=>!n.ignored&&['tree','treeitem'].includes(n.role?.value)),`${mode}: invented interactive tree roles`);
    await page.setViewportSize({width:390,height:900});await turn();assert(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1),`${mode}: mobile overflow`);
    await page.evaluate(()=>demo.unmount_display());assert(await page.locator('.sapid-json-node').count()===0,`${mode}: Owner leaked JSON nodes`);
    reports.push({mode,nestedLists:true,keyboardExpansion:true,controlledRejection:true,searchRevealWithoutSourceWrite:true,escapedValues:true,stablePathIdentity:true,unmountedClosedOwners:true,realAX:true,mobileContained:true});
  }
  await page.goto(`${base}/code-hydrate-blue-light.html?manual=1`);await page.waitForFunction(()=>window.moduleReady===true);const item=await branch().elementHandle();await branch().focus();await page.evaluate(()=>demo.mount_code(true,'blue','light'));await turn();assert(await item.evaluate(el=>el.isConnected&&el===document.activeElement),'JSON adoption replaced focused disclosure');await page.keyboard.press('Space');await turn();assert(await branch().getAttribute('aria-expanded')==='true','JSON adopted disclosure inert');await page.evaluate(()=>demo.unmount_display());reports.push({mode:'json-viewer-focused-adoption',focusedButtonIdentity:true,keyboardActivation:true});assert(errors.length===0,`JSON viewer errors ${errors.join(';')}`);return{reports,errors};
}
