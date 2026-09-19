<script>
 import {translate} from './i18n.js';
 let {code=[],locale='en',small=false}=$props();
</script>
<span class="code-arrows" class:small role="img" aria-label={code.map(d=>translate(locale,d.toLowerCase())).join(', ')}>
 {#each code as direction}<img src={`/icons_arrows/Stratagem_Arrow_${direction}.svg`} alt="" draggable="false"/>{/each}
</span>
