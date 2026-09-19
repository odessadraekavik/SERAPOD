// All checks happen before mutation, so a rejected drop never loses an entry.
export function findNode(root, id, parent = null, depth = 0) {
 if (root.id === id) return {node:root,parent,depth};
 for (const child of root.children || []) { const found=findNode(child,id,root,depth+1); if(found)return found; }
}
const height = node => node.kind==='folder' && node.children.length ? 1+Math.max(...node.children.map(height)) : 0;
export function dropEntry(root, payload, targetId, index, mode='insert') {
 if(mode==='trash'){
  if(!payload.nodeId)return '';
  const source=findNode(root,payload.nodeId);
  if(!source?.parent)return 'invalidTree';
  source.parent.children.splice(source.parent.children.indexOf(source.node),1);
  return '';
 }
 const target=findNode(root,targetId);
 if(!target || target.node.kind!=='folder')return 'invalidFolder';
 const source=payload.nodeId?findNode(root,payload.nodeId):null;
 const node=source?.node || payload.entry;
 if(!node || (payload.nodeId && !source?.parent))return 'invalidTree';
 if(mode==='on'){
  const destination=target.node.children[index];
  if(node.kind!=='stratagem'||destination?.kind!=='stratagem')return 'invalidTree';
  if(source){
   const old=source.parent.children.indexOf(node);
   source.parent.children[old]=destination;
  }
  target.node.children[index]=node;
  return '';
 }
 if(findNode(node,targetId))return 'dropCycle';
 if(target.depth+1+height(node)>8)return 'maxDepth';
 if(target.node.children.length>=12 && source?.parent!==target.node)return 'maxSectors';
 let position=Math.max(0,Math.min(index,target.node.children.length));
 if(source){const old=source.parent.children.indexOf(node);if(source.parent===target.node && old<position)position--;source.parent.children.splice(old,1);}
 target.node.children.splice(position,0,node);
 return '';
}
// Sector interiors replace/swap; narrow boundary zones keep insertion available.
export function wheelDropTarget(x,y,count){
 const radius=Math.hypot(x,y);
 if(radius>264 || (count && radius<92))return null;
 if(!count)return {index:0,mode:'insert'};
 const step=2*Math.PI/count,angle=(Math.atan2(x,-y)+2*Math.PI)%(2*Math.PI);
 const sector=Math.floor((angle+step/2)/step)%count;
 const boundary=(angle/step+.5);
 const nearBoundary=Math.abs(boundary-Math.round(boundary))*step<Math.min(.09,step*.12);
 return nearBoundary?{index:Math.round(boundary)%count,mode:'insert'}:{index:sector,mode:'on'};
}
