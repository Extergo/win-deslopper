repo: Extergo/win-deslopper
branch: main

## Last sync

date: 2026-08-17T01:51:00Z

### Updated in this project

- Built the full design system from the shipped app: tokens, 25 components, 17 foundation cards.
- Recreated the Owner Mode desktop app as an interactive UI kit (6 sections, apply/undo flows).
- Re-themed to a dark Razer-Synapse-style chrome with daidai orange; light mode keeps the shipped palette.
- Copied `icons/app-icon.svg` and `icons/icon.ico` into `assets/` (the only real brand assets in the repo).

## Screen map

| Project file | Built from |
| --- | --- |
| `tokens/*.css` | `frontend/src/lib/theme.css`, `docs/ui-design-system.md` |
| `components/ds-components.css` | `frontend/src/routes/+page.svelte` (style block) |
| `components/**` | `frontend/src/routes/+page.svelte` markup, `frontend/src/lib/product-ui.ts` |
| `ui_kits/deslopper-app/Shell.jsx` | `+page.svelte` sidebar + overview section |
| `ui_kits/deslopper-app/ComponentsScreen.jsx` | `+page.svelte` components section, `src/platform.rs` |
| `ui_kits/deslopper-app/Screens.jsx` | `+page.svelte` desired/drift/history/settings sections |
| `ui_kits/deslopper-app/data.js` | `src/platform.rs` (`v1_catalogue`), `docs/detector-matrix.md` |
| `readme.md` (content + visual foundations) | `docs/product-principles.md`, `docs/read-only-product-alpha.md`, `docs/ui-design-system.md` |
| `assets/logo.svg`, `assets/icon.ico` | `icons/app-icon.svg`, `icons/icon.ico` |
