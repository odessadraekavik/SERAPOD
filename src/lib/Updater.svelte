<script>
 import {onMount} from 'svelte';
 import {invoke} from '@tauri-apps/api/core';
 import {listen} from '@tauri-apps/api/event';
 import {translate} from './i18n.js';
 import {version} from '../../package.json';
 let {locale='en',open=$bindable(false)}=$props();
 const t=(key,params={})=>translate(locale,key,params);
 let dialog=$state(),offer=$state(null),checking=$state(false),busy=$state(false),error=$state('');
 let progress=$state({downloaded:0,total:1,stage:'downloading'});
 const percent=$derived(Math.min(100,Math.floor(progress.downloaded/Math.max(1,progress.total)*100)));
 $effect(()=>{if(!dialog)return;if(open&&!dialog.open)dialog.showModal();else if(!open&&dialog.open)dialog.close();});
 export async function check(manual=false){
  if(checking||busy)return;
  checking=true;error='';offer=null;if(manual)open=true;
  try{offer=await invoke('check_update');if(offer)open=true;}
  catch(e){if(manual){error=String(e);open=true;}}
  finally{checking=false;}
 }
 async function install(){
  if(busy||!offer)return;
  busy=true;error='';progress={downloaded:0,total:1,stage:'downloading'};
  try{await invoke('install_update');}
  catch(e){error=String(e);busy=false;}
 }
 function dismiss(){if(!busy)open=false;}
 onMount(()=>{
  let disposed=false,unlisten;
  listen('update-progress',e=>{progress=e.payload;}).then(fn=>{if(disposed){fn();return;}unlisten=fn;check();})
   .catch(()=>{});
  // Report a healthy frontend before removing the previous executable.
  invoke('finish_update').catch(e=>{error=String(e);open=true;});
  return()=>{disposed=true;unlisten?.();};
 });
</script>

<dialog bind:this={dialog} class="update-dialog" aria-labelledby="update-title" oncancel={e=>{e.preventDefault();dismiss();}}>
 <div class="update-heading"><img src="/logo.svg" alt="" width="42" height="42"/><div><div class="update-eyebrow">SERAPOD · {t('updateDispatch')}</div><h2 id="update-title">{t(checking?'updateChecking':offer?'updateAvailable':error?'updateProblem':'updateCurrent')}</h2></div></div>
 {#if offer}
  <div class="update-versions"><div><span>{t('updateInstalled')}</span><strong class="update-old">{version}</strong></div><span class="update-arrow" aria-hidden="true">→</span><div><span>{t('updateNew')}</span><strong class="update-new">{offer.version}</strong></div></div>
  <p class="update-description">{t('updateQuestion')}</p>
  {#if offer.notes}<div class="update-notes"><h3>{t('updateNotes')}</h3><p>{offer.notes}</p></div>{/if}
 {:else if !checking&&!error}<p class="update-description">{t('updateLatest',{version})}</p>{/if}
 {#if busy}<div class="update-progress" role="status" aria-live="polite"><div><span>{t('updateStage_'+progress.stage)}</span><strong>{percent}%</strong></div><progress max="100" value={percent} aria-label={t('updateDownloadProgress')}></progress></div>{/if}
 {#if error}<p class="update-error" role="alert">{t(error)}</p>{/if}
 <div class="update-actions"><button disabled={busy} onclick={dismiss}>{t(offer?'updateNo':'close')}</button>{#if offer}<button class="primary" disabled={busy||checking} onclick={install}>{t('updateNow')}</button>{/if}</div>
</dialog>

<style>
 .update-dialog{width:min(500px,calc(100vw - 40px));max-height:calc(100vh - 64px);overflow:auto;margin:auto;padding:26px;border:1px solid #484e58;border-radius:12px;background:#191d23;color:#eef0f3;box-shadow:0 24px 90px #0009;}
 .update-dialog::backdrop{background:#090c12bb;backdrop-filter:blur(5px);}
 .update-heading{display:flex;gap:14px;align-items:center;margin-bottom:24px;}
 .update-eyebrow{color:#b9bfca;font-size:10px;letter-spacing:1.5px;text-transform:uppercase;margin-bottom:5px;}
 h2{font-size:20px;}h3{font-size:11px;color:#b9bfca;margin:0 0 8px;text-transform:uppercase;letter-spacing:1px;}
 .update-versions{display:flex;align-items:center;gap:22px;padding:17px;background:#101318;border:1px solid #303640;border-radius:7px;margin-bottom:17px;}
 .update-versions>div{display:flex;flex:1;flex-direction:column;gap:6px;}.update-versions span{font-size:11px;color:#aeb5c1;}.update-versions strong{font-size:25px;font-variant-numeric:tabular-nums;}.update-old{color:#f49a74;}.update-new{color:#8edea3;}.update-arrow{font-size:23px!important;}
 .update-description{color:#c9cfd9;font-size:13px;line-height:1.6;}
 .update-notes{margin-top:18px;max-height:170px;overflow:auto;padding:13px;background:#12161b;border-radius:5px;}.update-notes p{white-space:pre-wrap;overflow-wrap:anywhere;font-size:12px;color:#bdc5d1;line-height:1.6;}
 .update-progress{margin-top:20px;}.update-progress>div{display:flex;justify-content:space-between;font-size:12px;margin-bottom:9px;}progress{width:100%;height:8px;appearance:none;border:0;border-radius:6px;overflow:hidden;}progress::-webkit-progress-bar{background:#353b44;}progress::-webkit-progress-value{background:#ffe500;transition:width .1s;}
 .update-error{margin-top:16px;padding:12px;border:1px solid #c16e5944;background:#c16e5912;color:#f0ab94;line-height:1.5;font-size:12px;}
 .update-actions{display:flex;justify-content:flex-end;gap:10px;margin-top:24px;}
</style>
