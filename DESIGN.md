# Design System — Claudy Analytics Dashboard

## Product Context
- **What this is:** A desktop analytics dashboard (Tauri + Svelte 5) for tracking Claude CLI usage: token counts, costs, tool calls, session history, and cost-saving recommendations.
- **Who it's for:** Developers who use Claude CLI regularly and want to understand their usage patterns and spending.
- **Space/industry:** Developer tools, AI observability, CLI analytics.
- **Project type:** Desktop application (Tauri) with web frontend (Svelte 5 + Chart.js)

## Themes
- **Dark theme:** Financial-platform dark design system — `DESIGN-dark.md`
- **Light theme:** Consumer marketplace light design system — `DESIGN-light.md`

---

## Dark Theme

### Colors
- `primary` / accent: `#fcd535` — Yellow. Every primary CTA, active nav, brand moment.
- `primary-active`: `#f0b90b`
- `primary-disabled`: `#3a3a1f`
- `on-primary`: `#181a20` — black text on yellow
- `canvas-dark` / bg: `#0b0e11`
- `surface-card-dark`: `#1e2329`
- `surface-elevated-dark`: `#2b3139`
- `on-surface` / body: `#eaecef`
- `on-dark`: `#ffffff`
- `muted`: `#707a8a`
- `muted-strong`: `#929aa5`
- `hairline-on-dark`: `#2b3139`
- `trading-up` / positive: `#0ecb81`
- `trading-down` / critical: `#f6465d`
- `warning`: `#f2b94b`
- `border`: `#2b3139`
- `border-strong`: `#363d47`

### Typography
- Display: **Bricolage Grotesque**
- Numbers/data: **JetBrains Mono**
- `hero-display`: 64px / 700 / line-height 1.1 / letter-spacing -1px
- `display-lg`: 48px / 700 / 1.1 / -0.5px
- `display-md`: 40px / 600 / 1.15 / -0.3px
- `display-sm`: 32px / 600 / 1.2
- `title-lg`: 24px / 600 / 1.3
- `title-md`: 20px / 600 / 1.35
- `title-sm`: 16px / 600 / 1.4
- `number-display`: 40px / 700 / 1.1 / -0.3px — JetBrains Mono
- `number-md`: 16px / 500 / 1.4 — JetBrains Mono
- `number-sm`: 14px / 500 / 1.4 — JetBrains Mono
- `body-md`: 14px / 400 / 1.5
- `body-sm`: 13px / 400 / 1.5
- `caption`: 12px / 500 / 1.4
- `button`: 14px / 600 / 1
- `nav-link`: 14px / 500 / 1.4

### Border Radius
- `xs`: 2px
- `sm`: 4px
- `md`: 6px — standard CTA buttons, inputs
- `lg`: 8px — content cards, trust badges
- `xl`: 12px — elevated card containers
- `pill`: 9999px — prominent feature CTAs only

### Spacing
- section: 80px
- xxl: 48px / xl: 32px / lg: 24px / md: 16px / sm: 12px / xs: 8px / xxs: 4px

### Components
- `button-primary`: bg `#fcd535`, text `#181a20`, radius `md` (6px), padding 12px 24px, height 40px, font `button` 14px/600
- `button-primary-active`: bg `#f0b90b`
- `button-primary-disabled`: bg `#3a3a1f`, text `#707a8a`
- `button-secondary-on-dark`: bg `#1e2329`, text `#ffffff`, radius `md`
- `button-tertiary-text`: transparent, text `#eaecef`
- `top-nav-dark`: bg `#0b0e11`, text `#ffffff`, height 64px
- `markets-table-card`: bg `#1e2329`, radius `xl` (12px), padding 24px
- `markets-row`: transparent bg, padding 12px 0, hairline divider
- `price-up-cell`: text `#0ecb81`
- `price-down-cell`: text `#f6465d`
- `stat-callout-card`: transparent bg, text `#fcd535`, font `number-display`
- `trust-badge`: bg `#1e2329`, radius `lg` (8px), padding 16px 20px
- `faq-row`: transparent, padding 20px 0, font `title-sm`
- `cta-band-dark`: bg `#1e2329`, radius `xl`, padding 48px
- `text-link`: text `#fcd535`

---

## Light Theme

### Colors
- `primary` / accent: `#ff385c` — Red. Every primary CTA, search orb, active state.
- `primary-active`: `#e00b41`
- `primary-disabled`: `#ffd1da`
- `on-primary`: `#ffffff`
- `canvas` / bg: `#ffffff`
- `surface-soft`: `#f7f7f7`
- `surface-strong`: `#f2f2f2`
- `ink` / on-surface: `#222222`
- `body` / on-surface-variant: `#3f3f3f`
- `muted` / outline: `#6a6a6a`
- `muted-soft` / outline-variant: `#929292`
- `hairline` / border: `#dddddd`
- `hairline-soft`: `#ebebeb`
- `border-strong`: `#c1c1c1`
- `positive`: `#008a05`
- `critical` / error: `#c13515`

### Typography
- Display/UI: **Space Grotesk**
- Data/Numbers: **JetBrains Mono** (shared)
- `display-xl`: 28px / 700 / 1.43
- `display-lg`: 22px / 500 / 1.18 / -0.44px
- `display-md`: 21px / 700 / 1.43
- `display-sm`: 20px / 600 / 1.20 / -0.18px
- `title-md`: 16px / 600 / 1.25
- `title-sm`: 16px / 500 / 1.25
- `body-md`: 16px / 400 / 1.5
- `body-sm`: 14px / 400 / 1.43
- `caption`: 14px / 500 / 1.29
- `caption-sm`: 13px / 400 / 1.23
- `badge`: 11px / 600 / 1.18
- `button-md`: 16px / 500 / 1.25
- `button-sm`: 14px / 500 / 1.29
- `nav-link`: 16px / 600 / 1.25

### Border Radius
- `none`: 0px
- `xs`: 4px
- `sm`: 8px — buttons
- `md`: 14px — property cards
- `lg`: 20px
- `xl`: 32px — category strip
- `full` / `pill`: 9999px — search bar, pill buttons, tags

### Spacing
- section: 64px
- xxl: 48px / xl: 32px / lg: 24px / base: 16px / md: 12px / sm: 8px / xs: 4px / xxs: 2px

### Components
- `button-primary`: bg `#ff385c`, text `#ffffff`, radius `sm` (8px), padding 14px 24px, height 48px
- `button-primary-active`: bg `#e00b41`
- `button-primary-disabled`: bg `#ffd1da`
- `button-secondary`: bg `#ffffff`, text `#222222`, radius `sm`, border 1px `#222222`
- `button-pill`: bg `#ff385c`, text `#ffffff`, radius `full`, padding 10px 20px
- `search-bar-pill`: bg `#ffffff`, radius `full`, height 64px, border 1px `#dddddd`
- `search-orb`: bg `#ff385c`, text `#ffffff`, radius `full`, height 48px
- `top-nav`: bg `#ffffff`, text `#222222`, height 80px, border-bottom 1px `#dddddd`
- `property-card`: bg `#ffffff`, text `#222222`, radius `md` (14px)
- `reservation-card`: bg `#ffffff`, radius `md`, border 1px `#dddddd`, padding 24px
- `text-input`: bg `#ffffff`, border 1px `#dddddd`, radius `sm` (8px), height 56px
- `footer-light`: bg `#ffffff`, padding 48px 80px

---

## Elevation
- **Dark:** No shadows. Depth from `#0b0e11` → `#1e2329` → `#2b3139` surface steps.
- **Light:** One shadow tier only — `box-shadow: rgba(0,0,0,0.02) 0 0 0 1px, rgba(0,0,0,0.04) 0 2px 6px, rgba(0,0,0,0.1) 0 4px 8px` — used on hover cards and dropdowns only.

## Anti-Patterns
- Dark: No yellow body text or large fills — yellow is focal-point CTAs only
- Dark: No atmospheric gradients — flat color-block contrast only
- Dark: Never white text on yellow (always `#181a20` on `#fcd535`)
- Light: No hard corners — every interactive element is rounded
- Light: No dark mode on public/light surfaces
- Both: No glassmorphism, no backdrop blur, no decorative blobs
- Both: No smooth Bezier curves on charts — `tension: 0` always

## Decisions Log
| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-06 | Dark = financial-platform dark tokens, Light = consumer marketplace light tokens | User directive: use DESIGN-dark.md and DESIGN-light.md as-is |
| 2026-05-06 | Dark accent `#fcd535` Yellow, Light accent `#ff385c` Red | Exact brand colors from each reference |
| 2026-05-06 | `on-primary` dark = `#181a20` (black on yellow) | Dark theme signature — white on yellow loses contrast |
