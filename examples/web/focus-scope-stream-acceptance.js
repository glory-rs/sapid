async(hostPage,baseURL='http://127.0.0.1:8794')=>{
  const context=await hostPage.context().browser().newContext(),reports=[],errors=[];
  try{
    const page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));
    await page.goto(`${baseURL}/focus-stream.html`);
    await page.waitForFunction(()=>document.querySelector('#focus-stream-root')?.dataset.glyStreamState==='ready');
    await page.evaluate(()=>{
      const root=document.querySelector('#stream-focus-scope');
      for(const field of ['scope','loop','contain','auto','restore'])if(root.getAttribute(`data-gly-focus-${field}`)!=='true')throw Error(`stream focus field ${field}`);
      if(root.getAttribute('data-gly-focus-initial')!=='stream-initial'||root.getAttribute('data-gly-focus-return')!=='stream-return')throw Error('stream focus relations');
      const outside=document.createElement('button');outside.textContent='Before WASM';document.body.append(outside);outside.focus();
      if(document.activeElement!==outside)throw Error('classic metadata must not impersonate the live input driver');
      if(GloryStream.active()!==0)throw Error('completed stream controller retained');
    });reports.push({suite:'focus-scope-classic-policy-and-End',status:'passed'});
    const source=await page.evaluate(async()=>await(await fetch('focus-stream.html')).text());
    for(const kind of ['boolean','relation','old-version']){
      const result=await page.evaluate(({source,kind})=>{
        const parsed=new DOMParser().parseFromString(source,'text/html');
        const data=JSON.parse(parsed.querySelector('script[data-gly-stream]').textContent);
        const root=parsed.querySelector('#focus-stream-root').cloneNode(true);root.id=`focus-probe-${kind}`;document.body.append(root);
        const input=root.querySelector('input');input.value='Dirty before failed start';input.setSelectionRange(2,7);
        const before=root.innerHTML;
        const piece=data.nodes.flatMap(node=>node.shape.pieces).find(piece=>piece.attributes?.['data-gly-focus-scope']!==undefined);
        if(kind==='boolean')piece.attributes['data-gly-focus-loop']='perhaps';
        else if(kind==='relation')piece.attributes['data-gly-focus-return']='not a semantic token';
        else data.html_version=12;
        data.data_id=`focus-probe-data-${kind}`;
        let failure='';try{GloryStream.start(root,data)}catch(error){failure=String(error)}
        const good=!!failure&&root.innerHTML===before&&input.value==='Dirty before failed start'&&input.selectionStart===2&&input.selectionEnd===7&&GloryStream.active()===0;
        root.remove();return{good,failure};
      },{source,kind});
      if(!result.good)throw Error(`failed ${kind} start changed its host or retained a controller: ${result.failure}`);
      reports.push({suite:`focus-scope-classic-refuses-${kind}`,status:'passed'});
    }
  }catch(error){errors.push(error.message)}finally{await context.close()}
  return{reports,errors};
}
