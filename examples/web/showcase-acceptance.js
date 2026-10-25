async (hostPage) => {
  const browser = hostPage.context().browser();
  const context = await browser.newContext({viewport:{width:1440,height:1000}});
  const base = process.env.SAPID_SHOWCASE_BASE || 'http://127.0.0.1:8795';
  const reports = [], errors = [];
  let activePage;
  const assert = (value, message) => { if (!value) throw Error(message); };
  const setup = page => {
    page.on('pageerror', e => errors.push(e.message));
    page.on('console', e => { if (e.type() === 'error') errors.push(e.text()); });
  };
  const go = async (page, name) => {
    await page.goto(`${base}/${name}`);
    await page.waitForFunction(() => window.ready === true);
    await page.locator('.sapid-site').waitFor();
    assert(await page.locator('iframe').count() === 0, `${name}: no iframe UI`);
    assert(await page.locator('.site-subnav').count() === 0, `${name}: secondary row removed`);
    assert(await page.locator('.site-header-tools > .site-palettes + .site-theme-toggle').count() === 1, `${name}: palettes directly precede scheme toggle`);

  };
  const shot = (page, name) => page.screenshot({path:`D:/BuildCache/sapid/showcase-${name}.png`,fullPage:true});
  try {
    await context.grantPermissions(['clipboard-read','clipboard-write'],{origin:base});
    const p = await context.newPage(); activePage = p; setup(p);
    await go(p, 'showcase.html');
    assert(await p.getByRole('banner').count() === 1 && await p.getByRole('main').count() === 1, 'home has real landmarks');
    await shot(p, 'home-light');
    reports.push({suite:'showcase-rust-home',status:'passed'});

    await go(p, 'showcase.html?widget=P0-01');
    const routes = await p.locator('.site-nav-link').evaluateAll(elements => elements.map(e => ({href:e.getAttribute('href'),label:e.textContent.trim()})));
    assert(routes.length === 149 && new Set(routes.map(r => new URL(r.href, base).searchParams.get('widget'))).size === 149, '149 unique canonical navigation links');
    for (const route of routes) {
      const id = new URL(route.href, base).searchParams.get('widget');
      const anchor = p.locator(`.site-nav-link[href="${route.href}"]`);
      await anchor.click();
      await p.waitForFunction(id=>document.documentElement.getAttribute('data-sapid-widget')===id,id);
      await p.locator('h1.site-title').waitFor();
      assert(new URL(p.url()).searchParams.get('widget') === id, `${id}: actual route navigation`);
      assert(await p.locator('.site-demo .site-example').count() === 1, `${id}: real embedded Glory widget`);
      assert((await p.locator('.site-code').first().textContent()).trim().length > 30, `${id}: readable source`);
      assert(await p.locator('iframe').count() === 0, `${id}: no fixture iframe`);
      reports.push({suite:`showcase-route-${id}`,status:'passed'});
      console.log(`showcase route ${reports.length - 1}/149 ${id} passed`);
    }

    await go(p, 'showcase.html?widget=P0-01');
    const search = p.getByRole('textbox',{name:'Search widgets',exact:true});
    await search.fill('P0-02');
    await p.waitForFunction(() => document.querySelectorAll('.site-nav-link').length === 1);
    assert((await p.locator('.site-nav-link').getAttribute('href')).includes('P0-02'), 'search filters by canonical ID');
    await search.fill('no-such-widget-xyz');
    await p.locator('.site-nav-empty').first().waitFor();
    assert(await p.locator('.site-nav-link').count() === 0, 'empty search state');
    await search.fill('');
    await p.waitForFunction(() => document.querySelectorAll('.site-nav-link').length === 149);
    reports.push({suite:'showcase-live-search',status:'passed'});

    const demo = p.locator('.site-demo');
    await demo.getByRole('button',{name:'Filled button',exact:true}).click();
    await demo.getByText('Clicked 1 times',{exact:true}).waitFor();
    const preview = demo.locator('.site-playground-button');
    await preview.evaluate(e => window.showcasePreview = e);
    const mediumHeight = (await preview.boundingBox()).height;
    await demo.getByRole('combobox',{name:'Variant',exact:true}).selectOption('outline');
    await demo.getByRole('button',{name:'Outline button',exact:true}).waitFor();
    await demo.getByRole('button',{name:'Outline button',exact:true}).click();
    await demo.getByText('Clicked 2 times',{exact:true}).waitFor();
    await demo.getByRole('combobox',{name:'Size',exact:true}).selectOption('large');
    await p.waitForFunction(height => document.querySelector('.site-playground-button').getBoundingClientRect().height > height, mediumHeight);
    await demo.getByRole('checkbox',{name:'Disabled',exact:true}).check();
    await p.waitForFunction(() => document.querySelector('.site-playground-button').disabled);
    await demo.getByRole('checkbox',{name:'Disabled',exact:true}).uncheck();
    await demo.getByRole('checkbox',{name:'Loading',exact:true}).check();
    await p.waitForFunction(() => {const b=document.querySelector('.site-playground-button'); return b.disabled && b.getAttribute('aria-busy') === 'true';});
    await demo.getByRole('button',{name:'Loading…',exact:true}).waitFor();
    await demo.getByRole('checkbox',{name:'Loading',exact:true}).uncheck();
    await p.waitForFunction(() => !document.querySelector('.site-playground-button').disabled);
    assert(await preview.evaluate(e => e === window.showcasePreview), 'real property controls retain the same preview button');
    assert(await demo.getByText('Clicked 2 times',{exact:true}).count() === 1, 'disabled/loading changes never activate demo');
    const snippet = p.locator('.site-code').first();
    const codeBody = snippet.locator('.site-code-body');
    const collapsedHeight = (await codeBody.boundingBox()).height;
    const completeSource = await codeBody.locator('.sapid-code-source').textContent();
    assert(completeSource.includes('pub fn demo(') && completeSource.includes('on_checked_change'), 'collapsed example keeps complete Rust source');
    assert(completeSource.includes('use glory::style::FlexWrap;'), 'copyable Button snippet includes the explicit layout import');
    await snippet.getByRole('button',{name:'Expand code',exact:true}).click();
    await snippet.getByRole('button',{name:'Collapse code',exact:true}).waitFor();
    await p.waitForFunction(height => document.querySelector('.site-code-body').getBoundingClientRect().height > height, collapsedHeight);
    await snippet.getByRole('button',{name:'Copy code',exact:true}).click();
    await p.waitForFunction(async expected => (await navigator.clipboard.readText()) === expected, completeSource);
    await snippet.getByRole('button',{name:'Collapse code',exact:true}).click();
    await snippet.getByRole('button',{name:'Expand code',exact:true}).waitFor();
    assert(Math.abs((await codeBody.boundingBox()).height - collapsedHeight) <= 1, 'code collapse restores bounded preview height');
    reports.push({suite:'showcase-expand-and-complete-copy',status:'passed'});
    await p.getByRole('button',{name:'Source',exact:true}).click();
    await p.getByRole('heading',{name:'Rust source',exact:true}).waitFor();
    await p.locator('.site-code').waitFor();
    assert(await p.locator('.site-demo').count() === 0, 'Rust Source tab replaces demo content');
    await p.getByRole('button',{name:'Documentation',exact:true}).click();
    await p.locator('.site-demo').waitFor();
    await shot(p, 'button-light');
    reports.push({suite:'showcase-button-and-source',status:'passed'});

    await p.getByRole('link',{name:'Switch to dark theme',exact:true}).click();
    await p.locator('.sapid-site.site-dark').waitFor();
    assert(new URL(p.url()).searchParams.get('widget') === 'P0-01', 'theme reload retains route');
    await go(p, 'showcase.html?widget=P0-02&theme=green&scheme=dark');
    await p.getByRole('textbox',{name:'Your name',exact:true}).fill('Sapid reader');
    await p.getByText('Hello, Sapid reader!',{exact:true}).waitFor();
    await shot(p, 'textfield-dark');
    reports.push({suite:'showcase-theme-and-controlled-text',status:'passed'});

    for (const name of ['blue','green']) for (const scheme of ['light','dark']) {
      await go(p, `showcase.html?widget=P0-01&scheme=${scheme}&theme=${name === 'blue' ? 'green' : 'blue'}`);
      await p.getByRole('link',{name:`Use ${name} palette`,exact:true}).click();
      await p.locator(`.sapid-site.sapid-theme-${name}-${scheme}`).waitFor();
      const query = new URL(p.url()).searchParams;
      assert(query.get('widget') === 'P0-01' && query.get('theme') === name && query.get('scheme') === scheme, 'palette choice preserves guide and light/dark scheme');
      assert(await p.getByRole('link',{name:`Use ${name} palette`,exact:true}).getAttribute('aria-current') === 'location', 'selected palette is exposed as current');
      const swatch = await p.getByRole('link',{name:`Use ${name} palette`,exact:true}).evaluate(e => ({text:e.textContent.trim(),color:getComputedStyle(e).backgroundColor,outline:getComputedStyle(e).outlineColor}));
      assert(swatch.text === '' && swatch.color === (name === 'blue' ? 'rgb(25, 113, 194)' : 'rgb(35, 112, 50)'), 'palette is represented by its actual color, without a text label');

      const frame = await p.locator('.sapid-site').evaluate(e=>({background:getComputedStyle(e).backgroundColor,text:getComputedStyle(e).color}));
      assert(frame.background === (scheme === 'dark' ? 'rgb(20, 21, 23)' : 'rgb(248, 249, 250)'), 'page canvas comes from resolved palette data');
      assert(frame.text === (scheme === 'dark' ? 'rgb(222, 226, 230)' : 'rgb(33, 37, 41)'), 'page text comes from resolved palette data');
      const actual = await p.locator('.site-nav-link.is-active').evaluate(e=>getComputedStyle(e).color);
      const accent = name === 'blue' ? (scheme === 'dark' ? 'rgb(116, 192, 252)' : 'rgb(24, 100, 171)') : (scheme === 'dark' ? 'rgb(140, 233, 154)' : 'rgb(35, 112, 50)');
      assert(actual === accent, `${name}-${scheme}: website follows palette tokens (${actual})`);
      const filled = p.locator('.site-demo').getByRole('button',{name:'Filled button',exact:true});
      assert(await filled.evaluate(e=>getComputedStyle(e).backgroundColor) === (name === 'blue' ? 'rgb(25, 113, 194)' : 'rgb(35, 112, 50)'), 'preview follows the same palette');
      await p.getByRole('link',{name:`Switch to ${scheme === 'dark' ? 'light' : 'dark'} theme`,exact:true}).click();
      await p.locator(`.sapid-site.sapid-theme-${name}-${scheme === 'dark' ? 'light' : 'dark'}`).waitFor();
      assert(new URL(p.url()).searchParams.get('theme') === name, 'scheme choice preserves palette');
      reports.push({suite:`showcase-palette-${name}-${scheme}`,status:'passed'});
    }


    await go(p, 'showcase.html?widget=P0-03');
    const check = p.getByRole('checkbox',{name:'I agree to the terms and conditions',exact:true});
    await check.check(); assert(await check.isChecked(), 'real checkbox toggles');
    await p.getByText('Thank you for confirming.',{exact:true}).waitFor();
    reports.push({suite:'showcase-controlled-checkbox',status:'passed'});

    for (const hue of ['blue','green']) for (const scheme of ['light','dark']) for (const name of ['home','button','textfield','dates','chat']) {
      await go(p, `ssr-${name}-${hue}-${scheme}.html`);
      assert(await p.locator(`.sapid-site.site-${scheme}`).count() === 1, `SSR ${name}: adopted theme`);
      if (name !== 'home') assert(await p.locator('.site-nav-link').count() === 149, `SSR ${name}: full navigation`);
      reports.push({suite:`showcase-ssr-${name}-${hue}-${scheme}`,status:'passed'});
    }
    await p.goto(`${base}/ssr-textfield-blue-light.html?manual`);
    await p.waitForFunction(() => window.moduleReady === true);
    const dirty = p.getByRole('textbox',{name:'Your name',exact:true});
    await dirty.fill('Before hydration');
    await dirty.evaluate(e => {window.beforeShowcaseInput=e;e.focus();e.setSelectionRange(2,8,'backward');});
    await p.evaluate(() => window.showcase.mount(true,'P0-02','blue','light'));
    assert(await dirty.evaluate(e => e === window.beforeShowcaseInput && e.value === 'Before hydration' && e.selectionStart === 2 && e.selectionEnd === 8 && e.selectionDirection === 'backward'), 'SSR dirty editor identity/value/backward selection retained');
    await dirty.fill('After hydration');
    await p.getByText('Hello, After hydration!',{exact:true}).waitFor();
    reports.push({suite:'showcase-dirty-ssr-adoption',status:'passed'});

    const mobile = await browser.newContext({viewport:{width:360,height:800},hasTouch:true,isMobile:true});
    try {
      const m = await mobile.newPage(); setup(m);
      await go(m, 'showcase.html?widget=P0-01');
      for (const width of [320,360,390]) {
        await m.setViewportSize({width,height:800});
        const header = await m.locator('.site-header').evaluate(e => {
          const targets=[...e.querySelectorAll('.site-palette,.site-theme-toggle,.site-menu-toggle')].map(x=>{const r=x.getBoundingClientRect();return {left:r.left,right:r.right,top:r.top,bottom:r.bottom,width:r.width,height:r.height}});
          return {targets,overflow:document.body.scrollWidth>innerWidth+1};
        });
        assert(header.targets.length === 4 && !header.overflow && header.targets.every(r=>r.width>=22 && r.height>=22 && r.left>=0 && r.right<=width && r.top>=0 && r.bottom<=60), `${width}px: all four header controls visible without overflow`);
        assert(header.targets.slice(1).every((r,i)=>r.left>=header.targets[i].right), `${width}px: controls do not overlap`);
      }
      await m.setViewportSize({width:360,height:800});
      await m.getByRole('link',{name:'Use green palette',exact:true}).tap();
      await m.locator('.sapid-site.sapid-theme-green-light').waitFor();
      assert(new URL(m.url()).searchParams.get('widget') === 'P0-01', 'mobile swatch preserves guide');

      await m.getByRole('button',{name:'Open navigation',exact:true}).tap();
      await m.getByRole('button',{name:'Close navigation',exact:true}).waitFor();
      assert(await m.locator('.sapid-site.menu-open .site-sidebar').isVisible(), 'mobile Rust menu opens');
      await shot(m, 'mobile-navigation');
      await m.locator('.site-nav-link[href*="widget=P0-02&"]').tap();
      await m.locator('h1.site-title').waitFor();
      await m.getByRole('button',{name:'Open navigation',exact:true}).waitFor();
      assert(new URL(m.url()).searchParams.get('widget') === 'P0-02', 'mobile chooses real docs route');
      assert(await m.evaluate(() => innerWidth === 360 && document.documentElement.clientWidth === 360 && document.body.scrollWidth <= 361), 'mobile document contains its content');
      assert(await m.locator('.site-doc-note').evaluateAll(notes => notes.length > 0 && notes.every(note => getComputedStyle(note).display === 'block' && note.getClientRects().length === 1)), 'mobile descriptions have contiguous readable block backgrounds');
      await shot(m, 'mobile-docs');
      await go(m, 'showcase.html?theme=green&scheme=dark');
      await m.getByRole('button',{name:'Open navigation',exact:true}).tap();
      const menuTop = await m.locator('.site-mobile-nav').evaluate(e=>e.getBoundingClientRect().top);
      const headerBottom = await m.locator('.site-header').evaluate(e=>e.getBoundingClientRect().bottom);
      assert(Math.abs(menuTop-headerBottom)<=1, 'home mobile menu starts directly below the single header row');
      await shot(m, 'mobile-home-navigation');

    } finally { await mobile.close(); }
    reports.push({suite:'showcase-mobile-navigation',status:'passed'});

    await go(p, 'showcase.html?widget=does-not-exist');
    await p.getByRole('heading',{name:'Widget not found',exact:true}).waitFor();
    await p.getByRole('link',{name:'Sapid',exact:true}).click();
    await p.waitForFunction(()=>document.documentElement.getAttribute('data-sapid-widget')==='');
    await p.getByRole('main').waitFor();
    assert(!new URL(p.url()).searchParams.get('widget'), 'friendly 404 can return home');
    reports.push({suite:'showcase-friendly-404',status:'passed'});
    return {reports,errors};
  } catch (error) {
    errors.push(error.stack || String(error));
    if (activePage) {
      errors.push(JSON.stringify(await activePage.evaluate(() => ({url:location.href,ready:window.ready,title:document.querySelector('h1')?.textContent,body:document.body.innerText.slice(-1600)})).catch(e => ({diagnostic:String(e)}))));
      await shot(activePage,'acceptance-failure').catch(() => {});
    }
    return {reports,errors};
  }
  finally { await context.close(); }
}
