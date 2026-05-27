# KDP Book Template System — Specification

## Overview

The `docx-mcp-server` template system provides pre-configured document formats for Amazon KDP publishing across 7 book categories. Each template sets up page size, margins, fonts, styles, headers/footers, and exposes category-specific tools.

## Usage

```json
{"name": "create_document", "arguments": {"title": "My Book", "format": "kdp:technical"}}
{"name": "create_document", "arguments": {"title": "My Book", "format": "kdp:novel"}}
{"name": "create_document", "arguments": {"title": "My Book", "format": "kdp:cookbook"}}
{"name": "create_document", "arguments": {"title": "My Book", "format": "kdp:children"}}
{"name": "create_document", "arguments": {"title": "My Book", "format": "kdp:interior_design"}}
{"name": "create_document", "arguments": {"title": "My Book", "format": "kdp:encyclopedia"}}
{"name": "create_document", "arguments": {"title": "My Book", "format": "kdp:manga"}}
```

---

## Template 1: Technical Book — `kdp:technical`

### Page Setup
- **Trim**: 6" × 9" (standard technical/programming book)
- **Margins**: 0.75" top/bottom, 0.75" outside, 0.875" inside (gutter)
- **Body font**: Garamond 11pt
- **Code font**: Consolas/Courier 9pt
- **Line spacing**: 1.3× (312 twips)
- **First-line indent**: 0.3" on body paragraphs (except first after heading)

### Styles

| Style | Font | Size | Properties |
|-------|------|------|------------|
| ChapterNum | Garamond | 12pt | Centered, small caps, letter-spacing |
| ChapterTitle / Heading1 | Garamond Bold | 24pt | Centered, space before 240, after 240 |
| Heading2 | Garamond Bold | 14pt | Left, space before 360, after 120 |
| Heading3 | Garamond Bold | 12pt | Left, space before 240, after 60 |
| BodyText | Garamond | 11pt | No indent (first para after heading) |
| BodyTextIndent | Garamond | 11pt | 0.3" first-line indent |
| CodeBlock | Consolas | 9pt | Light gray background (#F5F5F5), 0.5pt border, no indent, preserve whitespace |
| CodeInline | Consolas | 10pt | Within body text, no background |
| CalloutTip | Garamond | 10pt | Left border 3pt green, indent 0.3", "TIP:" prefix bold |
| CalloutWarning | Garamond | 10pt | Left border 3pt orange, indent 0.3", "WARNING:" prefix bold |
| CalloutNote | Garamond | 10pt | Left border 3pt blue, indent 0.3", "NOTE:" prefix bold |
| PullQuote | Garamond Italic | 12pt | Indented 0.5" both sides, space above/below |
| FigureCaption | Garamond Italic | 9pt | Centered, "Figure X.X: description" |
| TableCaption | Garamond Italic | 9pt | Left-aligned, "Table X.X: description" |
| Epigraph | Garamond Italic | 10pt | Right-aligned, indented 1.5" from left |
| EpigraphAttribution | Garamond | 9pt | Right-aligned, em-dash prefix |
| TitlePage | Garamond Bold | 28pt | Centered |
| Subtitle | Garamond Italic | 14pt | Centered |
| Author | Garamond | 14pt | Centered |
| Copyright | Garamond | 9pt | Left-aligned, small |

### Running Headers
- **Even (left) pages**: Book title, Garamond Italic 9pt, left-aligned
- **Odd (right) pages**: Chapter title, Garamond Italic 9pt, right-aligned
- **No header on chapter openers**
- **Footer**: Page number centered, Garamond 9pt

### Front Matter Order
1. Half title
2. Title page
3. Copyright
4. Dedication (optional)
5. Table of Contents (linked, heading levels 1-2)
6. Preface / Introduction

### Back Matter Order
1. Appendices
2. Glossary
3. Bibliography / References
4. Index
5. About the Author

### Special Elements
- **Code blocks**: Monospace, shaded, bordered, preserve indentation
- **Callout boxes**: Tip (green), Warning (orange), Note (blue) with left border
- **Tables**: Header row shaded, borders on all cells
- **Figures**: Numbered per chapter (Figure 1.1, 1.2...)
- **Cross-references**: "See Chapter 3" or "See Figure 2.4"
- **Linked TOC**: Auto-generated from Heading1 and Heading2

### Tools Needed
- `insert_code_block(document_handle, code, language?)` — monospace shaded block
- `insert_callout(document_handle, type: tip|warning|note, text)` — bordered callout
- `insert_toc(document_handle)` — linked table of contents

---

## Template 2: Novel (Fiction) — `kdp:novel`

### Page Setup
- **Trim**: 5.25" × 8" (most popular fiction size on KDP)
- **Margins**: 0.75" top, 0.75" bottom, 0.625" outside, 0.875" inside (gutter)
- **Body font**: Garamond 11pt or Baskerville 11pt
- **Line spacing**: 1.3× (312 twips)
- **First-line indent**: 0.3" on all body paragraphs except first after scene break/chapter
- **No space between paragraphs** (indent-only separation — fiction standard)

### Styles

| Style | Font | Size | Properties |
|-------|------|------|------------|
| ChapterNum | Garamond | 12pt | Centered, small caps, letter-spacing 2pt |
| ChapterTitle / Heading1 | Garamond | 24pt | Centered, bold, 1/3 page drop from top |
| BodyText | Garamond | 11pt | No indent (first para after heading/break) |
| BodyTextIndent | Garamond | 11pt | 0.3" first-line indent |
| SceneBreak | Garamond | 11pt | Centered "* * *" or "◆" with spacing above/below |
| DropCap | Garamond | 48pt | First letter, 3-line drop, rest of para normal |
| Epigraph | Garamond Italic | 10pt | Right-aligned, indented 1.5" from left |
| EpigraphAttribution | Garamond | 9pt | Right-aligned, em-dash prefix |
| Dialogue | Garamond | 11pt | Same as body (no special style, just indent) |
| LetterText | Garamond Italic | 10.5pt | Indented 0.5" both sides, italic |
| ThoughtText | Garamond Italic | 11pt | Same as body but italic |
| Dedication | Garamond Italic | 12pt | Centered, 1/3 page drop |
| PartTitle | Garamond Bold | 28pt | Centered, own page |
| PartSubtitle | Garamond Italic | 14pt | Centered, below part title |
| TimeLocation | Garamond | 10pt | Centered, small caps, "LONDON, 1847" |
| TitlePage | Garamond Bold | 28pt | Centered |
| Author | Garamond | 16pt | Centered |
| Copyright | Garamond | 9pt | Left-aligned |

### Running Headers
- **Even (left) pages**: Book title, Garamond Italic 9pt, left-aligned
- **Odd (right) pages**: Chapter title, Garamond Italic 9pt, right-aligned
- **No header on chapter openers** (first page of each chapter)
- **Footer**: Page number centered, Garamond 9pt

### Front Matter Order
1. Half title
2. Also By (list of other books)
3. Title page
4. Copyright
5. Dedication
6. Epigraph (optional book-level quote)
7. Table of Contents (optional for fiction)

### Back Matter Order
1. Acknowledgments
2. About the Author
3. Also By (repeated)
4. Preview of next book (first chapter)

### Special Elements
- **Scene breaks**: "* * *" centered with 1 blank line above/below. At page boundaries, use blank line only (no asterisks)
- **Chapter openers**: Start 1/3 down the page (6-8 blank lines). Optional drop cap on first paragraph
- **Time/location stamps**: Small caps, centered, with spacing (e.g., "LONDON, 1847")
- **Flashbacks**: Can use italic for entire section, or just a scene break with no style change
- **Letters/documents within story**: Indented both sides, italic, different font optional

### Tools Needed
- `insert_scene_break(document_handle, style: "asterisks"|"diamond"|"blank")` — scene separator
- `insert_drop_cap(document_handle, text)` — first letter large, rest normal
- `insert_epigraph(document_handle, quote, attribution)` — chapter-opening quote

---

## Template 3: Cookbook — `kdp:cookbook`

### Page Setup
- **Trim**: 8" × 10" (standard cookbook, allows photos)
- **Margins**: 0.75" all sides, 1" inside (gutter for lay-flat binding)
- **Body font**: Georgia 10.5pt (readable, warm)
- **Heading font**: Gill Sans or Montserrat (clean, modern)
- **Line spacing**: 1.2× (tight for dense content)
- **Columns**: Single column for recipes, optional 2-column for index

### Styles

| Style | Font | Size | Properties |
|-------|------|------|------------|
| RecipeTitle | Gill Sans Bold | 20pt | Left-aligned, bottom border, space after 12pt |
| RecipeSubtitle | Gill Sans Italic | 12pt | Below title (e.g., "Grandma's secret version") |
| PrepInfo | Gill Sans | 9pt | Gray text, "Prep: 15 min | Cook: 45 min | Serves: 4" |
| SectionLabel | Gill Sans Bold | 10pt | All caps, letter-spacing, e.g., "INGREDIENTS" |
| IngredientList | Georgia | 10pt | Bulleted, hanging indent 0.25", em-dash bullets |
| InstructionStep | Georgia | 10.5pt | Numbered, hanging indent 0.3", space between 6pt |
| ChefTip | Georgia Italic | 9.5pt | Left border 2pt gold, indent 0.3", cream background |
| ChefNote | Gill Sans Italic | 9pt | Parenthetical (e.g., "Note: Can substitute...") |
| NutritionFacts | Gill Sans | 8pt | Bordered box, 2-column layout inside |
| ServingSize | Gill Sans Bold | 9pt | Right-aligned badge style |
| PhotoCaption | Gill Sans Italic | 8.5pt | Centered below image |
| ChapterIntro | Georgia Italic | 11pt | No indent, first para (personal story) |
| VariationBox | Georgia | 9.5pt | Bordered box, "VARIATIONS:" header |
| TitlePage | Gill Sans Bold | 32pt | Centered |
| Author | Gill Sans | 16pt | Centered |
| Copyright | Georgia | 8.5pt | Left-aligned |

### Running Headers
- **Even pages**: Book title, Gill Sans 8pt, left
- **Odd pages**: Chapter/section name, Gill Sans 8pt, right
- **Footer**: Page number centered

### Front Matter
1. Half title
2. Full-page hero photo (signature dish)
3. Title page
4. Copyright
5. Dedication ("For Mom, who taught me that love is the secret ingredient")
6. Table of Contents (by chapter/section)
7. Introduction (author's food philosophy, 2-4 pages)
8. Pantry Essentials (list of must-have ingredients)
9. Equipment Guide

### Chapter Structure
1. Chapter opener: Full-page photo + chapter title overlay
2. Chapter intro: 1-2 paragraphs of story/context
3. Recipes: Each gets own page (or spread)

### Recipe Layout (single page)
```
[Recipe Title]
[Subtitle]
[Prep: 15 min | Cook: 45 min | Serves: 4]

INGREDIENTS
— 2 cups all-purpose flour
— 1 tsp baking soda
— ½ cup unsalted butter

INSTRUCTIONS
1. Preheat oven to 375°F...
2. In a large bowl, whisk...
3. Fold in chocolate chips...

[Chef's Tip box]
[Nutrition Facts box (optional)]
```

### Back Matter
1. Conversion charts (cups→grams, F→C)
2. Seasonal ingredient guide
3. Index (alphabetical by recipe name + ingredient)
4. About the Author + photo
5. Acknowledgments

### Tools Needed
- `insert_recipe(document_handle, title, subtitle?, prep_time, cook_time, servings, ingredients[], instructions[])` — full recipe layout
- `insert_ingredient_list(document_handle, items[])` — formatted ingredient list
- `insert_chef_tip(document_handle, text)` — bordered tip box
- `insert_nutrition_facts(document_handle, calories, fat, protein, carbs, ...)` — nutrition box

---

## Template 4: Children's Book — `kdp:children`

### Page Setup
- **Trim**: 8.5" × 8.5" (square, standard picture book)
- **Margins**: 0.5" all sides (maximize image area)
- **Body font**: Sassoon Primary 16-20pt (designed for children's reading)
- **Fallback font**: Century Schoolbook or Bookman Old Style 18pt
- **Line spacing**: 1.5× (generous for young readers)
- **Text per page**: 1-4 sentences maximum (ages 3-7)

### Styles

| Style | Font | Size | Properties |
|-------|------|------|------------|
| NarratorText | Sassoon/Century | 18pt | Centered or left, max 3 lines per page |
| BigText | Sassoon Bold | 24pt | Centered, for emphasis words ("ROAR!") |
| CharacterDialogue | Sassoon | 16pt | With character name in bold, speech marks |
| SoundEffect | Sassoon Bold Italic | 28pt | Centered, all caps |
| PageNumber | Sassoon | 10pt | Bottom center, subtle |
| TitlePage | Sassoon Bold | 36pt | Centered, mid-page |
| AuthorName | Sassoon | 14pt | Centered, below title |
| IllustratorCredit | Sassoon Italic | 12pt | Centered, "Illustrated by..." |
| Dedication | Sassoon Italic | 12pt | Centered, simple |
| EndMatter | Sassoon | 11pt | "About the Author" page |

### Page Layout Patterns
- **Full spread**: Image fills entire page, text overlaid in white box or at bottom
- **Half-half**: Top half image, bottom half text (or left/right)
- **Text-only**: Rare, used for dramatic pause or title pages
- **Vignette**: Small image centered with text wrapped around

### Structure (32 pages standard)
1. Page 1: Half title
2. Page 2: Full title + author + illustrator
3. Page 3: Copyright (tiny text, parent-facing)
4. Page 4: Dedication
5. Pages 5-30: Story (26 pages, 13 spreads)
6. Page 31: "The End" or moral/question
7. Page 32: About Author/Illustrator

### Design Rules
- **One idea per spread** (2 pages)
- **Page turns = suspense** (end each spread with anticipation)
- **Repetition** (children love repeated phrases)
- **Large images** (70%+ of page area)
- **High contrast** text on image (dark text on light area, or text box)
- **No justified text** (left-aligned or centered only)
- **No hyphenation**

### Tools Needed
- `insert_spread(document_handle, image_path, text, text_position: "bottom"|"top"|"overlay")` — full-page image + text
- `insert_big_text(document_handle, text)` — large emphasis text
- `insert_sound_effect(document_handle, text)` — stylized SFX

---

## Template 5: Interior Design Book — `kdp:interior_design`

### Page Setup
- **Trim**: 8.5" × 11" (large format, photo-forward)
- **Margins**: 0.75" top/bottom, 0.875" inside, 0.75" outside
- **Body font**: Minion Pro 10pt (elegant serif)
- **Heading font**: Futura or Avenir 12-24pt (modern sans)
- **Line spacing**: 1.25×
- **Columns**: 2-column for text sections, full-width for photos

### Styles

| Style | Font | Size | Properties |
|-------|------|------|------------|
| RoomTitle | Futura Bold | 24pt | Left-aligned, bottom rule |
| RoomSubtitle | Futura Light | 14pt | Below title, describes the space |
| BodyText | Minion | 10pt | 2-column, justified |
| DesignPrinciple | Futura Bold | 11pt | Numbered, with explanation below |
| MaterialSpec | Minion | 9pt | Table format: Material / Source / Price |
| ColorSwatch | — | — | Small colored rectangle + hex code + name |
| BudgetBox | Futura + Minion | 9pt | Bordered, "BUDGET: $$$" header, line items |
| BeforeAfterCaption | Futura | 8pt | Below paired images, "Before" / "After" |
| DesignTip | Minion Italic | 9.5pt | Left border 3pt, indented, "PRO TIP:" prefix |
| SourceList | Minion | 8.5pt | "WHERE TO BUY:" + vendor list |
| PhotoCaption | Futura Light Italic | 8pt | Below images |
| PullQuote | Futura Light | 16pt | Large, centered, decorative quotes |
| Sidebar | Minion | 9pt | Shaded box, "DID YOU KNOW?" or "TREND ALERT" |
| ChapterIntro | Minion | 11pt | First para, no indent, slightly larger |
| TitlePage | Futura Bold | 36pt | Centered |
| Author | Futura Light | 16pt | Centered |
| Copyright | Minion | 8pt | Left-aligned |

### Page Layout Patterns
- **Hero spread**: Full-bleed photo across 2 pages, title overlay
- **Grid layout**: 4-6 photos in grid with captions
- **Before/After**: Side-by-side comparison
- **Mood board page**: Collage of textures, colors, furniture
- **Floor plan page**: Diagram with callouts
- **Shopping list**: Table with items, sources, prices

### Chapter Structure
1. Full-page room hero photo
2. Chapter title + intro paragraph
3. Design principles for this space (3-5 numbered)
4. Photo gallery (4-8 images with captions)
5. Materials & sources table
6. Budget breakdown box
7. Pro tips sidebar
8. Before/after (if renovation)

### Front/Back Matter
- **Front**: Mood board collage, title, author bio with photo, "How to Use This Book"
- **Back**: Resource directory, vendor index, color palette reference, acknowledgments

### Tools Needed
- `insert_photo_grid(document_handle, images[], captions[], columns: 2|3)` — image grid with captions
- `insert_budget_box(document_handle, title, items[{name, cost}], total)` — bordered budget breakdown
- `insert_design_tip(document_handle, text)` — bordered pro tip
- `insert_before_after(document_handle, before_image, after_image, caption)` — side-by-side comparison

---

## Template 6: Encyclopedia — `kdp:encyclopedia`

### Page Setup
- **Trim**: 8.5" × 11" (reference format)
- **Margins**: 0.625" top/bottom, 0.75" outside, 1" inside
- **Body font**: Minion Pro 9.5pt (dense but readable)
- **Heading font**: Myriad Pro (clean sans-serif)
- **Line spacing**: 1.15× (tight for density)
- **Columns**: 2-column body, full-width for diagrams/tables

### Styles

| Style | Font | Size | Properties |
|-------|------|------|------------|
| AlphaHeader | Myriad Bold | 36pt | Large letter at start of section ("A") |
| EntryTitle | Myriad Bold | 12pt | Bold, followed by pronunciation/category |
| EntryCategory | Myriad Italic | 9pt | In brackets after title, e.g., "[Computing]" |
| Definition | Minion | 9.5pt | First sentence bold (the definition), rest normal |
| CrossReference | Myriad | 8.5pt | "See also:" + linked terms, italic |
| InfoBox | Minion | 9pt | Bordered box, colored header bar, key facts |
| Timeline | Myriad | 8.5pt | Left-aligned dates, right-aligned events |
| FactPanel | Minion | 9pt | Shaded sidebar, "KEY FACTS" header |
| QuickStats | Myriad | 8pt | Table: Stat / Value pairs |
| FigureCaption | Myriad Italic | 8pt | Below diagrams/images |
| TableHeader | Myriad Bold | 9pt | Shaded row, white text |
| Pronunciation | Myriad Italic | 9pt | IPA in slashes, e.g., /ˈprəʊtəkɒl/ |
| RelatedEntries | Myriad | 8.5pt | Comma-separated linked terms |
| PageHeader | Myriad | 8pt | First entry – Last entry on page (dictionary style) |
| TitlePage | Myriad Bold | 36pt | Centered |
| Copyright | Minion | 8pt | Left-aligned |

### Page Layout
- **Running header**: First entry on left page – Last entry on right page (dictionary style)
- **Thumb index**: Shaded tab on page edge showing current letter
- **2-column text** with occasional full-width info boxes or diagrams
- **Cross-references** throughout (bold terms link to their own entries)

### Entry Structure
```
PROTOCOL [Computing] /ˈprəʊtəkɒl/

A set of rules governing the exchange of data between
devices or systems. In networking, protocols define how
data is formatted, transmitted, and received...

KEY FACTS
━━━━━━━━━━━━━━━━━━━━
First use:     1970s (ARPANET)
Examples:      HTTP, TCP/IP, MCP
Related:       API, Interface, Standard

See also: API, HTTP, Model Context Protocol, TCP/IP
```

### Front/Back Matter
- **Front**: Title, editorial board, "How to Use This Encyclopedia", abbreviations key
- **Back**: Full index, bibliography, contributor list, timeline of the field

### Tools Needed
- `insert_entry(document_handle, title, category, pronunciation?, definition, key_facts{})` — encyclopedia entry
- `insert_info_box(document_handle, title, content)` — bordered fact box
- `insert_cross_reference(document_handle, terms[])` — "See also" links

---

## Template 7: Manga/Comic — `kdp:manga`

### Page Setup
- **Trim**: 5" × 7.5" (B6 standard manga size) or 6.625" × 10.25" (US comic)
- **Margins**: 0.25" (bleed area), 0.5" safe zone for text
- **Dialogue font**: CC Wild Words or Manga Temple 9-10pt
- **Narration font**: CC Astro City or Times New Roman Italic 8pt
- **SFX font**: Custom/hand-drawn style, 14-48pt
- **Reading direction**: Right-to-left (manga) or left-to-right (western comic)

### Styles

| Style | Font | Size | Properties |
|-------|------|------|------------|
| Dialogue | CC Wild Words | 10pt | Centered in speech bubble, all caps |
| Whisper | CC Wild Words | 8pt | Centered, smaller, dashed bubble border |
| Shout | CC Wild Words Bold | 12pt | Centered, bold, jagged bubble border |
| Thought | CC Wild Words Italic | 9pt | Centered, cloud bubble |
| Narration | CC Astro City | 8.5pt | Rectangular box, top or bottom of panel |
| SFX_Large | Impact/custom | 36pt | Rotated, integrated into art, bold |
| SFX_Small | Impact/custom | 18pt | Smaller effects (footsteps, etc.) |
| PanelDescription | Courier | 9pt | Script-only: describes the panel art |
| CharacterName | CC Wild Words Bold | 8pt | Above dialogue in script format |
| PageDirection | Myriad | 7pt | "Read right to left →" on first page |
| ChapterTitle | Custom display | 24pt | Stylized, on chapter cover page |
| VolumeTitle | Custom display | 36pt | Cover page |
| Credits | Myriad | 8pt | Creator, editor, letterer credits |

### Panel Layout Templates
- **Full page**: Single panel, dramatic moment
- **6-panel grid**: 2×3 standard layout
- **4-panel strip**: Horizontal, comedic timing
- **L-shaped**: Large panel + 2-3 smaller
- **Diagonal split**: Dynamic action scenes
- **Borderless**: Panel bleeds to edge (emotional moments)

### Page Structure
- **Cover page**: Full illustration + title + volume number
- **Inside cover**: Character profiles or "Story so far"
- **Chapter cover**: Full-page art + chapter number/title
- **Story pages**: 4-7 panels per page average
- **Between chapters**: Author notes, sketches, bonus content

### Script Format (for the MCP tool)
```
PAGE 1
PANEL 1 (full width, top third)
[Description: Wide shot of city skyline at night]
NARRATION: "In a world where AI agents roam free..."

PANEL 2 (left half)
[Description: Close-up of character at computer]
CHARACTER: "The server is responding."

PANEL 3 (right half)
[Description: Screen showing MCP protocol]
SFX: PING!
CHARACTER: "We're connected."
```

### Manga-Specific Features
- **Speed lines**: Described in panel descriptions
- **Tone/screentone**: Referenced for print production
- **Gutter spacing**: 0.125" between panels
- **Bleed panels**: Extend 0.125" past trim
- **Sound effects**: Integrated into art, not in bubbles

### Tools Needed
- `insert_panel(document_handle, layout: "full"|"half"|"third", description)` — panel placeholder
- `insert_dialogue(document_handle, character, text, style: "normal"|"whisper"|"shout"|"thought")` — speech
- `insert_sfx(document_handle, text, size: "small"|"large")` — sound effect
- `insert_narration(document_handle, text)` — narrator box

---

## Implementation Summary

| Template | Trim | Unique Tools | Priority | Complexity |
|----------|------|-------------|----------|------------|
| Technical | 6×9 | 3 (code_block, callout, toc) | Phase 1 | Medium |
| Novel | 5.25×8 | 3 (scene_break, drop_cap, epigraph) | Phase 1 | Low |
| Cookbook | 8×10 | 4 (recipe, ingredients, chef_tip, nutrition) | Phase 2 | Medium |
| Children's | 8.5×8.5 | 3 (spread, big_text, sound_effect) | Phase 2 | Low |
| Interior Design | 8.5×11 | 4 (photo_grid, budget_box, design_tip, before_after) | Phase 3 | High |
| Encyclopedia | 8.5×11 | 3 (entry, info_box, cross_ref) | Phase 3 | Medium |
| Manga | 5×7.5 | 4 (panel, dialogue, sfx, narration) | Phase 3 | High |

### Shared Infrastructure
- Template registry (select by format string)
- Shared base styles (Copyright, TitlePage, Author)
- Page setup per template (trim, margins, columns)
- Running header system (odd/even, suppress on openers)
- TOC generation (linked, from headings)
- Front/back matter scaffolding

### Phased Delivery
- **Phase 1**: Technical + Novel (highest KDP demand, establishes pattern)
- **Phase 2**: Cookbook + Children's (visual-heavy, different layouts)
- **Phase 3**: Interior Design + Encyclopedia + Manga (niche, complex)
