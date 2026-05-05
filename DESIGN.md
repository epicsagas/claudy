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
- **Display/Labels:** Bricolage Grotesque — slightly irregular letterforms that feel hand-considered without being quirky. Weight range 200-800 for aggressive contrast between labels and headings.
- **Data/Metrics/Tables:** JetBrains Mono — tabular-nums for aligned figures, slashed zero, distinguishable 1Il. Audit-grade numbers.
- **UI/Controls:** Bricolage Grotesque (same as display)
- **Fallback chain:** `Bricolage Grotesque, JetBrains Mono, ui-monospace, monospace`
- **Loading:** Google Fonts CDN (`https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,200..800&family=JetBrains+Mono:ital,wght@0,100..800;1,100..800&display=swap`)
- **Scale:**
  - 48px: Hero metrics (font-weight 700-800)
  - 32px: KPI numerals (font-weight 700, tabular-nums)
  - 24px: Large values
  - 15px: Section labels (uppercase, letter-spacing 0.06em)
  - 13px: Body text
  - 11px: Micro labels, axis labels, timestamps (uppercase, tracked)
  - 10px: Filter labels (uppercase, letter-spacing 0.08em)

## Color
- **Approach:** Restrained — one accent, meaningful neutrals
- **Background:** `#0a0a0b` — near-black, warm, no blue tint
- **Surface:** `#141416` — raised panels
- **Surface elevated:** `#1c1c1f` — hover states, active rows
- **Border subtle:** `#2a2a2e`
- **Border hover:** `#3a3a3e`
- **Primary text:** `#e8e6e3` — warm white, like paper
- **Muted text:** `#7a7872`
- **Dim text:** `#4a4943` — disabled, placeholders
- **Accent (primary):** `#e85d04` — burnt orange. Reads as heat, fuel gauge, energy. Thermodynamically appropriate for tracking computational resources.
- **Accent hover:** `#f48c06`
- **Accent glow:** `rgba(232, 93, 4, 0.12)`
- **Accent dim:** `rgba(232, 93, 4, 0.06)`
- **Critical:** `#dc2f02` — cost overruns, alerts
- **Positive:** `#60a840` — savings, muted green
- **Warning:** `#f2b94b`
- **Chart palette:** `#e85d04` (primary), `#6b6459` (secondary), `#3d3a35` (tertiary), `#8a8477` (quaternary)

## Spacing
- **Base unit:** 8px
- **Density:** Compact — data-dense, not spacious
- **Scale:** xs(4px) sm(8px) md(16px) lg(24px) xl(32px)

## Layout
- **Approach:** Hybrid — sparkline ribbon + asymmetric editorial paneling
- **First viewport:** Full-width data ribbon (sparkline timeline with current KPIs overlaid), then asymmetric 60/40 split below
- **Left 60%:** Dominant chart area
- **Right 40%:** Compressed intelligence rail (metrics, model distribution, top tools)
- **Below fold:** Session history as a dense ledger, not a card gallery
- **Title block:** Top-left, not centered
- **Grid:** 8px grid with thin structural borders
- **Max content width:** 1280px
- **Border radius:** Sharp — max 4px on interactive elements, 2px on panels, 0px on data containers. Rounded corners signal "friendly consumer app." Sharp corners signal "instrument."

## Motion
- **Approach:** Minimal-functional — only animations that aid comprehension
- **Easing:** ease-out for enter, ease-in for exit
- **Duration:** Chart trace reveal on load (400ms), subtle count-up for spend numbers, row highlight sweep on hover (150ms)
- **Loading state:** Single horizontal scan line moving left-to-right (CRT feel), 1.5s cycle

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

## Decisions Log
| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-05-05 | Initial design system created | Created by /design-consultation. Industrial/Utilitarian aesthetic, burnt orange accent, Bricolage Grotesque + JetBrains Mono. Synthesized from competitive research (Vercel, Linear, Langfuse, Mintlify), Codex design voice, and independent Claude subagent. |
| 2026-05-05 | Burnt orange over acid-lime or blue/purple | Maps to resource tracking (heat, fuel gauge). Distinct from category norm of purple/blue AI dashboards. Codex recommended lime, subagent recommended orange. Orange chosen for "instrument" reading over "developer tool" reading. |
| 2026-05-05 | No stat cards, sparkline ribbon instead | Every AI dashboard shows 4 stat cards in a grid. Replacing with a data ribbon makes the first viewport a landscape, not a widget gallery. Risk accepted for visual distinction. |
| 2026-05-05 | Monospace-first typography | No sans-serif body font. UI is either labels (Bricolage Grotesque) or data (JetBrains Mono). Immediately reads as precision instrument. |
