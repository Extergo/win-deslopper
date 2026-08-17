# Deslopper design system

**Windows without the slop.**

Deslopper is a privacy-conscious, installable **Windows configuration inspector** with a small set of
reversible current-user controls. It is a Tauri 2 desktop app (Rust backend, SvelteKit frontend) that
runs unelevated, observes a fixed catalogue of documented Windows components, explains uncertainty and
authority, keeps redacted inspection history in a local SQLite database, detects meaningful drift,
records desired states, and produces non-executable previews.

The shipped release is `0.4.1` — **Owner Mode M4**. It observes **21 registered components**, can apply
and exactly undo **six fixed current-user settings**, and can remove **four exact package identities**
from the current account. There is no generic mutation command, filesystem capability, shell endpoint,
network client, updater, elevation manifest, service, or privileged helper.

## Sources this system was built from

| Source | Detail |
| --- | --- |
| GitHub repository | <https://github.com/Extergo/win-deslopper> (branch `main`) |
| Design authority | `docs/ui-design-system.md` |
| Live tokens | `frontend/src/lib/theme.css` |
| Live UI (single view, ~2,850 lines) | `frontend/src/routes/+page.svelte` |
| Copy / status vocabulary | `frontend/src/lib/product-ui.ts` |
| Component catalogue (21 definitions) | `src/platform.rs` (`v1_catalogue()`) |
| Detector rules and limits | `docs/detector-matrix.md` |
| Product contract & privacy | `docs/read-only-product-alpha.md`, `docs/product-principles.md` |
| App icon | `icons/app-icon.svg` → `assets/logo.svg` |

Read those repositories directly for anything this system abbreviates — especially
`+page.svelte` (the authoritative layout) and `docs/detector-matrix.md` (the authoritative status
vocabulary). Nothing in this design system is a substitute for the shipped source.

### Products represented

There is exactly **one product surface**: the installed Windows desktop application. There is no
marketing website, mobile app, docs site, or web console in the repository, so this system contains one
UI kit. An internal, compile-gated `mutation-alpha` engineering panel exists in the repo
(`frontend/src/lib/MutationAlphaPanel.svelte`); it is deliberately **not** recreated here because it is
not part of normal product authorization.

---

## ⚠️ Deliberate deviation from the repo's design authority

`docs/ui-design-system.md` specifies "stock-Android-inspired Material… **light theme first**" and
explicitly **forbids** "neon, glassmorphism, gamer styling". This design system was commissioned with a
different brief: **a Razer Synapse-style dark chrome with daidai (橙) orange accents, plus a light mode**.

What that changed, and what it did not:

- **Changed:** default theme is dark layered near-black; the accent is daidai orange `#EE7800` instead of
  the shipped indigo `#425B91`; an industrial grotesque (Archivo) carries titles, nav labels and micro-labels.
- **Kept exactly:** the spacing scale (4/8/12/16/24/32/48), radius scale (8/14/20/28/pill), type scale
  (12/14/16/22/32), hit-target minimums (42/48px), navigation structure, semantic-token approach,
  status-as-words rule, evidence-secondary hierarchy, and all product copy conventions.
- **Not adopted from Synapse:** no neon glow, no angular clipped panels, no animated backgrounds, no
  gradient text or gradient panels, no launcher-hub styling. The accent appears only on the primary
  action, the selected marker, the focus ring, eyebrows and the progress fill; every surface is flat
  colour with a hairline border. The intended read is a precision instrument, not a gaming hub.

The **light theme** (`data-theme="light"`) restores the shipped app's original paper palette, so a
consumer that must match the real product exactly can use it.

---

## Content fundamentals

Deslopper's copy is the product's main safety mechanism. It is careful, specific, and refuses to imply
work that did not happen.

**Voice.** First person singular for conclusions the app reached, second person for the user's own
decisions. The app says *"I found 2 settings that changed since an earlier inspection."* and
*"I have not inspected this PC yet."* — never "We" (there is no team behind the window) and never
"Your PC is optimised".

**Precision over reassurance.** Every uncertain result keeps its qualifier:
*"Unknown does not mean broken, and permission-limited does not mean absent."*
Six statuses are distinct and never merged: **Observed · Unknown · Permission limited · Externally
managed · Failed inspection · Not applicable**. There is no health score, percentage, grade, or
"issues found" count.

**Never claim completed work.** Previews say *"Execution is unavailable in this build"* and
*"Not yet applied"*. Success language is reserved for verified transactions:
*"applied and verified"*, *"restored to the exact prior DWORD"*. A rejected unchanged write is reported
as rejected, not as success.

**Explain the mechanism in empty states**, not the absence:
*"Drift appears only after comparable saved inspections. Failed and cancelled detector results are not
treated as proof of change."*

**Casing.** Sentence case everywhere in prose and buttons ("Run a fresh inspection", "Mark reviewed
locally", "Clear local history"). UPPERCASE only for eyebrows/kickers and micro-labels
("LOCAL WINDOWS CONFIGURATION", "OBSERVED STATE"). Windows nouns keep Microsoft's own casing
("Task View", "Widgets", "Clipchamp", "Phone Link"). Registry identities are verbatim and monospaced
(`SubscribedContent-338389Enabled`, `Microsoft.MicrosoftSolitaireCollection`, `DWORD 0`).

**Titles are declarative sentences with a period**: *"A clear view of what Windows is doing."*,
*"Understand Windows before deciding what you want."*, *"Observe first, explain clearly"*.

**No emoji, ever.** Three Unicode marks are permitted as UI furniture: `✓` (seal, principle list),
`●` (build-status dot), `→` (state change). Nothing else.

**Forbidden vocabulary:** debloat, optimise, clean, boost, repair, fix, malware/adware (for ordinary
Windows components), "one click to speed up", urgency, countdowns, and any dark pattern. Feature counts
are never inflated — the app states "six fixed current-user settings" rather than "full control".

**Buttons name the exact act and its scope:** *"Hide Widgets button"*, *"Remove for this account"*,
*"Undo last Task View change"*, *"Cancel safely"*, *"Retry safely"*.

---

## Visual foundations

**Colour.** One accent: daidai orange (`--daidai-500 #EE7800`) for the primary action, selected state,
eyebrows, focus ring, progress fill, and the state-change arrow. Neutrals follow Material's tonal
approach — derived from the accent hue at very low chroma, so the dark ramp is a warm near-neutral
(`#17161A` → `#F3EFF5`), never pure black. Light keeps the shipped paper palette. Status colours are green/amber/red
containers used **only alongside a word**. Depth comes from Material's **surface-container
hierarchy**, not from shadows: `--color-background` behind, `--color-surface-container-low` for cards,
`--color-surface-container` for nested blocks, `--color-surface-container-high` for the topmost tier.
Light mode drops the accent to `--daidai-600 #C25E00` for contrast on white.

**Type.** Archivo (display: page titles at 32–41px with `-0.035em` tracking, section titles at 22px,
nav labels, metric values, eyebrows at 12px `0.05em` uppercase) + IBM Plex Sans (body 16/1.55,
supporting 14, label 12) + IBM Plex Mono (registry values, package identities, DWORDs, transaction ids).
Only eyebrows are uppercase — micro-labels elsewhere stay sentence case so the UI reads as an
instrument rather than a launcher. Nothing
below 12px. Titles cap at 820px measure; body paragraphs at 760px.

**Spacing & layout.** 4/8/12/16/24/32/48 only. Fixed 244px sidebar (82px icon rail below 820px) with the
workspace scrolling independently; workspace padding `clamp(20px, 3vw, 42px)`. The catalogue is a
0.75fr/1.25fr split; overview is 1.35fr/0.65fr; metrics are five equal columns collapsing to three then
one. Nothing is centred in a hero-website sense — this is a desktop utility, dense at the top-left.

**Backgrounds.** Flat colour. No photography, no illustration, no pattern, no texture, no grain.
There are **no gradients at all** — the Overview hero is flat `--color-surface` with a hairline border.
No full-bleed imagery exists because the product ships none.

**Borders, cards, elevation.** Hairline 1px `--color-outline` first; shadow second. Cards are
`--radius-card` 20px, hairline border, `--shadow-card` (1px). No inner highlight, no glow.
Panels are 28px radius. The evidence `FactGrid` uses 1px gaps over an outline-coloured background so
cells separate without visible rules. Dialogs get `--shadow-floating` (18px/44px). Tonal separation is
always preferred to heavy borders — Material's tonal-elevation model, with shadow reserved for dialogs. The **only** accent-bordered surface is the Owner control block — the
one place a real change can be made.

**Transparency & blur.** Effectively none: `--color-scrim` behind dialogs and a `color-mix` translucent
seal. No backdrop blur, no glass. WebView2 rendering cost and Windows scaling reliability drive this.

**Corner radii.** 8 small (swatches, inline chips), 14 controls (buttons, inputs, glyph tiles, tonal
blocks), 20 cards, 28 panels/heroes/dialogs, pill for badges and filter chips. Never mix a new value.

**Hover / press / focus.** Interaction uses **Material state layers** rather than colour swaps: the base
surface is tinted by `--state-hover` (8%), `--state-pressed` (12%) or `--state-selected` (14% of the
accent). Filled primary buttons still move to `--daidai-400` on hover and `--daidai-600` on press. The
selected nav item and selected row carry a 14% accent tint plus an orange marker or border, never an
opaque orange fill; secondary hover also strengthens its border to `--color-outline-strong`. Press
is a 1px downward translate plus `--daidai-600`. Focus is a 3px `--color-focus-ring` box-shadow, always
visible, never removed. Disabled is `opacity: .55` with `cursor: not-allowed` — and the label still says
why (e.g. "Apply unavailable").

**Motion.** 140ms hover/press, 220ms section and dialog changes, 320ms progress fill. No entrance
animation, no parallax, no spinner theatre; progress is a real determinate bar with counts.
`prefers-reduced-motion` removes all of it. Motion never delays input.

**Selected state never relies on colour alone**: the active nav item carries a 4px orange marker plus
`aria-current="page"`, and rows carry `aria-selected`.

---

## Iconography

Deslopper ships **no icon set, no icon font, and no SVG sprite**. This is deliberate and documented:
production assets are local and must not require a CDN, and native semantics are preferred over custom
glyph widgets.

- **Letter glyphs are the iconography.** Navigation uses one-character tiles — `O` Overview, `C` Components,
  `P` Desired states, `D` Drift, `H` History, `S` Settings & About. Catalogue rows use the component
  name's first letter. `Glyph` is the component for this (`tonal` nav, `accent` list, `brand` app mark).
- **Three Unicode marks only:** `✓` (seal and principle lists), `●` (build-status dot), `→` (state change).
- **No emoji.** No Lucide/Heroicons/Fluent substitution has been introduced — adding an icon library
  would invent a vocabulary the product does not have. If a consumer genuinely needs icons, that is a new
  decision to make with the product owner, not something this system assumes.
- **The one real asset** is the app icon: `assets/logo.svg` (copied verbatim from `icons/app-icon.svg` —
  a rounded-square "D" in the original indigo `#425B91`) and `assets/icon.ico`. There is **no wordmark
  asset**; the in-app lockup is the orange `Glyph` tile plus "Deslopper" set in the display face.

---

## Components

Reusable primitives, grouped by concern. Each directory has a `@dsCard` HTML showing its states.

**`components/core/`** — `Button`, `Badge`, `Chip`, `Glyph`, `Eyebrow`, `CountChip`
**`components/forms/`** — `Field`, `Select`, `Checkbox`, `SearchInput`
**`components/layout/`** — `Surface`, `PageHeader`, `Hero`, `SectionHeading`, `MetricCard`, `FactGrid`, `EmptyState`
**`components/feedback/`** — `Callout`, `ProgressPanel`, `SafetyNote`, `StateChange`, `StatusText`, `Seal`
**`components/navigation/`** — `NavItem`, `ListRow`, `Disclosure`, `Dialog`

The inventory is derived from the shipped `+page.svelte` — every component corresponds to a real element
class in that file (`.ds-btn` ← `button.primary`, `Surface` ← `.surface`, `FactGrid` ← `dl.facts`,
`StateChange` ← `.state-change`, `Seal` ← `.read-only-seal`, and so on).

**Intentional additions** (no 1:1 source component, added for reuse):
- `Glyph` — wraps the repeated letter-tile markup (`.logo`, `.nav-glyph`, `.component-icon`).
- `Callout` — unifies four near-identical source blocks (`.boundary`, `.validation`, `.notice`,
  `.error-banner`, `.owner-result`) behind one `tone` prop.
- `StatusText` — the source renders `observationStatus()` inline; this makes the six statuses a typed API.

No Toast, Avatar, Tabs, Tooltip, Accordion, or Table exists — the product has none.

## UI kit

`ui_kits/deslopper-app/` — interactive recreation of the Owner Mode desktop app:
onboarding dialog → running inspection with live progress → Overview → component catalogue with detail,
Owner control (apply/undo) and package removal → desired states → drift → history → settings &
diagnostics review. Includes a dark/light toggle. All data is synthetic and shaped like the Rust model;
no real machine data is present.

## Index

- `styles.css` — the only file consumers link; `@import` list only.
- `tokens/` — `fonts.css`, `colors.css` (dark default + `[data-theme="light"]`), `typography.css`,
  `spacing.css`, `elevation.css`, `base.css`.
- `components/ds-components.css` — component class implementations (values lifted from the shipped app).
- `components/<group>/` — 25 components: `.jsx`, `.d.ts`, `.prompt.md`, plus one card HTML per group.
- `guidelines/` — 17 foundation specimen cards (Colors, Type, Spacing, Brand).
- `ui_kits/deslopper-app/` — the desktop app kit (`index.html`, `Shell.jsx`, `ComponentsScreen.jsx`,
  `Screens.jsx`, `data.js`, `README.md`).
- `assets/` — `logo.svg`, `icon.ico`.
- `thumbnail.html` — homepage tile.
- `github.md` — source-repo association and sync record.
- `SKILL.md` — Agent Skills entry point.

**Note on this copy of the system:** this skill directory currently holds the top-level contract
(`SKILL.md`, `readme.md`, `github.md`, `styles.css`, `_ds_manifest.json`, `_ds_bundle.js`,
`_adherence.oxlintrc.json`, `thumbnail.html`). The `tokens/`, `components/`, `guidelines/`,
`ui_kits/`, and `assets/` directories referenced above live in the source Claude Design project
(<https://claude.ai/design/p/c828b348-1fcc-4b56-866e-94b8763c0f56>) and have not been synced into the
repository yet — re-run the import to pull them in when they're needed locally.

## Font substitution — needs your input

The repository ships **no font binaries**. Its stack is `Inter, 'Segoe UI Variable Text', 'Segoe UI',
system-ui` — Segoe resolves from Windows at runtime. For this system:

- **IBM Plex Sans** replaces the source's Inter for UI text — engineered, plain, and closer to a
  desktop instrument than a product-marketing grotesque.
- **Archivo** is the display face for titles, nav labels and micro-labels. **This is an invention, not a
  brand asset.**
- **IBM Plex Mono** substitutes for Cascadia/Consolas for registry evidence.

If Deslopper has, or wants, a real display face, send the files and this system will be re-cut around it.
For production the app must bundle any webfont locally — remote CDN loading is forbidden by
`docs/ui-design-system.md`.
