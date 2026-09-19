$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath $PSScriptRoot
$nodeCommand = Get-Command node -ErrorAction SilentlyContinue
if (-not $nodeCommand) { throw 'Node.js est requis pour compiler SERAPOD.' }
$nodePath = $nodeCommand.Source
$releaseVersion = (Get-Content -Raw src-tauri/tauri.conf.json | ConvertFrom-Json).version
$packageVersion = (Get-Content -Raw package.json | ConvertFrom-Json).version
$cargoVersion = [regex]::Match((Get-Content -Raw src-tauri/Cargo.toml), '(?m)^version = "([^"]+)"').Groups[1].Value
if ($releaseVersion -notmatch '^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$' -or $releaseVersion -ne $packageVersion -or $releaseVersion -ne $cargoVersion) { throw 'Versions incoherentes entre les manifestes.' }
if (-not (Test-Path -LiteralPath 'node_modules')) { throw 'Installer les dépendances avec pnpm install avant de compiler.' }
& $nodePath node_modules/svelte-check/bin/svelte-check --tsconfig jsconfig.json
if ($LASTEXITCODE -ne 0) { throw 'Vérification Svelte échouée.' }
& $nodePath --test tests/*.test.mjs
if ($LASTEXITCODE -ne 0) { throw 'Tests JavaScript échoués.' }
& $nodePath node_modules/vite/bin/vite.js build
if ($LASTEXITCODE -ne 0) { throw "Compilation de l'interface échouée." }
cargo test --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) { throw 'Tests Rust échoués.' }
cargo build --manifest-path src-tauri/Cargo.toml --release --features custom-protocol
if ($LASTEXITCODE -ne 0) { throw 'Compilation Windows échouée.' }
New-Item -ItemType Directory -Force portable | Out-Null
$releasePath = "portable/SERAPOD-$releaseVersion.exe"
Copy-Item -LiteralPath 'src-tauri/target/release/serapod.exe' -Destination $releasePath
Write-Host "Ready: $releasePath"
