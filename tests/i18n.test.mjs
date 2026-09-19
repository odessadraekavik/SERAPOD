import test from 'node:test';
import assert from 'node:assert/strict';
import {readFile,access} from 'node:fs/promises';
import {messages,resolveLocale,translate} from '../src/lib/i18n.js';
import {validateState,initialProfile,defaults} from '../src/lib/model.js';
const catalog=JSON.parse(await readFile(new URL('../src/data/catalog.json',import.meta.url),'utf8'));
test('fresh profile labels follow the system language',()=>{
 const p=initialProfile(catalog);
 assert.equal(translate(resolveLocale(defaults.language,'en-US'),p.nameKey),'Main loadout');
 assert.equal(translate(resolveLocale(defaults.language,'fr-FR'),p.nameKey),'Équipement principal');
 assert.equal(translate('en',p.root.nameKey),'Main wheel');
});
test('automatic language resolution, explicit overrides and fallback',()=>{
 assert.equal(resolveLocale('auto','fr-FR'),'fr');assert.equal(resolveLocale('auto','fr_CA'),'fr');
 assert.equal(resolveLocale('auto','en-US'),'en');assert.equal(resolveLocale('auto','de-DE'),'en');
 assert.equal(resolveLocale('en','fr-FR'),'en');assert.equal(resolveLocale('fr','en-US'),'fr');
});
test('translations have complete keys and matching parameter placeholders',()=>{
 assert.deepEqual(Object.keys(messages.fr).sort(),Object.keys(messages.en).sort());
 const tokens=s=>(s.match(/\{\w+\}/g)||[]).sort();
 for(const key of Object.keys(messages.en)){assert.ok(messages.fr[key].trim());assert.deepEqual(tokens(messages.fr[key]),tokens(messages.en[key]),key);}
 assert.equal(translate('fr','added',{name:'Test'}),'Test ajouté.');
 assert.equal(translate('de','ready'),'Ready');
});
test('old profiles without a language preference still load',()=>{
 const p=initialProfile(catalog);const settings={...defaults};delete settings.language;
 assert.doesNotThrow(()=>validateState({version:1,profiles:[p],activeId:p.id,settings},catalog));
});
test('all four wiki direction assets are packaged locally',async()=>{
 for(const direction of ['Up','Down','Left','Right'])await access(new URL(`../public/icons_arrows/Stratagem_Arrow_${direction}.svg`,import.meta.url));
});
