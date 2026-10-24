(()=>{
  'use strict';
  const data=JSON.parse(document.querySelector('#gallery-data').textContent),byId=new Map(data.map(row=>[row.id,row]));
  const element=id=>document.getElementById(`gallery-${id}`),params=new URLSearchParams(location.search);
  const search=element('search'),phase=element('phase'),status=element('status'),hue=element('hue'),scheme=element('scheme'),example=element('example'),render=element('render'),preview=element('preview');
  let selected=null,currentURL=null,desiredURL=null,loading=false,watch=null;
  const navigate=()=>{
    if(loading||desiredURL===currentURL)return;
    currentURL=desiredURL;
    if(currentURL===null){preview.hidden=true;preview.removeAttribute('src');preview.removeAttribute('inert');preview.setAttribute('aria-busy','false');return;}
    loading=true;preview.hidden=false;preview.setAttribute('inert','');preview.setAttribute('aria-busy','true');preview.src=currentURL;
    const deadline=performance.now()+30000,url=currentURL;
    const check=()=>{
      watch=null;let ready=false;
      try{ready=preview.contentWindow?.location.href===url&&preview.contentWindow.ready===true}catch(_){/* only the declared same-origin fixture is accepted */}
      if(!ready&&performance.now()<deadline){watch=requestAnimationFrame(check);return;}
      loading=false;
      if(desiredURL!==currentURL){navigate();return;}
      preview.setAttribute('aria-busy','false');preview.removeAttribute('inert');preview.hidden=!ready;
      if(!ready)element('preview-note').textContent='Preview did not become ready within 30 seconds. Open the example for its diagnostics.';
    };
    watch=requestAnimationFrame(check);
  };
  const requestPreview=url=>{desiredURL=url;preview.style.visibility=url===null?'hidden':'';if(url===null&&!loading&&currentURL===null){preview.hidden=true;preview.setAttribute('aria-busy','false')}navigate()};
  for(const value of [...new Set(data.map(row=>row.phase))])phase.add(new Option(value,value));
  hue.value=params.get('theme')==='green'?'green':'blue';scheme.value=params.get('scheme')==='dark'?'dark':'light';
  const theme=()=>{document.documentElement.dataset.hue=hue.value;document.documentElement.dataset.mode=scheme.value;const url=new URL(location.href);url.searchParams.set('theme',hue.value);url.searchParams.set('scheme',scheme.value);if(selected)url.hash=selected.id;history.replaceState(null,'',url)};
  const updatePreview=()=>{
    if(!selected)return;
    const entry=selected.pages.find(page=>page.name===example.value),link=element('open');
    const hydration=render.querySelector('[value=hydrate]');hydration.disabled=!entry?.hydrate;if(hydration.disabled)render.value='csr';
    if(!entry){requestPreview(null);link.removeAttribute('href');element('preview-note').textContent='No interactive fixture is declared for this item.';return;}
    const stem=render.value==='hydrate'?(entry.name==='index'?'hydrate':`${entry.name}-hydrate`)+`-${hue.value}-${scheme.value}`:entry.name;
    const url=new URL(`${stem}.html`,location.href);url.searchParams.set('theme',hue.value);url.searchParams.set('scheme',scheme.value);
    requestPreview(url.href);link.href=url.href;element('preview-note').textContent=selected.capabilities.web.state==='tested'?'One live example is mounted. Changing palette or render mode remounts that example.':'This fixture is under development; its presence does not establish browser acceptance.';
  };
  const select=id=>{
    const row=byId.get(id);if(!row)return;selected=row;
    for(const current of document.querySelectorAll('tbody tr'))current.setAttribute('aria-current',String(current.dataset.id===id));
    element('title').textContent=`${row.id} · ${row.name}`;element('work').textContent=row.work;
    const capabilities=element('capabilities');capabilities.replaceChildren();
    for(const [axis,cell] of Object.entries(row.capabilities)){const term=document.createElement('dt'),description=document.createElement('dd');term.textContent=`${axis}: ${cell.state}`;description.textContent=cell.notes;capabilities.append(term,description);}
    example.replaceChildren(...row.pages.map(page=>new Option(page.name,page.name)));
    const evidence=element('evidence');evidence.replaceChildren();
    for(const [category,values] of Object.entries(row.evidence)){const title=document.createElement('strong'),list=document.createElement('ul');title.textContent=category;for(const value of values){const item=document.createElement('li');item.textContent=value;list.append(item)}evidence.append(title,list)}
    if(!evidence.childNodes.length)evidence.textContent='No canonical acceptance evidence recorded yet.';
    theme();updatePreview();
  };
  const filter=()=>{const query=search.value.toLocaleLowerCase().trim();let shown=0;for(const row of data){const visible=(!phase.value||row.phase===phase.value)&&(!status.value||row.status===status.value)&&(!query||`${row.id} ${row.name}`.toLocaleLowerCase().includes(query));document.querySelector(`tr[data-id="${row.id}"]`).hidden=!visible;shown+=Number(visible)}element('count').textContent=`${shown} of ${data.length} declared work items`;};
  search.addEventListener('input',filter);phase.addEventListener('change',filter);status.addEventListener('change',filter);
  for(const control of [hue,scheme])control.addEventListener('change',()=>{theme();updatePreview()});
  for(const control of [example,render])control.addEventListener('change',updatePreview);
  document.querySelector('tbody').addEventListener('click',event=>{const button=event.target.closest('button[data-select]');if(button)select(button.dataset.select)});
  window.addEventListener('hashchange',()=>select(location.hash.slice(1)));
  window.addEventListener('pagehide',()=>{if(watch!==null)cancelAnimationFrame(watch)});
  filter();select(byId.has(location.hash.slice(1))?location.hash.slice(1):data[0].id);window.galleryReady=true;
})();
