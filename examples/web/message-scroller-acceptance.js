async(hostPage,baseURL='http://127.0.0.1:8794')=>{
  const context=await hostPage.context().browser().newContext({viewport:{width:1200,height:1000}}),reports=[],errors=[];
  const assert=(value,message)=>{if(!value)throw Error(message)};
  try{
    const page=await context.newPage();page.on('pageerror',error=>errors.push(error.message));page.on('console',event=>{if(event.type()==='error')errors.push(event.text())});
    const tick=()=>page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
    const viewport=()=>page.locator('.sapid-message-scroller [data-gly-virtual="root"]');
    const tail=async()=>{await page.waitForFunction(()=>document.querySelector('.message-scroller-status')?.textContent==='following true; unread 0');await tick();await tick()};
    for(const hue of ['blue','green'])for(const scheme of ['light','dark'])for(const mode of ['csr','hydrate']){
      await page.goto(`${baseURL}/${mode==='csr'?'message-scroller':`message-scroller-hydrate-${hue}-${scheme}`}.html?theme=${hue}&scheme=${scheme}`);await page.waitForFunction(()=>window.ready===true);await tail();
      const stats=await page.locator('.message-scroller-stats').textContent(),rows=Number(stats.match(/rows (\d+)/)[1]),reads=Number(stats.match(/reads (\d+)/)[1]);
      assert(rows<=64&&reads<1000,'million-message source reads and mounted rows stay bounded');
      assert(await page.locator('#sapid-message-11_row-1000999').count()===1,'initial following mounts the latest keyed message');
      const box=await viewport().boundingBox();assert(box.height>=319&&box.height<=321,'the window has a real constrained viewport');
      reports.push(`message-scroller-${hue}-${scheme}-${mode}`);
    }
    await viewport().hover();await page.mouse.wheel(0,-600);await page.waitForFunction(()=>document.querySelector('.message-scroller-status').textContent.startsWith('following false;'));await tick();
    await page.getByRole('button',{name:'Append five',exact:true}).click();await page.getByRole('button',{name:'Append five',exact:true}).click();await tick();
    assert(await page.locator('.message-scroller-status').textContent()==='following false; unread 10','unread counts accepted appends without pulling the reader to the tail');reports.push('message-scroller-unread');
    const anchor=await viewport().evaluate(root=>{const bounds=root.getBoundingClientRect();const row=[...root.querySelectorAll('.sapid-message')].find(row=>{const b=row.getBoundingClientRect();return b.top<=bounds.top&&b.bottom>bounds.top});if(!row)throw Error('no visible anchor');return{id:row.id,y:row.getBoundingClientRect().top}});
    await page.getByRole('button',{name:'Prepend ten',exact:true}).click();await tick();await tick();
    assert(await page.locator(`#${anchor.id}`).evaluate((row,y)=>Math.abs(row.getBoundingClientRect().top-y)<2,anchor.y),'prepend preserves the actual keyed reading position');
    assert(await page.locator('.message-scroller-status').textContent()==='following false; unread 10','history prepend creates no unread messages');reports.push('message-scroller-prepend-anchor');
    await page.getByRole('button',{name:'Change message heights',exact:true}).click();await tick();await tick();
    assert(await page.locator(`#${anchor.id}`).evaluate((row,y)=>Math.abs(row.getBoundingClientRect().top-y)<2,anchor.y),'measured height changes preserve the same visible reading anchor');reports.push('message-scroller-height-anchor');
    await page.getByRole('button',{name:'Follow latest',exact:true}).click();await tail();
    assert(await page.locator('#sapid-message-11_row-1001009').count()===1,'explicit follow mounts the final appended message and clears unread');
    await page.getByRole('button',{name:'Append five',exact:true}).click();await tail();
    assert(await page.locator('#sapid-message-11_row-1001014').count()===1,'following survives later append and content height measurements');reports.push('message-scroller-follow-and-append');
    return{reports,errors};
  }finally{await context.close()}
}

