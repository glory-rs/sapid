async function(page,base='http://127.0.0.1:8794'){
 const reports=[],errors=[];const assert=(ok,message)=>{if(!ok)throw Error(message)};page.on('pageerror',e=>errors.push(String(e)));await page.context().grantPermissions(['clipboard-read','clipboard-write'],{origin:new URL(base).origin});const turn=()=>page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
 const read=()=>page.evaluate(async()=>(await navigator.clipboard.readText()).replace(/\r\n/g,'\n'));
 const rust="pub fn greet<'a>(name: &'a str) -> String {\n    // Unicode and HTML remain plain code: 😀 <script>\n    let message = format!(\"Hello, {name}!\");\n    message\n}\n";
 for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const hydrate of [false,true]){
 const mode=`code-json-copy-${hue}-${scheme}-${hydrate?'hydrate':'csr'}`;await page.setViewportSize({width:1100,height:900});await page.goto(`${base}/${hydrate?`code-hydrate-${hue}-${scheme}.html`:`code.html?theme=${hue}&scheme=${scheme}`}`);await page.waitForFunction(()=>window.ready===true);
 await page.locator('.code-copy').click();await page.waitForFunction(()=>document.querySelector('.code-copy').textContent==='Copied');assert(await read()===rust,`${mode}: code copied tokens or gutter instead of exact source`);await page.getByRole('button',{name:'Show JSON code',exact:true}).click();await turn();assert(await page.locator('.code-copy').textContent()==='Copy highlighted code',`${mode}: source change retained stale copy feedback`);await page.locator('.code-copy').click();await page.waitForFunction(()=>document.querySelector('.code-copy').textContent==='Copied');assert(await read()===await page.locator('.code-demo-highlight .sapid-code-source').textContent(),`${mode}: copy after language/source change stale`);
 await page.locator('.json-copy').click();await page.waitForFunction(()=>document.querySelector('.json-copy').textContent==='Copied');assert(await read()==='{"branch":{"needle":"😀 <script>","list":[1,2]},"version":1}',`${mode}: JSON copied only visible disclosure content`);
 await page.evaluate(()=>demo.unmount_display());reports.push({mode,exactCodeWithoutGutter:true,currentAcceptedSource:true,completeCollapsedJSON:true,realPlatformRoundtrip:true,ownerCleanup:true});
 }
 assert(errors.length===0,errors.join(';'));return{reports,errors};
}
