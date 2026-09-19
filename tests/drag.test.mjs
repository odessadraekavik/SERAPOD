import test from 'node:test';
import assert from 'node:assert/strict';
import {dropEntry,wheelDropTarget} from '../src/lib/drag.js';
const item=id=>({id,kind:'stratagem',stratagemId:id});
const folder=(id,children=[])=>({id,kind:'folder',name:id,children});
test('catalog replaces a sector even when full; existing entries swap without loss',()=>{
 const root=folder('root',Array.from({length:12},(_,i)=>item('i'+i)));
 assert.equal(dropEntry(root,{entry:item('new')},'root',1,'on'),'');
 assert.equal(root.children.length,12);assert.equal(root.children[1].id,'new');
 const original=root.children.map(n=>n.id);
 assert.equal(dropEntry(root,{nodeId:'i0'},'root',5,'on'),'');
 assert.equal(root.children[0].id,'i5');assert.equal(root.children[5].id,'i0');
 assert.deepEqual(root.children.map(n=>n.id).sort(),original.sort());
 assert.equal(dropEntry(root,{nodeId:'i0'},'root',5,'on'),'');
 assert.equal(root.children.length,12);
});
test('trash cancels catalog additions and removes only the dragged existing entry',()=>{
 const root=folder('root',[item('a'),item('b')]);
 assert.equal(dropEntry(root,{entry:item('new')},null,0,'trash'),'');
 assert.equal(root.children.length,2);
 assert.equal(dropEntry(root,{nodeId:'a'},null,0,'trash'),'');
 assert.deepEqual(root.children.map(n=>n.id),['b']);
 assert.equal(dropEntry(root,{nodeId:'root'},null,0,'trash'),'invalidTree');
});
test('wheel target distinguishes interiors, insertion boundaries and cancellation',()=>{
 assert.deepEqual(wheelDropTarget(0,-174,6),{index:0,mode:'on'});
 assert.deepEqual(wheelDropTarget(87,-174*Math.cos(Math.PI/6),6),{index:1,mode:'insert'});
 assert.equal(wheelDropTarget(0,0,6),null);
 assert.equal(wheelDropTarget(300,0,6),null);
 assert.deepEqual(wheelDropTarget(0,0,0),{index:0,mode:'insert'});
});
test('reorder forward and backward uses insertion boundaries without duplication',()=>{
 const root=folder('root',['a','b','c'].map(item));
 assert.equal(dropEntry(root,{nodeId:'a'},'root',3),'');
 assert.deepEqual(root.children.map(n=>n.id),['b','c','a']);
 assert.equal(dropEntry(root,{nodeId:'a'},'root',0),'');
 assert.deepEqual(root.children.map(n=>n.id),['a','b','c']);
});
test('catalog additions and moves into and out of folders preserve identity',()=>{
 const f=folder('f'),root=folder('root',[item('a'),f]);
 assert.equal(dropEntry(root,{nodeId:'a'},'f',0),'');
 assert.equal(dropEntry(root,{entry:item('b')},'f',0),'');
 assert.deepEqual(f.children.map(n=>n.id),['b','a']);
 assert.equal(dropEntry(root,{nodeId:'a'},'root',0),'');
 assert.equal(root.children[0].id,'a');assert.equal(f.children.length,1);
});
test('cycles, full destinations and excessive depth reject atomically',()=>{
 const child=folder('child'),f=folder('f',[child]),full=folder('full',Array.from({length:12},(_,i)=>item('i'+i))),root=folder('root',[f,full]);
 const before=JSON.stringify(root);
 assert.equal(dropEntry(root,{nodeId:'f'},'child',0),'dropCycle');
 assert.equal(dropEntry(root,{nodeId:'f'},'full',0),'maxSectors');
 assert.equal(JSON.stringify(root),before);
 let chain=folder('deep');const deep=chain;for(let i=0;i<8;i++){const next=folder('depth'+i);chain.children=[next];chain=next;}
 assert.equal(dropEntry(deep,{entry:item('new')},chain.id,0),'maxDepth');
});
