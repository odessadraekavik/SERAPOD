import {languages} from './i18n.js';
export const arrows={Up:'↑',Right:'→',Down:'↓',Left:'←'};
export const uid=()=>crypto.randomUUID();
export const sequence=code=>code.map(x=>arrows[x]).join(' ');
export const defaults={language:'auto',glass:true,transparency:35,trigger:'F1',gameKey:'ControlLeft',mode:'hold',up:'ArrowUp',down:'ArrowDown',left:'ArrowLeft',right:'ArrowRight',pressMs:45,gapMs:35,size:560,deadzone:70,sensitivity:1};
export const keyOptions=['F1','F2','F3','F4','F5','F6','F7','F8','F9','F10','F11','F12','Mouse4','Mouse5','ControlLeft','ControlRight','AltLeft','AltRight','ShiftLeft','ShiftRight','Tab','Space','ArrowUp','ArrowDown','ArrowLeft','ArrowRight',...'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('').map(x=>'Key'+x),...'0123456789'.split('').map(x=>'Digit'+x)];
export const keyLabel=s=>s.replace('Control','Ctrl ').replace('Left','G').replace('Right','D').replace('Shift','Maj ').replace('Key','').replace('Digit','').replace('ArrowUp','↑').replace('ArrowDown','↓').replace('ArrowG','←').replace('ArrowD','→').replace('Mouse4','Souris 4').replace('Mouse5','Souris 5');
export function initialProfile(catalog){
 const item=name=>{const c=catalog.find(x=>x.name.toLowerCase()===name.toLowerCase())||catalog.find(x=>x.name.toLowerCase().endsWith(' '+name.toLowerCase()));return c?{id:uid(),kind:'stratagem',stratagemId:c.id}:null;};
 const mission={id:uid(),kind:'folder',name:'Mission',nameKey:'mission',children:['NUX-223 Hellbomb','Super Earth Flag','SoS Beacon','SEAF Artillery'].map(item).filter(Boolean)};
 return {id:uid(),name:'Main loadout',nameKey:'defaultProfile',root:{id:uid(),kind:'folder',name:'Main wheel',nameKey:'mainWheel',children:[item('Reinforce'),item('Resupply'),item('Orbital Precision Strike'),item('Orbital Railcannon Strike'),item('B-1 Supply Pack'),item('A/MG-43 Machine Gun Sentry'),mission].filter(Boolean)}};
}
export function getFolder(root,path){let current=root;for(const id of path){const next=current.children.find(x=>x.id===id&&x.kind==='folder');if(!next)break;current=next;}return current;}
export function sectorAt(x,y,count,deadzone=70){if(!count||Math.hypot(x,y)<deadzone)return -1;const step=2*Math.PI/count;return Math.floor(((Math.atan2(x,-y)+step/2+Math.PI*2)%(Math.PI*2))/step)%count;}
export function sectorPath(index,count,outer=254,inner=92){
 const step=2*Math.PI/count,start=index*step-step/2,end=start+step;
 // Offset both radial edges by three SVG units: a constant six-unit gutter.
 const outerGap=Math.asin(3/outer),innerGap=Math.asin(3/inner);
 const point=(r,a)=>`${280+r*Math.sin(a)},${280-r*Math.cos(a)}`;
 return `M${point(outer,start+outerGap)} A${outer},${outer} 0 ${step-2*outerGap>Math.PI?1:0} 1 ${point(outer,end-outerGap)} L${point(inner,end-innerGap)} A${inner},${inner} 0 ${step-2*innerGap>Math.PI?1:0} 0 ${point(inner,start+innerGap)} Z`;
}
export function validateSettings(s){
 if(!s||!['auto',...Object.keys(languages)].includes(s.language??'auto'))return 'invalidLanguage';
 if(s.glass!==undefined && typeof s.glass!=='boolean')return 'invalidRange';
 if(s.transparency!==undefined && (!Number.isFinite(s.transparency)||s.transparency<0||s.transparency>80))return 'invalidRange';
 const keys=['trigger','gameKey','up','down','left','right'];
 if(keys.some(k=>!keyOptions.includes(s[k])))return 'invalidKey';
 if(keys.slice(1).some(k=>s[k].startsWith('Mouse')))return 'keyboardOnly';
 if(new Set(keys.map(k=>s[k])).size!==keys.length)return 'duplicateKeys';
 if(!['hold','toggle'].includes(s.mode))return 'invalidMode';
 for(const [key,min,max] of [['pressMs',15,250],['gapMs',15,250],['size',400,720],['deadzone',30,100],['sensitivity',0.3,3]])if(!Number.isFinite(s[key])||s[key]<min||s[key]>max)return 'invalidRange';
 return '';
}
export function validateState(data,catalog){
 if(!data||data.version!==1||!Array.isArray(data.profiles)||!data.profiles.length||data.profiles.length>32)throw Error('invalidFile');
 const err=validateSettings(data.settings);if(err)throw Error(err);
 const ids=new Set(catalog.map(x=>x.id));const seen=new Set();let total=0;
 function visit(n,depth){
  if(!n||typeof n.id!=='string'||seen.has(n.id)||++total>2000||depth>8)throw Error('invalidTree');seen.add(n.id);
  if(n.kind==='folder'){if(typeof n.name!=='string'||!n.name.trim()||n.name.length>80||!Array.isArray(n.children)||n.children.length>12)throw Error('invalidFolder');n.children.forEach(c=>visit(c,depth+1));}
  else if(n.kind!=='stratagem'||!ids.has(n.stratagemId))throw Error('unknownStratagem');
 }
 const profileIds=new Set();for(const p of data.profiles){if(typeof p.id!=='string'||profileIds.has(p.id)||typeof p.name!=='string'||!p.name.trim()||p.name.length>80||p.root?.kind!=='folder')throw Error('invalidProfile');profileIds.add(p.id);visit(p.root,0);}
 if(!profileIds.has(data.activeId))throw Error('missingProfile');return data;
}

