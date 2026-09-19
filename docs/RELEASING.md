# Publishing a SERAPOD release

1. Bump SemVer in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`. Keep `Cargo.lock` current.
2. Run `./build.ps1` on Windows. Commit source, tests, documentation, and lockfiles.
3. Create tag `v<version>` at that commit and a draft GitHub release in `odessadraekavik/SERAPOD`.
4. Attach `portable/SERAPOD-<version>.exe` and `SHA256SUMS.txt`. Never rename an executable from another version.
5. Check the release asset's `digest` in GitHub's Releases API. The updater requires `sha256:<64 hex characters>`, the exact file name, and a stable SemVer newer than the running app. Publish as latest after attaching the executable.
6. Verify the version prompt, No, download, progress, restart, preservation of profiles, and deletion of the previous executable in a disposable writable directory.

The updater only accepts this repository's HTTPS release assets. Network failures do not block startup. Download/install requires confirmation. Existing executables with different contents are never overwritten. The previous live process's versioned executable is removed only after the replacement frontend loads. Published release binaries must remain immutable.

The executable is not Authenticode-signed. SHA-256 checks the download against GitHub's asset digest; it is not an independent publisher signature. Protect GitHub release permissions accordingly.

The source archive at the release tag contains the corresponding source and build instructions. No private credentials are needed to build or use SERAPOD.
