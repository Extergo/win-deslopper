# UI design system

This is the authority for Deslopper's visual language: stock-Android-inspired Material adapted to a native Windows desktop, without copying Google branding or assets. The light theme is first. Layouts are spacious, calm, rounded, high-contrast, and responsive.

## Tokens

All reusable UI tokens live in `ui/theme/material-theme.slint`.

- Spacing scale: 4, 8, 12, 16, 24, 32, and 48 px.
- Radius scale: 8 px small, 14 px controls, 20 px cards, 28 px prominent surfaces/pills.
- Typography: 12 px label, 14 px supporting, 16 px body/control, 22 px section title, 32 px page title.
- Semantic colours: background, surface, elevated surface, primary, primary container, text, secondary text, outline, success, warning, and disabled. Components use semantic tokens rather than arbitrary colours.
- Interactive targets are at least 40 px high, preferably 48 px for primary controls.

Navigation has a persistent text label, a non-colour selected marker, hover/pressed feedback, and visible keyboard focus. Cards preserve readable hierarchy and use tonal separation instead of heavy borders. Planned and status states include words, not colour alone. Empty states use simple native shapes and helpful copy. Motion is short, restrained, and never delays input.

Forbidden: neon, glassmorphism, gamer styling, random gradients, inconsistent radii, arbitrary colours, dense enterprise tables as the default, legacy control-panel imitation, unapproved mixing of Fluent/macOS/Material/gamer styles, scattered hardcoded tokens, decorative blocking animation, and success states for work that did not occur.

