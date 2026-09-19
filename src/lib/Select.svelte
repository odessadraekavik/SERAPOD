<script>
 import {tick} from 'svelte';
 let {value=$bindable(),options=[],label='',id,variant='',onchange=()=>{}}=$props();
 const unique=$props.id();
 let open=$state(false),active=$state(0),trigger=$state(),container=$state(),popup=$state();
 let position=$state({left:0,top:0,width:200,height:280});
 let typed='',typedAt=0;
 const selected=$derived(options.find(option=>option.value===value));
 async function reveal(){await tick();popup?.querySelector('[data-active="true"]')?.scrollIntoView({block:'nearest'});}
 async function show(){
  const r=trigger.getBoundingClientRect(),below=window.innerHeight-r.bottom-12,above=r.top-12;
  const height=Math.min(280,Math.max(below,above));
  const width=Math.min(Math.max(r.width,180),window.innerWidth-16);
  position={left:Math.max(8,Math.min(r.left,window.innerWidth-width-8)),top:below>=Math.min(280,options.length*38+10)?r.bottom+6:Math.max(8,r.top-Math.min(height,options.length*38+10)-6),width,height};
  active=Math.max(0,options.findIndex(option=>option.value===value));open=true;typed='';await reveal();
 }
 function choose(index){if(!options[index])return;value=options[index].value;open=false;onchange();trigger?.focus();}
 function outside(e){if(!container?.contains(e.target))open=false;}
 function keydown(e){
  e.stopPropagation();
  if(e.key==='Tab'){open=false;return;}
  if(e.key==='Escape'){if(open)e.preventDefault();open=false;return;}
  if(['ArrowDown','ArrowUp','Home','End','Enter',' '].includes(e.key)){
   e.preventDefault();
   if(!open){show();return;}
   if(e.key==='Enter'||e.key===' '){choose(active);return;}
   active=e.key==='Home'?0:e.key==='End'?options.length-1:(active+(e.key==='ArrowDown'?1:-1)+options.length)%options.length;reveal();
  }else if(e.key.length===1){
   e.preventDefault();const now=Date.now();typed=now-typedAt>700?e.key:typed+e.key;typedAt=now;
   const index=options.findIndex(option=>option.label.toLocaleLowerCase().startsWith(typed.toLocaleLowerCase()));
   if(index>=0){if(open){active=index;reveal();}else choose(index);}
  }
 }
</script>
<svelte:window onpointerdown={outside} onresize={()=>open=false} onscroll={()=>open=false} onblur={()=>open=false}/>
<div class={`dropdown ${variant}`} bind:this={container}>
 <button type="button" id={id||unique} class="dropdown-trigger" class:open bind:this={trigger} role="combobox" aria-label={label} aria-haspopup="listbox" aria-expanded={open} aria-controls={`${unique}-list`} aria-activedescendant={open?`${unique}-${active}`:undefined} onclick={()=>open?open=false:show()} onkeydown={keydown}>
  <span>{selected?.label||'—'}</span><svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path d="m4 6 4 4 4-4"/></svg>
 </button>
 {#if open}<div class="dropdown-popup" bind:this={popup} id={`${unique}-list`} role="listbox" aria-label={label} style:left={`${position.left}px`} style:top={`${position.top}px`} style:width={`${position.width}px`} style:max-height={`${position.height}px`}>
  {#each options as option,i}<button type="button" role="option" id={`${unique}-${i}`} aria-selected={option.value===value} tabindex="-1" class="dropdown-option" class:active={active===i} class:chosen={option.value===value} data-active={active===i} onpointerdown={e=>e.preventDefault()} onpointermove={()=>active=i} onclick={()=>choose(i)}><span>{option.label}</span>{#if option.value===value}<span aria-hidden="true">✓</span>{/if}</button>{/each}
 </div>{/if}
</div>
