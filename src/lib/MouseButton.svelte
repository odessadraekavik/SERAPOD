<script>
 let {button='left',label=''}=$props();
</script>
<svg width="17" height="23" viewBox="0 0 20 28" role="img" aria-label={label}>
 <path d={button==='left'?'M9 2C4 2 2 5 2 10v2h7Z':'M11 2c5 0 7 3 7 8v2h-7Z'} fill="#ffe500"/>
 <rect x="1.5" y="1.5" width="17" height="25" rx="8" fill="none" stroke="currentColor" stroke-width="1.5"/>
 <path d="M10 2v10M2 12h16" fill="none" stroke="currentColor" stroke-width="1.3"/>
</svg>
