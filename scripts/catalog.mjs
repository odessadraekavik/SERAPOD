import {load} from 'cheerio';
import {readFile,writeFile,mkdir,copyFile,readdir} from 'node:fs/promises';
import path from 'node:path';
const $=load(await readFile('wiki_stratagems.html','utf8'));
const catalog=[];
let category='Mission Stratagems';
const slug=s=>s.toLowerCase().replace(/[^a-z0-9]+/g,'-').replace(/^-|-$/g,'');
$('summary, table.wikitable').each((_,el)=>{
  if(el.tagName==='summary'){category=$(el).text().trim();return;}
  $(el).find('tbody > tr').each((_,row)=>{
    const cells=$(row).children('td');
    if(cells.length<3)return;
    const code=cells.eq(2).find('img').map((_,img)=>($(img).attr('alt')||'').match(/Stratagem Arrow (Up|Down|Left|Right)/)?.[1]).get();
    if(!code.length)return;
    const name=cells.eq(1).text().replace(/\s+/g,' ').trim();
    const id=slug(name);
    if(!name||catalog.some(x=>x.id===id))return;
    catalog.push({id,name,category,code,icon:'/icons/fallback.svg',source:'https://helldivers.wiki.gg/wiki/Stratagems',wikiIcon:cells.eq(0).find('img').attr('src')||'',cooldown:cells.eq(3).text().trim()});
  });
});
// Supplement older wiki snapshots with the September 2026 vehicle addition.
if(!catalog.some(item=>item.id==='td-110-maelstrom'))catalog.push({id:'td-110-maelstrom',name:'TD-110 Maelstrom',category:'Vehicles',code:['Left','Down','Right','Down','Left','Down','Up','Left','Right'],icon:'/icons/td-110-maelstrom.svg',source:'https://helldivers.wiki.gg/wiki/TD-110_Maelstrom',wikiIcon:'',cooldown:'780s'});
await mkdir('public/icons',{recursive:true});
await mkdir('src/data',{recursive:true});
// Icons are downloaded once by setup; never fetched while playing.
const root='.tools/icons/Helldivers-2-Stratagems-icons-svg-master';
const normalize=s=>s.toLowerCase().replace(/stratagem|icon|background/g,'').replace(/[^a-z0-9]/g,'');
let files=[];
async function walk(dir){for(const e of await readdir(dir,{withFileTypes:true})){const p=path.join(dir,e.name);if(e.isDirectory())await walk(p);else if(e.name.endsWith('.svg'))files.push(p);}}
try{await walk(root);}catch{}
const aliases={'b-100-portable-hellbomb':'Hellbomb Portable','ax-tx-13-dog-breath':'Guard Dog Breath','ax-las-5-rover':'Guard Dog Rover','ax-flam-75-hot-dog':'Guard Dog Hot Dog','ax-arc-3-k-9':'Guard Dog K-9','md-8-gas-mines':'Gas Mine','eagle-gas-airstrike':'Eagle Gas Strike','cqc-20-breaching-hammer':'CQC-20','eat-411-leveller':'EAT-411','gl-28-belt-fed-grenade-launcher':'GL-28','aquifer-drill':'Tectonic Drill','activate-e-711-extraction-drill':'Tectonic Drill'};
let matched=0;
for(const item of catalog){
 const candidates=[item.name,aliases[item.id],decodeURIComponent(item.wikiIcon.split('/').pop()?.split('?')[0]||'').replace(/\.svg$/,'')].filter(Boolean).map(normalize);
 const file=files.find(f=>candidates.includes(normalize(path.basename(f,'.svg'))));
 if(file){await copyFile(file,`public/icons/${item.id}.svg`);item.icon=`/icons/${item.id}.svg`;matched++;}
}
// Keep the user-reported in-game correction when regenerating from the wiki snapshot.
const sssd=catalog.find(item=>item.id==='sssd-delivery');
if(sssd)sssd.code=['Down','Down','Down','Up','Up'];
await writeFile('src/data/catalog.json',JSON.stringify(catalog,null,2)+'\n');
console.log(`${catalog.length} stratagems, ${matched} local SVG icons. ${catalog.length-matched} use a fallback.`);
