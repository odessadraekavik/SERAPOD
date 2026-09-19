<script>
 import {sectorPath} from './model.js';
 import {translate} from './i18n.js';
 import Arrows from './Arrows.svelte';
 let {items=[],catalog=[],selected=-1,title='SERAPOD',subtitle='',onselect=()=>{},oncenter=()=>{},size=560,locale='en',interactive=true,glass=false,transparency=35,onitemdown=()=>{},onitemcontext=null,dropIndex=-1,dropFolder=-1,dragging=false,draggedId=null,dropAction='',cursor=null}=$props();
 const lookup=id=>catalog.find(c=>c.id===id);
 const sheenId=$props.id();
 const name=n=>n.kind==='folder'?(n.nameKey?translate(locale,n.nameKey):n.name):lookup(n.stratagemId)?.name||'?';
 let active=$derived(items[selected]);
</script>
<svg class="wheel" class:interactive class:glass class:dragging viewBox="0 0 560 560" style:--panel-alpha={1-transparency/100} style:width={`${size}px`} aria-label={translate(locale,'wheelLabel')}>
 <defs><linearGradient id={sheenId} x1="0" y1="0" x2="1" y2="1"><stop stop-color="#edf6ff" stop-opacity=".18"/><stop offset=".45" stop-color="#cadfff" stop-opacity=".02"/><stop offset="1" stop-color="#b9d8ff" stop-opacity=".10"/></linearGradient></defs>
 <circle cx="280" cy="280" r="264" fill="none" stroke="#34383f"/>
 {#each items as item,i (item.id)}
  {@const angle=i*2*Math.PI/items.length}
  {@const x=280+174*Math.sin(angle)}{@const y=280-174*Math.cos(angle)}
  <g class:selected={selected===i} class:drop-folder={dropFolder===i} class:drag-source={draggedId===item.id} class="sector" role="button" tabindex={interactive?0:-1} aria-label={name(item)} aria-haspopup={onitemcontext?'menu':undefined} oncontextmenu={e=>{if(onitemcontext){e.preventDefault();onitemcontext(e,item);}}} onpointerdown={e=>onitemdown(e,item)} onclick={()=>onselect(i)} onkeydown={e=>{if(e.key==='Enter'||e.key===' '){e.preventDefault();onselect(i);}}}>
   <path d={sectorPath(i,items.length)}/>
   {#if glass}<path d={sectorPath(i,items.length)} fill={`url(#${sheenId})`} pointer-events="none"/>{/if}
   {#if item.kind==='folder'}<path class="folder-symbol" d={`M${x-21},${y-19}h17l5,6h23v28h-45z`}/><text x={x+1.5} y={y+1} text-anchor="middle" dominant-baseline="central" class="folder-count">{item.children.length}</text>
   {:else}<image href={lookup(item.stratagemId)?.icon||'/icons/fallback.svg'} x={x-24} y={y-30} width="48" height="48"/>{/if}
   <text x={x} y={y+35} text-anchor="middle" class="sector-label">{name(item).length>20?name(item).slice(0,18)+'…':name(item)}</text>
   {#if dropFolder===i&&dropAction}<g class="drop-action" pointer-events="none"><rect x={x-47} y={y-62} width="94" height="21" rx="4"/><text x={x} y={y-48} text-anchor="middle">{dropAction}</text></g>{/if}
  </g>
 {/each}
 {#if dropIndex>=0}
  {@const angle=items.length?(dropIndex-.5)*2*Math.PI/items.length:0}
  <line class="drop-separator" x1={280+94*Math.sin(angle)} y1={280-94*Math.cos(angle)} x2={280+260*Math.sin(angle)} y2={280-260*Math.cos(angle)}/>
 {/if}
 <g role="button" tabindex={interactive?0:-1} aria-label={translate(locale,'backCancel')} onclick={oncenter} onkeydown={e=>{if(e.key==='Enter'){e.preventDefault();oncenter();}}} class="wheel-center">
  <circle cx="280" cy="280" r="85" fill="#141619" stroke="#44484f"/>
  <text x="280" y="255" text-anchor="middle" class="center-overline">{active?translate(locale,'selection'):subtitle}</text>
  <foreignObject x="205" y="265" width="150" height="53"><div class="center-name">{active?name(active):title}</div></foreignObject>
  <foreignObject x="200" y="323" width="160" height="22"><div class="center-code">{#if active&&active.kind!=='folder'}<Arrows code={lookup(active.stratagemId)?.code||[]} {locale} small/>{:else}{active?translate(locale,'clickOpen'):translate(locale,'back')}{/if}</div></foreignObject>
 </g>
 {#if cursor}<g class="radial-cursor" transform={`translate(${280+cursor[0]} ${280+cursor[1]})`} pointer-events="none" aria-hidden="true"><circle r="5" fill="#fff" stroke="#11151b" stroke-width="2"/></g>{/if}
</svg>
