# Design System — Claudy Analytics Dashboard

## Product Context
- **What this is:** A desktop analytics dashboard (Tauri + Svelte 5) for tracking Claude CLI usage: token counts, costs, tool calls, session history, and cost-saving recommendations.
- **Who it's for:** Developers who use Claude CLI regularly and want to understand their usage patterns and spending.
- **Space/industry:** Developer tools, AI observability, CLI analytics.
- **Project type:** Desktop application (Tauri) with web frontend (Svelte 5 + Chart.js)
- **Memorable thing:** "Serious tool for serious work"

## Aesthetic Direction
- **Direction:** Industrial/Utilitarian
- **Decoration level:** Minimal — typography does all the work. No gradients, no glassmorphism, no decorative blobs. Structure comes from tone blocks, thin 1px borders, and spacing.
- **Mood:** The visual language of instruments, where decoration is a distraction from signal. Bloomberg Terminal meets Linear. Data-dense, monospace accents, function-first.
- **Reference points:** Bloomberg Terminal, oscilloscope screens, Leica technical manuals, Unix tool seriousness.

## Typography

### Dark Theme
- **Display/Labels:** Bricolage Grotesque — slightly irregular letterforms that feel hand-considered without being quirky. Weight range 200-800 for aggressive contrast between labels and headings.
- **Data/Metrics/Tables:** JetBrains Mono — tabular-nums for aligned figures, slashed zero, distinguishable 1Il. Audit-grade numbers. All telemetry, logs, metrics, and code snippets must use `font-variant-numeric: tabular-nums`.
- **UI/Controls:** Bricolage Grotesque (same as display)
- **Loading:** Google Fonts CDN (`https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,200..800&family=JetBrains+Mono:ital,wght@0,100..800;1,100..800&display=swap`)

### Light Theme
- **Display/Labels:** Space Grotesk — geometric, technical feel that pairs with the warmer light palette. Weight range 400-700.
- **Data/Metrics/Tables:** JetBrains Mono — same as dark theme.
- **UI/Controls:** Space Grotesk (same as display)
- **Loading:** Add `&family=Space+Grotesk:wght@400;500;600;700` to the Google Fonts URL.

### Shared Scale
- **Fallback chain:** `[Bricolage Grotesque | Space Grotesk], JetBrains Mono, ui-monospace, monospace`
- **Scale:**
  - 48px `display-xl`: Hero metrics (wght 800, line-height 1.1, letter-spacing -0.02em)
  - 32px `display-lg`: KPI numerals (wght 700, line-height 1.2, letter-spacing -0.01em)
  - 24px `data-lg`: Large values (wght 700, line-height 1, letter-spacing -0.02em, JetBrains Mono)
  - 20px `heading-md`: Section headings (wght 600, line-height 1.4)
  - 15px: Section labels (uppercase, letter-spacing 0.06em)
  - 14px `ui-medium`: Body text (wght 500, line-height 1.2)
  - 13px `data-mono`: Table body, code (wght 400, line-height 1.5, JetBrains Mono)
  - 12px `ui-small`: Navigation labels (wght 500, line-height 1)
  - 11px `data-label`: Micro labels, axis labels, timestamps (wght 500, letter-spacing 0.05em, JetBrains Mono, uppercase)
  - 10px: Filter labels (uppercase, letter-spacing 0.08em)
- **Rule:** Data labels must use uppercase with increased tracking to mimic technical blueprints. All numeric columns must use JetBrains Mono with tabular-nums.

## Color — Dark Theme
- **Approach:** Restrained — one accent, meaningful neutrals. Material You Fidelity variant with custom overrides (primary `#e85d04`, neutral `#7a7872`).
- **System tokens (from Stitch):**
  - `background` / `surface` / `surface-dim`: `#14130f`
  - `surface-container-lowest`: `#0f0e0a`
  - `surface-container-low`: `#1c1c17`
  - `surface-container`: `#20201b`
  - `surface-container-high`: `#2b2a25`
  - `surface-container-highest`: `#363530`
  - `surface-bright`: `#3a3934`
  - `surface-variant`: `#363530`
- **Text:**
  - `on-surface` (primary text): `#e6e2db` — warm white
  - `on-surface-variant`: `#e1bfb2` — warm muted
  - `outline`: `#a98a7e` — muted text, secondary info
  - `outline-variant`: `#594137` — dim text, placeholders
- **Accent (primary):**
  - `primary`: `#ffb596` — Material You primary tone
  - `primary-container`: `#f26411` — Accent for interactive elements, buttons
  - Override accent: `#e85d04` — burnt orange. Reads as heat, fuel gauge, energy.
  - Accent hover: `#f48c06`
  - Accent glow: `rgba(232, 93, 4, 0.12)`
  - Accent dim: `rgba(232, 93, 4, 0.06)`
- **Secondary/tertiary:**
  - `secondary`: `#c8c6c3`
  - `secondary-container`: `#494947`
  - `tertiary`: `#c8c6c9`
- **Semantic:**
  - Critical: `#dc2f02` — cost overruns, alerts, errors
  - Positive: `#60a840` — savings, success states (slightly desaturated)
  - Warning: `#f2b94b`
  - Error: `#ffb4ab` on `#93000a` container
- **Borders:**
  - Structural: `#2a2a2e` — grid-line borders, 1px solid
  - Hover: `#3a3a3e`
- **Chart palette:** `#e85d04` (primary), `#6b6459` (secondary), `#3d3a35` (tertiary), `#8a8477` (quaternary)

## Color — Light Theme
- **Approach:** Warm neutral base with burnt orange accent. Inverts the dark theme's tonal layering — surfaces move from lightest background to slightly darker as they gain elevation.
- **System tokens (from Stitch Token & Cost Analysis v2):**
  - `background`: `#fbfaf8`
  - `surface`: `#fff8f2`
  - `surface-container-lowest`: `#ffffff`
  - `surface-container-low`: `#fdf2e4`
  - `surface-container`: `#f7ecde`
  - `surface-container-high`: `#f1e7d9`
  - `surface-container-highest`: `#ebe1d3`
  - `surface-dim`: `#e3d9cb`
  - `surface-bright`: `#fff8f2`
  - `surface-variant`: `#ebe1d3`
- **Text:**
  - `on-surface` (primary text): `#14130f`
  - `on-surface-variant`: `#6b6459`
  - `outline`: `#7a776f` — muted text
  - `outline-variant`: `#cbc6bd` — borders, dividers
- **Accent:**
  - Primary accent: `#e85d04` — same burnt orange as dark theme
  - `secondary`: `#e85d04` (doubles as accent in light theme)
  - `on-secondary`: `#ffffff`
- **Semantic:**
  - Error: `#ba1a1a`
  - Positive: `#2e7d32` / `bg-green-50` with `border-green-200`
- **Borders:**
  - Structural: `#cbc6bd`
  - Grid lines: `#ebe1d3` (1px CSS background-image pattern at 20px intervals)

## Spacing
- **Base unit:** 4px
- **Density:** Compact — data-dense, not spacious
- **Scale:** xs(4px) sm(8px) md(16px) lg(24px) xl(40px) gutter(1px)
- **Grid approach:** 1px structural borders (`gutter: 1px`). Panels share borders rather than having independent margins. Use CSS grid with `gap: 1px; background-color: <border-color>` for the grid-line technique.

## Layout

### Shell Structure (Fixed)
All screens share a common shell:
- **Side navigation rail:** 64px wide, fixed left, full viewport height. Contains: logo, icon-only nav buttons (Dashboard, Tokens, Tools, Logs), settings at bottom. Active state: orange border-left 2px + orange text + orange/5% background.
- **Top app bar:** 48px tall, fixed top (offset `left: 64px`). Contains: app title (`CLAUDY_ANALYTICS`, JetBrains Mono 11px uppercase), tab navigation (LIVE_FEED / METRICS / ALERTS), search input, utility icons. Active tab: orange underline + orange text.
- **Status bar:** 24px tall, fixed bottom (offset `left: 64px`). Contains: system status (API: NOMINAL), throughput, latency, region (left), live sync indicator (right).
- **Main content:** `margin-left: 64px; margin-top: 48px; height: calc(100vh - 48px - 24px)`.

### Content Layout
- **Approach:** Hybrid — sparkline ribbon + asymmetric editorial paneling
- **First viewport:** Full-width data ribbon (4 sparkline tiles with KPIs), then asymmetric 60/40 split below
- **Left 60%:** Dominant chart area (usage line chart, stacked bar chart, etc.)
- **Right 40%:** Compressed intelligence rail (model distribution, top tool metrics, optimization recommendations)
- **Below fold:** Session history as a dense ledger, not a card gallery
- **Title block:** Top-left, not centered
- **Grid:** 4px modular grid with thin structural borders (grid-line technique)
- **Max content width:** 1280px
- **Border radius:** Sharp — max 4px on interactive elements, 2px on dashboard tiles, 0px on data containers/panels. Rounded corners signal "friendly consumer app." Sharp corners signal "instrument."

## Components

### Buttons
- **Primary:** Accent background (`#e85d04`), white text. No rounded corners (0-4px max). On hover: slightly lighter tint. No glow, no shadow.
- **Secondary:** Transparent background, 1px border (`border-technical`). On hover: fill with slightly lighter tint.
- **Active state:** Scale 95% on press (`active:scale-95`).
- **Font:** `data-label` (JetBrains Mono 11px, uppercase, tracked).

### Data Grids / Tables
- **Row height:** 32px (dense) or 40px (default).
- **Dividers:** 1px horizontal dividers only (`border-technical`).
- **Column headers:** `data-label` typography, uppercase, pinned sticky top during scroll.
- **Zebra striping:** Subtle — alternate rows use `surface-container-lowest` at 30% opacity.
- **Hover:** `bg-orange-500/5` with `cursor: crosshair`.
- **Cell padding:** `px-4 py-2` (dense), `p-3` (default).

### Metrics / KPIs
- **Large numbers:** `data-lg` (JetBrains Mono 24px, wght 700).
- **Trend indicators:** Arrows using Positive/Critical colors. Inline next to value in smaller `data-mono` font.
- **Charts:** No smooth Bezier curves — stepped lines or sharp angles only. This emphasizes raw data points over interpolated trends.
- **Sparklines:** Minimal bars in a horizontal strip, using accent color at varying opacities.

### Status Tags
- **Shape:** Small rectangular chips, 0px border-radius.
- **Style:** Subtle background (10% opacity of status color) + solid 1px border of same color.
- **Text:** `data-label` typography, always uppercase.
- **Variants:**
  - Active: `border-orange-500/50 bg-orange-500/10 text-orange-500`
  - Closed: `border-zinc-700 bg-zinc-800 text-zinc-400`
  - Success: `bg-emerald-500` LED indicator + `text-emerald-500` label
  - Fail: `bg-rose-600` LED indicator + `text-rose-600` label
  - Critical: `bg-rose-500/10 text-rose-500 border-rose-500/20`
  - Optimized: `text-green-500 bg-green-500/10 border-green-500/20`

### Inputs
- **Search/command bars:** 1px border, no outer glow. Focus state: 1px accent border.
- **Font:** Monospace (`data-mono`) for all text input to maintain character alignment.
- **Select dropdowns:** `surface-container-lowest` background, `border-technical`, no custom styling beyond native.

### Filter Bars
- **Layout:** Horizontal bar between top app bar and content area, `h-14`, `surface-container-low` background.
- **Contents:** Tool filter (select), status toggle (segmented buttons: ALL/SUCCESS/FAIL), date range picker, action button (Export).
- **Segmented buttons:** Container with `border-technical`, inner buttons with no border. Active: accent background + white text.

### FAB (Floating Action Button)
- **Only on:** Tool Inspector screen (for adding new tool configurations).
- **Style:** Square (0px radius), accent background, white icon, 1px border.
- **Position:** Fixed bottom-right, above status bar.

## Screen Specifications

### Screen 1: Claudy Dashboard
- **Layout:** Data ribbon → 60/40 split → session ledger → footer
- **Data ribbon:** 4-column grid (`grid-cols-4`), each tile shows: label (`data-label`), value (`data-lg`), trend indicator, sparkline bars.
  - Tiles: TOTAL_TOKENS, EST_COST, TOOL_CALLS, AVG_LATENCY
- **60/40 split:**
  - Left (60%, `col-span-6`): SVG line chart (sharp angles, no Bezier). Time range toggle (1H/24H/7D). Tooltip with dashed vertical line.
  - Right (40%, `col-span-4`): MODEL_DISTRIBUTION (progress bars) + TOP_TOOL_METRICS (list items with icons).
- **Session ledger table:** Columns: SESSION_ID, START_TIME, TOKENS, COST, TOOLS, STATUS. Status tags: Active (orange) / Closed (zinc).
- **Footer:** System status line with live sync pulse indicator.

### Screen 2: Token & Cost Analysis (Dark)
- **Layout:** KPI header → stacked bar chart → model cost table + optimization panel
- **KPI header:** 2-column grid. Left: Current Month Spend + progress bar. Right: Projected Spend + segmented progress.
- **Stacked bar chart:** Daily token cost (Input + Output). CSS-based bars with hover tooltips. X-axis: dates. Legend: OUTPUT (accent) / INPUT (muted).
- **Model cost table:** Columns: Model Identifier (with color dot), Tokens (In), Tokens (Out), Total Cost, Efficiency (tag). Sortable.
- **Optimization panel (right, `col-span-3`):** Recommendation cards with priority tag (HIGH SAVINGS / EFFICIENCY / CONTEXT), estimated savings, description, action button.
- **Bottom visual:** Server rack image with gradient overlay + system pulse label.

### Screen 3: Token & Cost Analysis (Light)
- **Same layout as Screen 2** but with light theme tokens and Space Grotesk typography.
- **Grid lines:** CSS background-image pattern at 20px intervals using `#ebe1d3`.
- **Background:** `#fbfaf8`, panels use `#ffffff` with `border-outline-variant`.
- **Recommendation cards:** White background with subtle shadow (`shadow-sm`).

### Screen 4: Session History
- **Layout:** Filter header → ledger table → pagination bar
- **Filter header:** Search input (full-width with icon), filter/calendar buttons, Export CSV button.
- **Ledger table:** Columns: checkbox, SESSION_ID, START_TIME, DURATION, TOTAL_TOKENS, TOTAL_COST, PRIMARY_MODEL (tag), TOOL_COUNT. Font: `data-mono`. Rows alternate background.
- **Pagination bar:** Entry count, page size selector (25/50/100), page navigation (first/prev/1/2/3/4/next/last). Active page: orange highlight.
- **Floating widget:** Session Insights panel (bottom-right) showing AVG_LATENCY, TOTAL_BURN, PEAK_CONCURRENCY.

### Screen 5: Tool Inspector
- **Layout:** Split view — tool call ledger (left, `flex-1`) + detail pane (right, `w-[450px]`)
- **Tool call ledger:** Columns: TIMESTAMP_UTC, TOOL_ID, ARGUMENTS_SNAPSHOT, STATUS (LED + label), LAT_MS, TOKENS. Active row highlighted with `bg-orange-500/10`.
- **Detail pane (Execution Inspector):**
  - Header: title + close button
  - Metadata grid (2-col): Execution_Time, Total_Latency
  - Input Arguments: code block (`bg-zinc-950`, monospace, syntax-highlighted JSON)
  - Stack Trace / Error Log: code block with error in rose, trace in dim text. Critical badge.
  - Visual context: Server image with gradient overlay + Node_Cluster_ID label
  - Footer actions: Retry_Call, Copy_JSON (full-width buttons)

## Motion
- **Approach:** Minimal-functional — only animations that aid comprehension
- **Easing:** ease-out for enter, ease-in for exit
- **Duration:** Chart trace reveal on load (400ms), subtle count-up for spend numbers, row highlight sweep on hover (150ms)
- **Loading state:** Single horizontal scan line moving left-to-right (CRT feel), 1.5s cycle
- **Transition:** All transitions use `transition-colors duration-75` for snappy, industrial feel. No slow fades.

## Anti-Patterns (never do these)
- No purple/violet gradients as default accent
- No 3-column feature grid with icons in colored circles
- No centered everything with uniform spacing
- No uniform bubbly border-radius on all elements
- No gradient buttons as primary CTA
- No glassmorphism or frosted glass effects
- No system-ui / -apple-system as primary display or body font
- No floating stat cards with rounded corners and shadows
- No decorative background shapes or blobs
- No smooth Bezier curves on data charts — stepped or sharp angles only
- No backdrop blurs or ambient shadows — flat, physical, mechanically assembled

## Decisions Log
| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-05 | Initial design system created | Created by /design-consultation. Industrial/Utilitarian aesthetic, burnt orange accent, Bricolage Grotesque + JetBrains Mono. Synthesized from competitive research (Vercel, Linear, Langfuse, Mintlify), Codex design voice, and independent Claude subagent. |
| 2026-05-05 | Burnt orange over acid-lime or blue/purple | Maps to resource tracking (heat, fuel gauge). Distinct from category norm of purple/blue AI dashboards. Codex recommended lime, subagent recommended orange. Orange chosen for "instrument" reading over "developer tool" reading. |
| 2026-05-05 | No stat cards, sparkline ribbon instead | Every AI dashboard shows 4 stat cards in a grid. Replacing with a data ribbon makes the first viewport a landscape, not a widget gallery. Risk accepted for visual distinction. |
| 2026-05-05 | Monospace-first typography | No sans-serif body font. UI is either labels (Bricolage Grotesque) or data (JetBrains Mono). Immediately reads as precision instrument. |
| 2026-05-05 | Stitch screen import — background #14130f | Stitch screens use warmer near-black (#14130f) over original DESIGN.md (#0a0a0b). Adopted for Material You Fidelity variant consistency across all tokens. |
| 2026-05-05 | Dual-theme support (dark + light) | Light theme added based on Token & Cost Analysis v2 Stitch screen. Space Grotesk chosen as light theme display font for its geometric, technical character. Dark theme retains Bricolage Grotesque. |
| 2026-05-05 | Material You surface tier system | 5-tier surface layering (lowest → low → → high → highest) replaces the original 3-tier system. Provides finer tonal control for elevation without shadows. |
| 2026-05-05 | Grid-line technique for borders | CSS grid with `gap: 1px; background-color: <border>` creates shared structural borders without double-border artifacts. Panels use `background-color: <surface>` to fill cells. |
