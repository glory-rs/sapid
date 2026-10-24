async(hostPage)=>{
  const context=await hostPage.context().browser().newContext();const errors=[];
  try{
    const page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));
    await page.goto('http://127.0.0.1:8794/document-stream.html');
    await page.waitForFunction(()=>document.querySelector('#document-stream-root')?.dataset.glyStreamState==='ready');
    for(const tag of['code','kbd','mark','blockquote','ol','ul','li','dl','dt','dd']){
      if(await page.locator(`.stream-document ${tag}`).count()===0)throw new Error(`stream document tag ${tag} absent`);
    }
    const state=await page.evaluate(()=>({active:GloryStream.active(),pending:document.querySelector('#document-stream-root').hasAttribute('data-gly-stream-pending'),resources:JSON.parse(document.querySelector('#document-stream-data').textContent),text:document.querySelector('code').textContent}));
    if(state.active!==0||state.pending||state.text!=='code <&>'||state.resources.records[0]?.name!=='document'||errors.length)throw new Error(JSON.stringify({state,errors}));
    return{reports:[{suite:'document-classic-stream',status:'passed',checks:'real finite document tags in Rust-generated Patch, classic receiver, End rekey/resource handoff, no live controller retention'}],errors};
  }finally{await context.close();}
}
