async(hostPage,baseURL='http://127.0.0.1:8794')=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1200,height:1000}}),reports=[],errors=[];
  const assert=(value,message)=>{if(!value)throw new Error(message)};
  try{
    const page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));page.on('console',event=>{if(event.type()==='error')errors.push(event.text())});
    const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const mode of ['csr','hydrate']){
      await page.goto(`${baseURL}/${mode==='csr'?'messages':`messages-hydrate-${hue}-${scheme}`}.html?theme=${hue}&scheme=${scheme}`);await page.waitForFunction(()=>window.ready===true);await tick();
      const answer=page.locator('#sapid-message-6_answer');assert(await answer.getAttribute('role')==='group','message has its actual named group role');
      assert(await answer.getByRole('blockquote').count()===1,'quote is a real blockquote');
      assert(await answer.getByRole('link',{name:'Original question',exact:true}).getAttribute('href')==='#sapid-message-8_question','citation names the stable message target');
      await answer.locator('.sapid-bubble-body').evaluate(element=>window.messageBody=element);
      await page.getByRole('button',{name:'Complete message',exact:true}).click();await tick();
      assert(await answer.locator('.sapid-bubble-body').evaluate(element=>element===window.messageBody&&element.textContent==='Completed answer 🙂'),'accepted update keeps the actual message body node');
      assert(await answer.locator('.sapid-message-status').textContent()==='Sent','caller status is displayed');
      assert(await page.locator('.messages-unstyled').evaluate(element=>getComputedStyle(element).borderTopWidth)==='0px','unstyled bubble removes cosmetic border');
      assert(await page.locator('.messages-unstyled').textContent()==='<script>display only</script>','content is literal text');
      await page.getByRole('button',{name:'Application retry',exact:true}).click();await tick();assert(await page.locator('.messages-action-count').textContent()==='Retries 1','only explicit application action runs');
      reports.push(`messages-${hue}-${scheme}-${mode}`);
    }
    await page.getByRole('button',{name:'Group middle',exact:true}).click();await tick();assert(!await page.locator('#sapid-message-6_answer .sapid-message-header').isVisible()&&!await page.locator('#sapid-message-6_answer .sapid-message-footer').isVisible(),'middle grouping hides metadata');reports.push('messages-grouping');
    await page.getByRole('button',{name:'Group single',exact:true}).click();await tick();assert(await page.locator('#sapid-message-6_answer .sapid-message-footer').isVisible(),'single grouping restores metadata');reports.push('messages-regrouping');
    await page.getByRole('link',{name:'Jump to question',exact:true}).click();await tick();assert(new URL(page.url()).hash==='#sapid-message-8_question','actual marker follows its native fragment destination');reports.push('messages-marker-navigation');
    await page.getByRole('button',{name:'Remove quote',exact:true}).click();await tick();assert(!await page.locator('#sapid-message-6_answer').getByRole('blockquote').isVisible(),'removed quote is hidden from interaction and accessibility');reports.push('messages-quote-removal');
    return{reports,errors};
  }finally{await context.close()}
}
