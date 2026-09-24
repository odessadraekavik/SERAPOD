# Sources and attribution

- **Helldivers Wiki** — names, categories, and directional sequences extracted from the user-provided copy `wiki_stratagems.html`, revision `133897`: https://helldivers.wiki.gg/wiki/Stratagems?oldid=133897. The data is not guaranteed to be complete or remain accurate after subsequent game updates.
- **nvigneux and contributors**, *Helldivers-2-Stratagems-icons-svg*: https://github.com/nvigneux/Helldivers-2-Stratagems-icons-svg. SVG files downloaded on September 18, 2026, from the master branch. The README permits their use in other projects. The SVG files are stored in `public/icons` and renamed to match the catalogue identifiers. Their original appearance is preserved.
- Shared drill symbols use the Tectonic Drill icon where the wiki also indicates a shared drill icon. Tactical Video Camera uses a generic symbol created for SERAPOD.
- Directional arrows supplied by the user in `public/icons_arrows`, sourced from the Helldivers Wiki. The SVG files carry the credit “Traced by Dogo314”. They are used without modification in the sequences displayed in the catalogue, editor, and wheel.
- Radial logo `public/logo.svg`: vector artwork created for SERAPOD, also adapted into a Windows icon.
- Helldivers names, trademarks, and graphics belong to their respective rights holders. No ownership of these assets is claimed.
- Svelte, Vite, Tauri, and other dependencies retain their respective licenses, available in their distributions.

Implementation references:

- TD-110 Maelstrom catalogue addition: Helldivers Wiki stratagem listing, checked September 24, 2026: https://helldivers.wiki.gg/wiki/Stratagem. Icon added from the nvigneux repository above on the same date.

- Tauri: https://v2.tauri.app/reference/config/
- Windows prerequisites: https://v2.tauri.app/start/prerequisites/
- Microsoft LowLevelMouseProc: https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelmouseproc
- Microsoft SendInput: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput
