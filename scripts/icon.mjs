import { spawnSync } from 'node:child_process';

// Generate the executable icon from the same radial logo as the interface.
const result = spawnSync(process.execPath, [
  'node_modules/@tauri-apps/cli/tauri.js',
  'icon', 'public/logo.svg', '--output', 'src-tauri/icons',
], { stdio: 'inherit' });
if (result.error) throw result.error;
process.exitCode = result.status ?? 1;
