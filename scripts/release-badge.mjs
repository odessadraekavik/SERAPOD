import {readFile, writeFile} from 'node:fs/promises';

const {version}=JSON.parse(await readFile(new URL('../package.json',import.meta.url),'utf8'));
if(!/^\d+\.\d+\.\d+$/.test(version))throw new Error('Expected a stable release version');
const rightWidth=Math.max(45,Math.ceil((version.length+1)*7+12));
const width=75+rightWidth;
await writeFile(new URL('../medias/release-badge.svg',import.meta.url),`<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="20" role="img" aria-label="deployment: v${version}">
  <title>deployment: v${version}</title>
  <clipPath id="r"><rect width="${width}" height="20" rx="3"/></clipPath>
  <g clip-path="url(#r)"><path fill="#555" d="M0 0h75v20H0z"/><path fill="#ffe500" d="M75 0h${rightWidth}v20H75z"/></g>
  <g text-anchor="middle" font-family="Verdana,DejaVu Sans,sans-serif" font-size="11"><text x="37.5" y="14" fill="#fff">deployment</text><text x="${75+rightWidth/2}" y="14" fill="#333">v${version}</text></g>
</svg>\n`);
