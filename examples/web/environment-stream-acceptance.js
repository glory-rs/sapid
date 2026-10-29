async(hostPage,baseURL='http://127.0.0.1:8794')=>{
  const context=await hostPage.context().browser().newContext(),reports=[],errors=[];
  try{
    const page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));
    await page.goto(`${baseURL}/environment-stream.html`);
    await page.waitForFunction(()=>document.querySelector('#environment-stream-root')?.dataset.glyStreamState==='ready');
    await page.evaluate(()=>{
      const root=document.querySelector('#environment-stream-root'),environment=root.querySelector('[lang][dir]');
      if(environment.getAttribute('lang')!=='ar'||environment.getAttribute('dir')!=='rtl')throw Error('classic environment finite patches');
      if(root.firstChild.nodeValue!=='gly:v:14'||GloryStream.active()!==0)throw Error('environment End/version/controller release');
    });reports.push({suite:'environment-classic-fields-and-End',status:'passed'});
    const source=await page.evaluate(async()=>await(await fetch('environment-stream.html')).text());
    for(const kind of ['direction','language','language-budget','old-version']){
      const good=await page.evaluate(({source,kind})=>{
        const parsed=new DOMParser().parseFromString(source,'text/html'),data=JSON.parse(parsed.querySelector('script[data-gly-stream]').textContent);
        const root=parsed.querySelector('#environment-stream-root').cloneNode(true);root.id=`environment-probe-${kind}`;document.body.append(root);
        const input=root.querySelector('input');input.value='Dirty 🙂';input.setSelectionRange(2,5);const before=root.innerHTML;
        const piece=data.nodes.flatMap(node=>node.shape.pieces).find(piece=>piece.attributes?.lang!==undefined);
        if(kind==='direction')piece.attributes.dir='auto';else if(kind==='language')piece.attributes.lang='bad language';else if(kind==='language-budget')piece.attributes.lang='en-'+Array(30).fill('abcd').join('-');else data.html_version=12;
        data.data_id=`environment-probe-data-${kind}`;
        let failure='';try{GloryStream.start(root,data)}catch(error){failure=String(error)}
        const result=!!failure&&root.innerHTML===before&&input.value==='Dirty 🙂'&&input.selectionStart===2&&input.selectionEnd===5&&GloryStream.active()===0;root.remove();return result;
      },{source,kind});
      if(!good)throw Error(`environment ${kind} refusal changed its host`);
      reports.push({suite:`environment-classic-refuses-${kind}`,status:'passed'});
    }
  }catch(error){errors.push(error.message)}finally{await context.close()}
  return{reports,errors};
}
