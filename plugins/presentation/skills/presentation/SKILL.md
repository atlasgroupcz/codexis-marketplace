---
uuid: a26db6f3-54c7-46e4-8881-f39edfb75f19
name: presentation
description: >-
  Generate PowerPoint presentations (PPTX) using python-pptx.
  Triggers on "presentation", "prezentace", "slides", "slideshow",
  "powerpoint", "pptx", "udělej prezentaci", "create slides", "make a presentation".
allowed-tools: shell
i18n:
  cs:
    displayName: "Tvorba prezentací"
    summary: "Generování prezentací PowerPoint (PPTX) z textových podkladů pomocí python-pptx."
  en:
    displayName: "Presentation Generator"
    summary: "Generate PowerPoint (PPTX) presentations from text briefs using python-pptx."
  sk:
    displayName: "Tvorba prezentácií"
    summary: "Generovanie prezentácií PowerPoint (PPTX) z textových podkladov pomocou python-pptx."
---

# Presentation Generator (PPTX)

Generate PowerPoint files using `python-pptx`.

## Setup

```bash
pip3 install python-pptx --quiet
```

## Slide Layouts

| Index | Layout | Use for |
|-------|--------|---------|
| 0 | Title Slide | First slide, section dividers |
| 1 | Title + Content | Standard bullet slides |
| 5 | Blank | Tables, images, custom layouts |
| 6 | Title Only | Slide with title, custom body |

## Common Patterns

### Boilerplate

```python
from pptx import Presentation
from pptx.util import Inches, Pt, Emu
from pptx.enum.text import PP_ALIGN
from pptx.dml.color import RGBColor

prs = Presentation()
prs.slide_width = Emu(12192000)   # 16:9
prs.slide_height = Emu(6858000)
```

### Title slide

```python
slide = prs.slides.add_slide(prs.slide_layouts[0])
slide.shapes.title.text = "Presentation Title"
slide.placeholders[1].text = "Subtitle — Date"
```

### Bullet slide

```python
slide = prs.slides.add_slide(prs.slide_layouts[1])
slide.shapes.title.text = "Key Points"
tf = slide.placeholders[1].text_frame
tf.text = "First point"
for text in ["Second point", "Third point"]:
    p = tf.add_paragraph()
    p.text = text
    p.level = 0
```

### Nested bullets

```python
tf = slide.placeholders[1].text_frame
tf.text = "Main point"
p = tf.add_paragraph()
p.text = "Sub-point"
p.level = 1
```

### Table slide

```python
slide = prs.slides.add_slide(prs.slide_layouts[6])
slide.shapes.title.text = "Comparison"
table = slide.shapes.add_table(
    rows=3, cols=3,
    left=Inches(0.5), top=Inches(1.8),
    width=Inches(9), height=Inches(3)
).table
table.cell(0, 0).text = "Header"
```

### Styled text

```python
run = p.add_run()
run.text = "Bold colored text"
run.font.bold = True
run.font.size = Pt(18)
run.font.color.rgb = RGBColor(0x00, 0x66, 0xCC)
```

### Image

```python
slide = prs.slides.add_slide(prs.slide_layouts[5])
slide.shapes.add_picture("image.png", Inches(1), Inches(1), width=Inches(8))
```

### Speaker notes

```python
slide.notes_slide.notes_text_frame.text = "Speaker notes here"
```

### Save

```python
prs.save("/home/codexis/output.pptx")
```

## Guidelines

- 16:9 aspect ratio
- Titles: 28-36pt, body: 18-24pt
- 3-5 bullets per slide max
- Short topic: 5-8 slides, standard: 10-15 slides
- Always include: title slide → content slides → closing slide

## Workflow

1. Install `python-pptx` if not present
2. Write a Python script that builds the presentation
3. Run it via shell
4. Report the output file path to the user

## Output

File path: `{workDir}/{descriptive-name}.pptx`
