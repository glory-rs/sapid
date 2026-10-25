const fs=require('node:fs');
process.env.TEMP=process.env.TMP='D:/BuildCache/sapid/tmp';
const {chromium}=require(process.env.SAPID_PLAYWRIGHT||'D:/Works/glory-rs/glory/tests/playwright/node_modules/playwright');
(async()=>{let browser;const report={routes:[],errors:[]};try{
 browser=await chromium.launch({headless:true});const page=await browser.newPage();
 const tasks=JSON.parse(fs.readFileSync('docs/widget-inventory.json','utf8')).tasks;
 for(const task of tasks){
  await page.goto(`http://127.0.0.1:8795/showcase.html?widget=${task.id}&theme=green&scheme=dark`);await page.waitForFunction(()=>window.ready===true);
  const result=await page.evaluate(()=>({background:getComputedStyle(document.querySelector('.sapid-site')).backgroundColor,unexpected:[...document.querySelectorAll('.site-demo .sapid-theme-blue-light:not(.sapid-unstyled)')].filter(e=>e.checkVisibility()).map(e=>({class:e.className,name:e.getAttribute('aria-label'),text:(e.textContent||'').slice(0,90)}))}));
  report.routes.push({id:task.id,...result});if(result.background!=='rgb(20, 21, 23)'||result.unexpected.length)report.errors.push(task.id);
  if(report.routes.length%25===0)console.log(`Dark audit ${report.routes.length}/149, ${report.errors.length} routes need attention`);
 }
}catch(error){report.errors.push(error.stack||String(error))}finally{if(browser)await browser.close();fs.writeFileSync('D:/BuildCache/sapid/dark-theme-audit.json',JSON.stringify(report,null,2));console.log(JSON.stringify({routes:report.routes.length,errors:report.errors}));if(report.errors.length)process.exitCode=1}})();
