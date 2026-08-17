# Design system (dark daidai-orange) — not yet adopted

This directory is a hand-ported Svelte translation of the "Deslopper Design System" built in
Claude Design (project `c828b348-1fcc-4b56-866e-94b8763c0f56`; see
`.claude/skills/deslopper-design/` for the imported skill contract). It exists so real Svelte
components implementing that visual language are available in this repo, ready to ship — but
**it is not wired into the live app.** `frontend/src/routes/+page.svelte` still imports
`$lib/theme.css` and is unaffected by anything here.

## Why this is separate from the shipped app

`docs/ui-design-system.md` (decision `D-002` in `docs/decision-log.md`) is this repo's accepted
visual authority: stock-Android-inspired Material, light theme first, indigo accent, and it
explicitly forbids "neon, glassmorphism, gamer styling." The Claude Design project this library
was ported from was deliberately commissioned with a **different** brief — a dark
Razer-Synapse-style chrome with daidai (橙) orange accents — and says so in its own readme. That
is a second, competing visual language, which `D-002` says requires a new decision before it
governs any shipped surface. See `docs/decision-log.md` (`D-022`) for that record: this library's
existence is accepted, but adopting it in place of the current `+page.svelte` styling is a
separate, not-yet-made decision.

## What's here

- `tokens/` — `fonts.css`, `colors.css` (dark default + `[data-theme="light"]`), `typography.css`,
  `spacing.css`, `elevation.css`, `base.css` (scoped under `.ds-root`, see below).
- `components.css` — the component class implementations (`.ds-btn`, `.ds-badge`, `.ds-surface`,
  etc.), ported verbatim from the source project's `components/ds-components.css`.
- `index.css` — imports all of the above in order. Import this once, from wherever a page mounts
  these components.
- `components/<group>/<Name>.svelte` — 26 components across `core/`, `feedback/`, `forms/`,
  `layout/`, `navigation/`, translated 1:1 from the source project's React/JSX components and CSS
  class names (`Button` → `.ds-btn`, `Surface` → `.ds-surface`, etc.). Props mirror the original
  API (see `.claude/skills/deslopper-design/_adherence.oxlintrc.json` for the authoritative prop
  lists). React children became Svelte default/named slots; `...rest` spreads became
  `{...$$restProps}`.
- `index.ts` — barrel export, e.g. `import { Button, Surface } from '$lib/design-system';`.

## Using it

Wrap the subtree that uses these components in an element with `class="ds-root"` — the base
reset in `tokens/base.css` is scoped to `.ds-root` rather than global `body`/`*` so it can't leak
into or fight the shipped app's own `$lib/theme.css` reset if both are ever loaded on the same
page:

```svelte
<script lang="ts">
  import '$lib/design-system/index.css';
  import { Button, Surface, Badge } from '$lib/design-system';
</script>

<div class="ds-root">
  <Surface>
    <Badge tone="success">Observed</Badge>
    <Button variant="primary">Run a fresh inspection</Button>
  </Surface>
</div>
```

Dark is the default surface; add `data-theme="light"` on an ancestor (e.g. `<div class="ds-root" data-theme="light">`)
to get the shipped app's original paper palette instead.

## Known gaps versus the source project

- **No local webfonts.** The source project's `tokens/fonts.css` loads Archivo, IBM Plex Sans, and
  IBM Plex Mono from a Google Fonts CDN `@import`. `docs/ui-design-system.md` forbids remote CDN
  loading for production assets, so that import was dropped here. Until real font files are
  bundled locally, these fall back to the system fonts listed in each `--font-*` stack.
- **Not integrated into `+page.svelte`.** Swapping the live app over to this visual language —
  or offering it as a user-selectable theme — is a separate, larger change this PR does not make.
- **No dedicated component tests.** These are presentational wrappers with little logic of their
  own; the ones with real branching (`StatusText`'s tone lookup, `CountChip`'s value/no-value
  branch, `Surface`/`SectionHeading`'s dynamic tag) were verified through `svelte-check` and a
  manual read, not automated tests. Adding real render tests would need a Svelte testing
  dependency (`@testing-library/svelte` or similar) that isn't in this repo yet — that's a
  `docs/dependency-policy.md` decision, not something to add silently here.
- **Guideline cards, the UI kit, and assets were not ported.** Only the 26 primitive components and
  the token/component CSS made the trip. `guidelines/*.card.html`, `ui_kits/deslopper-app/`, and
  `assets/` still live only in the source Claude Design project.
