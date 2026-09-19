<script>
 import {onMount,tick} from 'svelte';
 import {invoke} from '@tauri-apps/api/core';
 import {listen} from '@tauri-apps/api/event';
 import catalog from './data/catalog.json';
 import Wheel from './lib/Wheel.svelte';
 import Select from './lib/Select.svelte';
 import Arrows from './lib/Arrows.svelte';
 import MouseButton from './lib/MouseButton.svelte';
 import Updater from './lib/Updater.svelte';
 import {version} from '../package.json';
 import {dropEntry,findNode,wheelDropTarget} from './lib/drag.js';
 import {translate,resolveLocale,languages} from './lib/i18n.js';
 import {defaults,initialProfile,getFolder,keyOptions,uid,sectorAt,validateState,validateSettings} from './lib/model.js';

 const native=!!window.__TAURI_INTERNALS__;
 const overlay=new URLSearchParams(location.search).has('overlay');
 const starter=initialProfile(catalog);
 let saved,loadError='';
 try{const raw=localStorage.getItem('serapod.v1') || localStorage.getItem('stratcom.v1');if(raw)saved=validateState(JSON.parse(raw),catalog);}catch{loadError='loadError';}
 let profiles=$state(saved?.profiles||[starter]);
 let activeId=$state(saved?.activeId||starter.id);
 let settings=$state({...defaults,...saved?.settings});
 let systemLocale=$state(navigator.language||'en');
 let locale=$derived(resolveLocale(settings.language,systemLocale));
 const t=(key,params={})=>translate(locale,key,params);
 const named=n=>n.nameKey?t(n.nameKey):n.name;
 const keyLabel=key=>({ControlLeft:'Ctrl L',ControlRight:'Ctrl R',ShiftLeft:'Shift L',ShiftRight:'Shift R',AltLeft:'Alt L',AltRight:'Alt R',ArrowUp:t('up'),ArrowDown:t('down'),ArrowLeft:t('left'),ArrowRight:t('right'),Mouse4:`${t('mouse')} 4`,Mouse5:`${t('mouse')} 5`}[key]||key.replace('Key','').replace('Digit',''));
 let page=$state('editor'),path=$state([]),search=$state(''),category=$state('all');
 let updater=$state(),updateOpen=$state(false);
 let selected=$state(-1),toast=$state(null),noticeTimer;
 let status=$state({game:false,foreground:false,hookReady:false,overlayReady:false,configured:false,enabled:false,error:'',triggerCount:0,openCount:0,overlayVisible:false}),bridgeReady=$state(false);
 let preview=$state(false),previewPath=$state([]),previewSelected=$state(-1),previewHeld=false;
 let nativeFrame=$state({open:false,items:[],selected:-1,title:'SERAPOD',locale:'en',testing:false});
 let folderName=$state(''),profileName=$state(''),rename=$state(''),recording=$state('');
 let importInput=$state();
 let syncPending=$state(false),syncError=$state('');
 let drag=$state(null),drop=$state(null),suppressClick=false;
 let itemMenu=$state(null),menuButton=$state();
 async function openItemMenu(e,item){
  cancelDrag();
  const rect=e.currentTarget.getBoundingClientRect();
  itemMenu={id:item.id,name:itemName(item),origin:e.currentTarget,x:Math.max(8,Math.min(e.clientX||rect.left+rect.width/2,window.innerWidth-208)),y:Math.max(8,Math.min(e.clientY||rect.top+rect.height/2,window.innerHeight-60))};
  await tick();menuButton?.focus();
 }
 function dismissItemMenu(e){if(!e.target.closest?.('.item-context-menu'))itemMenu=null;}
 function removeMenuItem(){
  if(!itemMenu)return;
  const error=dropEntry(profile.root,{nodeId:itemMenu.id},null,0,'trash');
  itemMenu=null;selected=-1;if(error)flash(error);
 }
 function startDrag(e,payload,label){if(e.button!==0)return;drag={...payload,label,x:e.clientX,y:e.clientY,startX:e.clientX,startY:e.clientY,active:false};}
 function dragMove(e){
  if(!drag)return;
  drag.x=e.clientX;drag.y=e.clientY;
  if(!drag.active && Math.hypot(drag.x-drag.startX,drag.y-drag.startY)<6)return;
  drag.active=true;e.preventDefault();drop=null;
  const hit=document.elementFromPoint(e.clientX,e.clientY);
  if(hit?.closest('[data-drag-trash]')){drop={mode:'trash'};return;}
  const crumb=hit?.closest('[data-folder-target]');
  if(crumb){drop={targetId:crumb.dataset.folderTarget,index:999,into:true};return;}
  const row=hit?.closest('[data-slot]');
  if(row){const i=Number(row.dataset.slot),rect=row.getBoundingClientRect(),fraction=(e.clientY-rect.top)/rect.height,n=folder.children[i];
   drop=n.kind==='folder'&&fraction>.25&&fraction<.75?{targetId:n.id,index:n.children.length,into:true,row:i}:{targetId:folder.id,index:i+(fraction>.5?1:0),row:i,after:fraction>.5};return;}
  const stage=hit?.closest('.wheel-stage');
  if(stage){const rect=stage.querySelector('svg.wheel').getBoundingClientRect(),x=(e.clientX-rect.left-rect.width/2)*560/rect.width,y=(e.clientY-rect.top-rect.height/2)*560/rect.height,n=folder.children.length;
   const target=wheelDropTarget(x,y,n);if(!target)return;
   const entry=folder.children[target.index];
   if(target.mode==='on'&&entry?.kind==='folder'){drop={targetId:entry.id,index:entry.children.length,into:true,row:target.index};}
   else {if(target.mode==='on'&&drag.nodeId&&findNode(profile.root,drag.nodeId)?.node.kind==='folder')target.mode='insert';drop={targetId:folder.id,...target};}return;}
  if(hit?.closest('.slots'))drop={targetId:folder.id,index:folder.children.length};
 }
 function finishDrag(e){if(drag?.active){dragMove(e);suppressClick=true;setTimeout(()=>suppressClick=false,0);if(drop){const payload=drag.nodeId?{nodeId:drag.nodeId}:{entry:{id:uid(),kind:'stratagem',stratagemId:drag.stratagemId}};const error=dropEntry(profile.root,payload,drop.targetId,drop.index,drop.mode);if(error)flash(error);else selected=-1;}}drag=null;drop=null;}
 function cancelDrag(){drag=null;drop=null;}
 function captureClick(e){if(suppressClick){e.preventDefault();e.stopPropagation();}}
 const categories=[...new Set(catalog.map(c=>c.category))];
 let profile=$derived(profiles.find(p=>p.id===activeId)||profiles[0]);
 let folder=$derived(getFolder(profile.root,path));
 let previewFolder=$derived(getFolder(profile.root,previewPath));
 let filtered=$derived(catalog.filter(c=>(category==='all'||c.category===category)&&c.name.toLowerCase().includes(search.toLowerCase())));
 let settingsError=$derived(validateSettings(settings));
 let stateLabel=$derived(!native?'browser':settingsError||syncError||status.error?'engineError':!status.game?'waiting':status.enabled?'ready':!status.hookReady?'waitingHooks':!status.overlayReady?'waitingOverlay':!status.configured?'waitingConfig':!status.gameWindow?'waitingWindow':'starting');
 const lookup=id=>catalog.find(c=>c.id===id);
 const itemName=n=>n.kind==='folder'?named(n):lookup(n.stratagemId)?.name;
 function flash(key,params={}){toast={key,params};clearTimeout(noticeTimer);noticeTimer=setTimeout(()=>toast=null,6000);}
 function snapshotState(){return {version:1,profiles:$state.snapshot(profiles),activeId,settings:$state.snapshot(settings)};}
 function nativeNode(n){return n.kind==='folder'?{...n,name:named(n),children:n.children.map(nativeNode)}:{...n,name:lookup(n.stratagemId)?.name,code:lookup(n.stratagemId)?.code,icon:lookup(n.stratagemId)?.icon};}
 $effect(()=>{document.documentElement.lang=overlay?nativeFrame.locale:locale;});
 $effect(()=>{
  if(overlay)return;
  const data=snapshotState();
  try{localStorage.setItem('serapod.v1',JSON.stringify(data));}catch{flash('saveError');}
 });
 // Debounce typing and only send the active profile to the native engine.
 $effect(()=>{
  if(overlay||!native||!bridgeReady)return;
  const config={settings:{...$state.snapshot(settings),locale},root:nativeNode(profile.root)};
  syncPending=true;
  let disposed=false;
  const timer=setTimeout(()=>invoke('configure',{config}).then(()=>{if(!disposed){syncError='';syncPending=false;}}).catch(e=>{if(!disposed){syncError=String(e);syncPending=false;flash(String(e));}}),150);
  return ()=>{disposed=true;clearTimeout(timer);};
 });
 function add(c){if(folder.children.length>=12)return flash('maxSectors');folder.children.push({id:uid(),kind:'stratagem',stratagemId:c.id});flash('added',{name:c.name});}
 function addFolder(){if(!folderName.trim())return;if(path.length>=7)return flash('maxDepth');if(folder.children.length>=12)return flash('maxSectors');folder.children.push({id:uid(),kind:'folder',name:folderName.trim().slice(0,80),children:[]});folderName='';}
 function enter(i){selected=i;const n=folder.children[i];if(n?.kind==='folder'){path=[...path,n.id];selected=-1;}}
 function move(i,d){const j=i+d;if(j<0||j>=folder.children.length)return;[folder.children[i],folder.children[j]]=[folder.children[j],folder.children[i]];selected=j;}
 function newProfile(){if(!profileName.trim())return;if(profiles.length>=32)return flash('maxProfiles');const p={id:uid(),name:profileName.trim().slice(0,80),root:{id:uid(),kind:'folder',name:t('mainWheel'),nameKey:'mainWheel',children:[]}};profiles.push(p);activeId=p.id;path=[];profileName='';}
 function resetApp(){
  if(!confirm(t('resetAppConfirm')))return;
  cancelDrag();cancelPreview();itemMenu=null;recording='';
  const p=initialProfile(catalog);profiles=[p];activeId=p.id;settings={...defaults};path=[];selected=-1;search='';category='all';folderName='';profileName='';rename='';flash('resetAppDone');
 }
 function cloneProfile(){if(profiles.length>=32)return flash('maxProfiles');const p=JSON.parse(JSON.stringify(profile));function ids(n){n.id=uid();n.children?.forEach(ids);}ids(p.root);p.id=uid();p.name=(named(profile)+' — '+t('copy')).slice(0,80);delete p.nameKey;profiles.push(p);activeId=p.id;path=[];}
 function deleteProfile(p){if(confirm(t('deleteConfirm',{name:named(p)}))){profiles=profiles.filter(x=>x.id!==p.id);if(activeId===p.id)activeId=profiles[0].id;path=[];}}
 function exportProfiles(){const url=URL.createObjectURL(new Blob([JSON.stringify(snapshotState(),null,2)],{type:'application/json'}));const a=document.createElement('a');a.href=url;a.download='serapod-profiles.json';a.click();URL.revokeObjectURL(url);flash('exported');}
 async function importProfiles(e){const file=e.target.files?.[0];if(!file)return;try{if(file.size>2e6)throw Error('fileTooLarge');const data=validateState(JSON.parse(await file.text()),catalog);profiles=data.profiles;activeId=data.activeId;settings={...defaults,...data.settings};path=[];flash('imported');}catch(e){flash(e instanceof SyntaxError?'invalidFile':String(e.message));}e.target.value='';}
 async function testOverlay(){if(!native)return flash('nativeOnly');try{await invoke('test_overlay');}catch(e){flash(String(e));}}
 async function restartOverlay(){try{await invoke('restart_overlay');}catch(e){flash(String(e));}}
 function openPreview(){previewPath=[];previewSelected=-1;preview=true;}
 function cancelPreview(){preview=false;previewSelected=-1;}
 function previewBack(e){e.preventDefault();if(previewPath.length){previewPath=previewPath.slice(0,-1);previewSelected=-1;}else cancelPreview();}
 function commitPreview(click=false){
  const n=previewFolder.children[previewSelected];
  if(!n){if(click&&previewPath.length){previewPath=previewPath.slice(0,-1);previewSelected=-1;}else cancelPreview();return;}
  if(n.kind==='folder'){if(click){previewPath=[...previewPath,n.id];previewSelected=-1;}else cancelPreview();return;}
  cancelPreview();flash('simulated',{name:itemName(n)});
 }
 function previewMove(e){const svg=e.currentTarget.querySelector('svg');const rect=svg.getBoundingClientRect();previewSelected=sectorAt((e.clientX-rect.left-rect.width/2)*560/rect.width,(e.clientY-rect.top-rect.height/2)*560/rect.height,previewFolder.children.length,settings.deadzone);}
 function keydown(e){
  if(updateOpen)return;
  if(itemMenu){if(e.key==='Escape'){e.preventDefault();itemMenu.origin?.focus();itemMenu=null;}else if(e.key==='Tab'){itemMenu=null;}return;}
  if(recording){e.preventDefault();e.stopPropagation();if(e.key==='Escape'){recording='';return;}if(keyOptions.includes(e.code)){settings[recording]=e.code;recording='';}return;}
  if(e.key==='Escape'){cancelDrag();cancelPreview();return;}
  if(overlay||/INPUT|SELECT|TEXTAREA/.test(e.target.tagName))return;
  if(e.code===settings.trigger){e.preventDefault();if(!e.repeat){previewHeld=true;openPreview();}}
 }
 function keyup(e){if(e.code===settings.trigger&&previewHeld){e.preventDefault();previewHeld=false;if(preview)commitPreview();}}
 onMount(()=>{
  let unlisten,interval,retryTimer,disposed=false,lastError='';
  if(loadError)flash(loadError);
  if(native){
   if(overlay){
    const ready=()=>invoke('overlay_ready').catch(()=>{if(!disposed)retryTimer=setTimeout(ready,500);});
    const connect=()=>listen('wheel',e=>nativeFrame=e.payload).then(fn=>{if(disposed)fn();else{unlisten=fn;ready();}}).catch(()=>{if(!disposed)retryTimer=setTimeout(connect,500);});connect();
   }
   else {
    invoke('system_locale').then(value=>{if(!disposed)systemLocale=value;}).catch(()=>{}).finally(()=>{if(!disposed)bridgeReady=true;});
    const poll=()=>invoke('status').then(s=>{if(disposed)return;status=s;if(s.error&&s.error!==lastError)flash(s.error);lastError=s.error;}).catch(e=>{if(!disposed)flash(String(e));});poll();interval=setInterval(poll,750);
   }
  }
  return ()=>{disposed=true;unlisten?.();clearInterval(interval);clearTimeout(retryTimer);clearTimeout(noticeTimer);};
 });
</script>

<svelte:window onkeydown={keydown} onkeyup={keyup} onpointerdown={dismissItemMenu} onscroll={()=>itemMenu=null} onresize={()=>itemMenu=null} onpointermove={dragMove} onpointerup={finishDrag} onpointercancel={cancelDrag} ondragstart={e=>e.preventDefault()} onclickcapture={captureClick} onblur={()=>{itemMenu=null;cancelDrag();previewHeld=false;recording='';cancelPreview();}} />

{#if overlay}
 <div class="native-overlay" style:visibility={nativeFrame.open?'visible':'hidden'}>
  <Wheel items={nativeFrame.items} {catalog} selected={nativeFrame.selected} title={nativeFrame.title} subtitle={translate(nativeFrame.locale,nativeFrame.testing?'simulation':'releaseConfirm')} size={560} locale={nativeFrame.locale} interactive={false} glass={nativeFrame.glass??true} transparency={nativeFrame.transparency??35} cursor={nativeFrame.testing?null:(nativeFrame.cursor??[0,0])}/>
  <div class="overlay-hint">{#if nativeFrame.testing}{translate(nativeFrame.locale,'overlayTest')}{:else}<span><MouseButton label={translate(nativeFrame.locale,'hintLeftMouse')}/>{translate(nativeFrame.locale,'hintSelect')}</span><span><MouseButton button="right" label={translate(nativeFrame.locale,'hintRightMouse')}/>{translate(nativeFrame.locale,'hintBack')}</span><span><kbd>Esc</kbd>{translate(nativeFrame.locale,'hintCancel')}</span>{/if}</div>
 </div>
{:else}
 <div class="app-shell">
  <aside class="sidebar">
   <div class="brand"><img src="/logo.svg" alt=""/><span>SERAPOD</span></div>
   <nav aria-label={t('navigation')}>
    {#each [['editor','◎'],['profiles','▱'],['settings','⚙']] as [key,icon]}<button class:active={page===key} onclick={()=>page=key} title={t(key)}><span aria-hidden="true">{#if key==='profiles'}<svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="1.5"><path d="m12 3 9 5-9 5-9-5zM3 12l9 5 9-5M3 16l9 5 9-5"/></svg>{:else}{icon}{/if}</span><span class="nav-label">{t(key)}</span></button>{/each}
   </nav>
   <div class="sidebar-bottom"><span class="signature">Made with <span class="heart">♥</span> by Ødesså</span><span class="version">SERAPOD <span>{version}</span></span></div>
  </aside>
  <main>
   <header class="page-heading"><div class="heading-title"><h1>{t(page)}</h1>{#if page==='editor'}<Select variant="profile-select" label={t('activeProfile')} bind:value={activeId} options={profiles.map(p=>({value:p.id,label:named(p)}))} onchange={()=>{path=[];selected=-1;}}/>{/if}</div><div class="header-actions"><span class="game-status" class:ready={stateLabel==='ready'} class:error={stateLabel==='engineError'} role="status"><i></i>{t(stateLabel)}</span>{#if page==='editor'}<button class="primary" onclick={openPreview}>{t('testWheel')} <kbd>{keyLabel(settings.trigger)}</kbd></button>{/if}</div></header>
   {#if page==='editor'}
    <div class="editor-grid">
     <section class="panel radial-panel"><div class="panel-heading"><h2>{t('wheel')}</h2><span class="subtle">{t('sectors',{count:folder.children.length})}</span></div>
      <div class="wheel-breadcrumb"><button data-folder-target={profile.root.id} class:drop-target={drop?.targetId===profile.root.id&&drop?.into} onclick={()=>{path=[];selected=-1;}}>{t('mainWheel')}</button>{#each path as id,i}<span>/</span><button data-folder-target={id} class:drop-target={drop?.targetId===id&&drop?.into} onclick={()=>{path=path.slice(0,i+1);selected=-1;}}>{named(getFolder(profile.root,path.slice(0,i+1)))}</button>{/each}</div>
      <div class="wheel-stage"><Wheel items={folder.children} {catalog} {selected} title={named(folder)} subtitle={t('preview')} {locale} dragging={!!drag?.active} draggedId={drag?.active?drag.nodeId:null} dropAction={drop?.into?t('dragInside'):drop?.mode==='on'?t(drag?.nodeId?'dragSwap':'dragReplace'):''} onitemcontext={openItemMenu} onitemdown={(e,item)=>startDrag(e,{nodeId:item.id},itemName(item))} dropIndex={drop?.targetId===folder.id&&!drop?.into&&drop?.mode!=='on'?drop.index:-1} dropFolder={drop?.into?folder.children.findIndex(n=>n.id===drop.targetId):drop?.mode==='on'?drop.index:-1} onselect={enter} oncenter={()=>{path=path.slice(0,-1);selected=-1;}}/>{#if !folder.children.length}<div class="empty-wheel">{t('addEmpty')}</div>{/if}</div>
      <div class="wheel-help"><span><kbd>{keyLabel(settings.trigger)}</kbd> {t('hold')}</span><span>{t('mouse')} · {t('select')}</span><span>{t('release')} · {t('prepare')}</span></div>
      <div class="slots-heading">{t('wheelContents')}<p class="drag-help">{t('dragHint')}</p></div>
      <div class="slots">{#each folder.children as item,i (item.id)}
       <div data-slot={i} class:selected={selected===i} class:drop-before={drop?.row===i&&!drop?.into&&!drop?.after} class:drop-after={drop?.row===i&&!drop?.into&&drop?.after} class:drop-target={drop?.row===i&&drop?.into} class="slot"><span class="slot-number">{String(i+1).padStart(2,'0')}</span>{#if item.kind==='folder'}<svg class="folder-mini" viewBox="0 0 50 40" width="28" height="28" aria-hidden="true"><path class="folder-symbol" d="M2 3h17l5 6h23v28H2z"/></svg>{:else}<img class="slot-icon" src={lookup(item.stratagemId)?.icon} alt=""/>{/if}
        <button class="slot-name" onpointerdown={e=>startDrag(e,{nodeId:item.id},itemName(item))} onclick={()=>enter(i)}><span>{itemName(item)}</span>{#if item.kind==='folder'}<small>{t('openFolder',{count:item.children.length})}</small>{:else}<Arrows code={lookup(item.stratagemId)?.code||[]} {locale} small/>{/if}</button>
        <button class="icon-button" aria-label={t('moveUp',{name:itemName(item)})} disabled={i===0} onclick={()=>move(i,-1)}>↑</button><button class="icon-button" aria-label={t('moveDown',{name:itemName(item)})} disabled={i===folder.children.length-1} onclick={()=>move(i,1)}>↓</button><button class="icon-button remove" aria-label={t('remove',{name:itemName(item)})} onclick={()=>{folder.children.splice(i,1);selected=-1;}}>×</button>
       </div>
      {/each}</div>
      <form class="new-folder" onsubmit={e=>{e.preventDefault();addFolder();}}><input aria-label={t('newFolder')} placeholder={t('newFolder')} maxlength="80" bind:value={folderName}/><button type="submit">＋ {t('create')}</button></form>
      {#if path.length}<form class="rename-folder" onsubmit={e=>{e.preventDefault();if(rename.trim()){folder.name=rename.trim().slice(0,80);delete folder.nameKey;rename='';}}}><input aria-label={t('renameFolder')} placeholder={t('renameFolder')} maxlength="80" bind:value={rename}/><button>{t('rename')}</button></form>{/if}
     </section>
     <section class="panel catalog-panel"><div class="panel-heading"><h2>{t('catalog')}</h2><span class="subtle">{catalog.length}</span></div>
      <div class="catalog-controls"><input class="search-field" type="search" aria-label={t('search')} placeholder={t('search')} bind:value={search}/><div class="category-chips"><button class:chosen={category==='all'} onclick={()=>category='all'}>{t('all')}</button>{#each categories as c}<button class:chosen={category===c} onclick={()=>category=c}>{t('category.'+c)}</button>{/each}</div></div>
      <div class="catalog-count">{t('results',{count:filtered.length})}<span>{t('clickAdd')}</span></div>
      <div class="catalog-list">{#each filtered as c}<button class="catalog-card" onpointerdown={e=>startDrag(e,{stratagemId:c.id},c.name)} onclick={()=>add(c)} title={t('add',{name:c.name})}><div class="catalog-icon"><img src={c.icon} alt=""/></div><div class="catalog-info"><span>{c.name}</span><small>{t('category.'+c.category)}</small><Arrows code={c.code} {locale}/></div><span class="add-symbol">＋</span></button>{:else}<div class="empty">{t('noResults')}<button onclick={()=>{search='';category='all';}}>{t('clearFilters')}</button></div>{/each}</div>
      <div class="catalog-footer">{t('offline')}</div>
     </section>
    </div>
   {:else if page==='settings'}
    <section class="panel general-panel"><div><h2>{t('language')}</h2><p>{t('systemLanguage',{locale:systemLocale})}</p></div><Select variant="language-select" label={t('language')} bind:value={settings.language} options={[{value:'auto',label:t('automatic')},...Object.entries(languages).map(([value,label])=>({value,label}))]}/><p class="language-help">{t('languageHelp')}</p></section>
    <div class="settings-grid">
     <section class="panel settings-panel"><div class="panel-heading"><h2>{t('shortcuts')}</h2></div><p class="section-description">{t('bindingsHelp')}</p>
      {#each ['trigger','gameKey','up','down','left','right'] as key}<div class="setting-row"><label for={`key-${key}`}>{t(key)}</label><div class="key-control"><Select variant="key-select" id={`key-${key}`} label={t(key)} bind:value={settings[key]} options={keyOptions.filter(k=>key==='trigger'||!k.startsWith('Mouse')).map(k=>({value:k,label:keyLabel(k)}))}/><button class:recording={recording===key} type="button" onclick={()=>recording=key}>{t(recording===key?'recording':'capture')}</button></div></div>{/each}
      <p class="muted">{t('captureHelp')}</p><label class="setting-row"><span>{t('mode')}</span><Select variant="mode-select" label={t('mode')} bind:value={settings.mode} options={[{value:'hold',label:t('hold')},{value:'toggle',label:t('toggle')}]}/></label>{#if settingsError}<p class="error-message">{t(settingsError)}</p>{/if}
     </section>
     <section class="panel settings-panel"><div class="panel-heading"><h2>{t('timing')}</h2></div>
      {#each [['pressMs',15,250,5,'ms'],['gapMs',15,250,5,'ms'],['size',400,720,10,'px'],['deadzone',30,100,5,'px'],['sensitivity',0.3,3,0.1,'×']] as [key,min,max,step,unit]}<label class="range-row"><span>{t(key)}<b>{settings[key]} {unit}</b></span><input type="range" {min} {max} {step} bind:value={settings[key]}/></label>{/each}
      <label class="setting-row"><span>{t('glass')}</span><input type="checkbox" bind:checked={settings.glass}/></label>
      <label class="range-row"><span>{t('transparency')}<b>{settings.transparency} %</b></span><input type="range" min="0" max="80" step="5" disabled={!settings.glass} bind:value={settings.transparency}/></label><p class="muted">{t('glassHelp')}</p>
      <div class="button-row"><button class="reset-app" onclick={resetApp}>{t('resetApp')}</button><button disabled={!native} onclick={()=>updater?.check(true)}>{t('checkUpdates')}</button></div><p class="muted">{t('gameHelp')}</p>
     </section>
     <section class="panel settings-panel diagnostics"><div class="panel-heading"><h2>{t('diagnostics')}</h2></div>
      <div class="diagnostic-grid">{#each [['gameProcess',status.game],['gameWindow',status.gameWindow],['configured',status.configured],['foreground',status.foreground],['hooks',status.hookReady],['overlayReady',status.overlayReady],['overlayVisible',status.overlayVisible]] as [label,ok]}<div><span>{t(label)}</span><strong class:ok>{t(ok?'available':'unavailable')}</strong></div>{/each}</div>
      <p class="muted">{t('lastTrigger',{count:status.triggerCount})} · {t('lastOpen',{count:status.openCount})}</p>
      {#if status.game&&!status.gameWindow}<p class="muted">{status.detectionDetail}</p>{/if}
      {#if native}<button onclick={restartOverlay}>{t('retryOverlay')}</button>{/if}
      <div class="overlay-test-row"><button disabled={!native||!status.overlayReady||syncPending||!!settingsError} onclick={testOverlay}>{t('testOverlay')}</button><p>{t('testOverlayHint')}</p></div>{#if status.error||syncError}<p class="error-message">{t(status.error||syncError)}</p>{/if}
     </section>
    </div>
   {:else}
    <div class="profiles-layout"><section class="panel settings-panel"><div class="panel-heading"><h2>{t('savedProfiles')}</h2></div>
     {#each profiles as p}<div class="profile-row"><button class:chosen={p.id===activeId} onclick={()=>{activeId=p.id;path=[];}}><strong>{named(p)}<small>{t('rootSectors',{count:p.root.children.length})}</small></strong>{#if p.id===activeId}<span class="tag">{t('active')}</span>{/if}</button><button class="icon-button" aria-label={t('deleteProfile',{name:named(p)})} disabled={profiles.length===1} onclick={()=>deleteProfile(p)}>×</button></div>{/each}
     <form class="new-folder" onsubmit={e=>{e.preventDefault();newProfile();}}><input placeholder={t('newProfile')} aria-label={t('newProfile')} maxlength="80" bind:value={profileName}/><button class="primary">＋ {t('create')}</button></form>
     <label class="setting-row"><span>{t('profileName')}</span><input value={named(profile)} maxlength="80" onchange={e=>{if(e.target.value.trim()){profile.name=e.target.value.trim();delete profile.nameKey;}else e.target.value=named(profile);}}/></label><button class="duplicate-button" onclick={cloneProfile}>{t('duplicate')}</button>
    </section><section class="panel settings-panel"><div class="panel-heading"><h2>{t('transfer')}</h2></div><p class="section-description">{t('transferHelp')}</p><div class="button-row"><button class="primary" onclick={exportProfiles}>{t('export')}</button><button onclick={()=>importInput.click()}>{t('import')}</button><input bind:this={importInput} type="file" accept=".json,application/json" hidden onchange={importProfiles}/></div><p class="muted">{t('importHelp')}</p><div class="info-box"><strong>{t('credits')}</strong><p>{t('creditsText')}</p><p>{t('unaffiliated')}</p></div></section></div>
   {/if}
   <footer><span>{t('saved')}</span></footer>
  </main>
 </div>
 {#if itemMenu}<div class="item-context-menu" role="menu" aria-label={itemMenu.name} style:left={itemMenu.x+'px'} style:top={itemMenu.y+'px'}><button bind:this={menuButton} role="menuitem" onclick={removeMenuItem}>{t('removeItem')}</button></div>{/if}
 {#if drag?.active}<div class="drag-ghost" style:left={`${drag.x+16}px`} style:top={`${drag.y+16}px`}>{drag.label}{#if drop?.mode==='on'}<small>{t(drag.nodeId?'dragSwap':'dragReplace')}</small>{/if}</div><div class="drag-trash" class:over={drop?.mode==='trash'} data-drag-trash role="presentation"><svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.7" aria-hidden="true"><path d="M3 6h18M9 6V3h6v3M5 6l1 15h12l1-15M10 10v7M14 10v7"/></svg><span>{t(drag.nodeId?'dragRemove':'dragCancel')}</span></div>{/if}
 {#if native}<Updater bind:this={updater} {locale} bind:open={updateOpen}/>{/if}
 {#if toast}<div class="toast" role="status">{t(toast.key,toast.params)}<button aria-label={t('close')} onclick={()=>toast=null}>×</button></div>{/if}
 {#if preview}<div class="preview-scrim" role="presentation" oncontextmenu={previewBack}><div class="preview-label">{t('simulation')}<span>{t('simulationHint')}</span></div><div class="preview-wheel" role="presentation" onmousemove={previewMove} style:width={`${settings.size}px`} style:height={`${settings.size}px`}><Wheel items={previewFolder.children} {catalog} selected={previewSelected} title={named(previewFolder)} subtitle={t('simulation')} size={settings.size} {locale} glass={settings.glass} transparency={settings.transparency} onselect={i=>{previewSelected=i;commitPreview(true);}} oncenter={()=>{previewSelected=-1;commitPreview(true);}}/></div><p>{t('simulationGuide')}</p><button onclick={cancelPreview}>{t('close')}</button></div>{/if}
{/if}
