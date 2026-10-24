(() => {
  const root=document.documentElement, params=new URLSearchParams(location.search);
  root.dataset.hue=params.get('theme')==='green'||location.pathname.includes('green')?'green':'blue';
  root.dataset.mode=params.get('scheme')==='dark'||location.pathname.includes('dark')?'dark':'light';
  window.setupShowcase=page=>{
    const sidebar=document.querySelector('.sidebar');
    if(sidebar&&!sidebar.querySelector('[data-page="gallery"]')){const gallery=document.createElement('a');gallery.href='gallery.html';gallery.dataset.page='gallery';gallery.textContent='All components · capability gallery';sidebar.append(gallery);}
    document.querySelectorAll(`[data-page="${page}"]`).forEach(e=>e.classList.add('active'));
    const go=(hue,mode)=>{
      const hydrate=location.pathname.includes('hydrate');
      const family=page==='basic'?'index':page;
      const stem=family==='index'?'hydrate':`${family}${location.pathname.includes('initial')?'-initial':''}-hydrate`;
      const target=hydrate?`${stem}-${hue}-${mode}.html`:`${family}.html?theme=${hue}&scheme=${mode}`;
      location.href=target;
    };
    document.querySelectorAll('a[href]').forEach(link=>{
      const href=link.getAttribute('href');
      if(href.startsWith('#')||!href.includes('.html'))return;
      const target=new URL(href,location.href);
      if(target.origin!==location.origin)return;
      if(target.pathname.includes('hydrate'))target.pathname=target.pathname.replace(/(?:-blue|-green)?(?:-light|-dark)?\.html$/,`-${root.dataset.hue}-${root.dataset.mode}.html`);
      else {target.searchParams.set('theme',root.dataset.hue);target.searchParams.set('scheme',root.dataset.mode);}
      link.href=target.href;
    });
    document.querySelectorAll('[data-hue-button]').forEach(button=>{
      button.setAttribute('aria-pressed',String(button.dataset.hueButton===root.dataset.hue));
      button.onclick=()=>go(button.dataset.hueButton,root.dataset.mode);
    });
    document.getElementById('mode-toggle').onclick=()=>go(root.dataset.hue,root.dataset.mode==='dark'?'light':'dark');
    const input=document.getElementById('nav-search');
    input.addEventListener('input',()=>document.querySelectorAll('.sidebar a,.sidebar .nav-soon').forEach(e=>e.hidden=!e.textContent.toLowerCase().includes(input.value.toLowerCase())));
    document.addEventListener('keydown',event=>{if(event.key==='/'&&!/INPUT|TEXTAREA/.test(document.activeElement.tagName)){event.preventDefault();input.focus();}});
    const colors=root.dataset.hue==='green'?['ebfbee','d3f9d8','b2f2bb','8ce99a','69db7c','51cf66','40c057','37b24d','2f9e44','2b8a3e']:['e7f5ff','d0ebff','a5d8ff','74c0fc','4dabf7','339af0','228be6','1c7ed6','1971c2','1864ab'];
    const palette=document.getElementById('palette');
    if(palette) for(const color of colors){const cell=document.createElement('span');cell.style.background=`#${color}`;cell.title=`#${color}`;palette.append(cell);}
    const caption=document.getElementById('theme-caption');if(caption)caption.textContent=`${root.dataset.hue==='green'?'Green':'Blue'} palette · ${root.dataset.mode} scheme · 10 tonal steps`;
  };
})();
