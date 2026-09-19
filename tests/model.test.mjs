import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {sectorAt,defaults,initialProfile,validateState,validateSettings,getFolder,sectorPath,mouseCode,keyOptions} from '../src/lib/model.js';
const catalog=JSON.parse(await readFile(new URL('../src/data/catalog.json',import.meta.url),'utf8'));
test('timing limits and legacy imports preserve profiles when upgrading fast delays',()=>{
 assert.equal(defaults.gapMs,150);
 for(const gapMs of [100,150,300])assert.equal(validateSettings({...defaults,gapMs}),'');
 for(const gapMs of [99,301,NaN])assert.equal(validateSettings({...defaults,gapMs}),'invalidRange');
 const p=initialProfile(catalog);
 for(const gapMs of [15,35,95,100,200,250,300]){
  const data={version:1,profiles:[p],activeId:p.id,settings:{...defaults,gapMs}};
  const restored=validateState(data,catalog);
  assert.equal(restored.settings.gapMs,gapMs<100?150:gapMs);
  assert.equal(restored.profiles[0],p);
  assert.equal(restored.activeId,p.id);
 }
 for(const gapMs of [-1,14,301])assert.throws(()=>validateState({version:1,profiles:[p],activeId:p.id,settings:{...defaults,gapMs}},catalog));
});
test('catalog codes are complete, unique and include known reference sequences',()=>{
 assert.ok(catalog.length>=100);assert.equal(new Set(catalog.map(c=>c.id)).size,catalog.length);
 for(const c of catalog){assert.ok(c.name);assert.ok(c.code.length>0);assert.ok(c.code.every(d=>['Up','Down','Left','Right'].includes(d)));}
 assert.deepEqual(catalog.find(c=>c.name==='Reinforce').code,['Up','Down','Right','Left','Up']);
 assert.deepEqual(catalog.find(c=>c.name==='Orbital Precision Strike').code,['Right','Right','Up']);
});
test('radial geometry handles cardinal directions, wraparound, center and empty menus',()=>{
 assert.equal(sectorAt(0,-100,4),0);assert.equal(sectorAt(100,0,4),1);assert.equal(sectorAt(0,100,4),2);assert.equal(sectorAt(-100,0,4),3);
 assert.equal(sectorAt(-.001,-100,4),0);assert.equal(sectorAt(40,40,4),-1);assert.equal(sectorAt(100,100,0),-1);assert.equal(sectorAt(100,0,1),0);
 assert.ok(!sectorPath(0,1).includes('NaN'));
});
test('sector gutters have parallel edges with constant width at both radii',()=>{
 for(const count of [2,3,6,12]){
  const points=[...sectorPath(0,count).matchAll(/([\d.]+),([\d.]+)/g)].map(m=>[Number(m[1])-280,Number(m[2])-280]);
  const boundary=-Math.PI/count;
  for(const [x,y] of [points[0],points.at(-1)])assert.ok(Math.abs(x*Math.cos(boundary)+y*Math.sin(boundary)-3)<1e-8);
 }
});
test('profile import validates references, settings, sizes and duplicate identities',()=>{
 const p=initialProfile(catalog);const data={version:1,profiles:[p],activeId:p.id,settings:{...defaults}};
 assert.equal(validateState(data,catalog),data);
 assert.equal(p.root.children.length,7);
 assert.deepEqual(p.root.children.filter(n=>n.kind==='stratagem').map(n=>catalog.find(c=>c.id===n.stratagemId).name),['Reinforce','Resupply','Orbital Precision Strike','Orbital Railcannon Strike','B-1 Supply Pack','A/MG-43 Machine Gun Sentry']);
 assert.deepEqual(p.root.children.find(n=>n.nameKey==='mission').children.map(n=>catalog.find(c=>c.id===n.stratagemId).name),['NUX-223 Hellbomb','Super Earth Flag','SoS Beacon','SEAF Artillery']);
 assert.equal(p.nameKey,'defaultProfile');
 const bad=structuredClone(data);bad.profiles[0].root.children.push({...bad.profiles[0].root.children[0]});assert.throws(()=>validateState(bad,catalog));
 const badRef=structuredClone(data);badRef.profiles[0].root.children[0].stratagemId='unknown';assert.throws(()=>validateState(badRef,catalog));
 assert.ok(validateSettings({...defaults,trigger:'ControlLeft'}));assert.ok(validateSettings({...defaults,pressMs:-1}));
 assert.equal(getFolder(p.root,['missing']),p.root);
});

test('mouse shortcuts reserve primary buttons and map browser button indices',()=>{
 assert.equal(mouseCode(0),null);assert.equal(mouseCode(2),null);assert.equal(mouseCode(5),null);
 for(const [button,key] of [[1,'Mouse3'],[3,'Mouse4'],[4,'Mouse5']]){
  assert.equal(mouseCode(button),key);assert.ok(keyOptions.includes(key));
  assert.equal(validateSettings({...defaults,trigger:key}),'');
  assert.equal(validateSettings({...defaults,gameKey:key}),'keyboardOnly');
 }
 for(const trigger of ['Mouse1','Mouse2','Mouse6'])assert.equal(validateSettings({...defaults,trigger}),'invalidKey');
});
