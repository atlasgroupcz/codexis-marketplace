---
uuid: 6c7ba16b-f527-4dd3-b71d-0d627eae4198
name: pptx
icon: icon.svg
description: >-
  Use any time a .pptx is involved — as input, output, or both. Covers building slide decks and
  presentations, reading or extracting content from an existing deck (even when the content is
  destined for something else), editing a deck, and working with templates or speaker notes.
  Triggers on "prezentace", "slidy", "deck", "pptx", "PowerPoint", "připrav prezentaci",
  "shrň do slidů", "školení", "pitch".
version: 1.0.0
allowed-tools: shell
i18n:
  cs:
    displayName: "PowerPoint prezentace"
    summary: "Tvorba a úprava prezentací (.pptx) s vizuální kontrolou každého slidu."
  en:
    displayName: "PowerPoint Presentations"
    summary: "Build and edit .pptx decks with visual inspection of every slide."
  sk:
    displayName: "PowerPoint prezentácie"
    summary: "Tvorba a úprava prezentácií (.pptx) s vizuálnou kontrolou každého slidu."
---

# PPTX — building decks that survive being looked at

Slides are the one format where the render *is* the product. Text extraction tells you nothing
about whether a deck works.

**IMPORTANT:** Stop on a genuine `Error:` line from `cdx-render` — report it and don't blindly
retry. A missing `soffice` or `pdftoppm` means the guest image has no renderer; that is a setup
problem to report, not something to work around with a hand-rolled conversion.

## Reading a deck

```bash
ocr /home/codexis/prezentace.pptx        # text and speaker notes
cdx-render -i /home/codexis/prezentace.pptx   # one PNG per slide
```

`Read` the slide images — for a deck, always. Positioning, overlaps and visual hierarchy are
the entire content.

## Creating a deck

Use `pptxgenjs` (preinstalled; `NODE_PATH` is set, so `require('pptxgenjs')` just works).

```javascript
const PptxGenJS = require('pptxgenjs');

const pptx = new PptxGenJS();
pptx.layout = 'LAYOUT_16x9';   // 13.33 × 7.5 inches

const slide = pptx.addSlide();
slide.background = { color: '1E2761' };
slide.addText('Restrukturalizace závazků', {
  x: 0.8, y: 2.4, w: 11.7, h: 1.2,
  fontSize: 40, bold: true, color: 'FFFFFF', fontFace: 'Arial',
});
slide.addText('Návrh postupu • Q3 2026', {
  x: 0.8, y: 3.6, w: 11.7, h: 0.6,
  fontSize: 16, color: 'CADCFC', italic: true,
});

await pptx.writeFile({ fileName: '/home/codexis/prezentace.pptx' });
```

Positions are in inches. Keep at least a 0.5" margin from every slide edge.

## Design — do not produce boring slides

Plain bullets on a white background impress nobody and read as machine-generated.

**Pick a palette that fits the topic.** If you could drop your colours into an unrelated deck
and it would still "work", you have not chosen anything. One colour dominates (60–70% of the
visual weight), one or two support it, one sharp accent. Never give all colours equal weight.

| Theme | Primary | Secondary | Accent |
|---|---|---|---|
| Midnight Executive | `1E2761` | `CADCFC` | `FFFFFF` |
| Charcoal Minimal | `36454F` | `F2F2F2` | `212121` |
| Berry & Cream | `6D2E46` | `A26769` | `ECE2D0` |
| Forest & Moss | `2C5F2D` | `97BC62` | `F5F5F5` |
| Ocean Gradient | `065A82` | `1C7293` | `21295C` |
| Warm Terracotta | `B85042` | `E7E8D1` | `A7BEAE` |

Dark backgrounds for the title and closing slides, light for the content in between — or commit
to dark throughout. Pick one visual motif (rounded image frames, icons in coloured circles, a
thick single-side border) and repeat it on every slide.

**Every slide needs a visual element** — a chart, an icon, a shape, a large stat callout. A
text-only slide is a forgettable slide.

| Element | Size |
|---|---|
| Slide title | 36–44pt bold |
| Section header | 20–24pt bold |
| Body | 14–16pt |
| Caption | 10–12pt, muted |

**Avoid:** the same layout twice in a row; centred body text (centre titles only, left-align
prose); low-contrast text or icons; cramming every inch; styling one slide and leaving the rest
plain. **Never put a thin accent line under a title** — it is the single clearest tell of an
AI-generated deck. Use whitespace or a background colour instead.

## QA — required

**Assume there are problems. Your job is to find them.** Your first render is almost never
correct. If you found zero issues on the first pass, you were not looking.

**1. Content check.**

```bash
ocr /home/codexis/prezentace.pptx
unzip -p /home/codexis/prezentace.pptx-ocr.zip | grep -iE 'lorem|ipsum|xxxx|TODO|\{\{|doplnit'
```

Any hit is a bug. Fix it before going further.

**2. Visual check — use a subagent, even for three slides.** You have been staring at the
coordinates and will see what you meant, not what rendered. A subagent has fresh eyes.

```bash
cdx-render -i /home/codexis/prezentace.pptx
```

Hand it the slide images with this prompt:

```
Visually inspect these slides. Assume there are issues — find them.

Look for:
- Overlapping elements (text through shapes, lines through words)
- Text overflowing its box or clipped at a slide edge
- A decorative line placed for a one-line title that has wrapped to two lines
- Footers or citations colliding with the content above them
- Elements closer than 0.3", or a large empty area next to a cramped one
- Less than 0.5" margin from the slide edge
- Columns that do not line up across slides
- Low-contrast text or icons (light grey on cream, dark icon on a dark background)
- Leftover placeholder content

For each slide, list every issue, including minor ones.
```

**3. Fix, then re-render the affected slides.** One fix routinely breaks the next thing.

```bash
cdx-render -i prezentace.pptx -f 4 -l 4
```

Do not declare success until you have completed at least one full fix-and-verify cycle.

## Dependencies

- `pptxgenjs` (Node, preinstalled) — creating decks
- `cdx-render` — deck → slide PNGs
- `ocr` — text and speaker notes
