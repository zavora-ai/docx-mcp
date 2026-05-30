# OOXML Feature Implementation Spec

Roadmap for closing the remaining WordprocessingML (ECMA-376) gaps in
**zavora-docx** (the library) and **docx-mcp** (the MCP server). Ordered by user
impact. Each feature lists the OOXML target, the library work, the MCP tool, and
acceptance tests.

## Conventions

- **Library layers**: `rdocx-oxml` (typed CT_/ST_ models + parse/serialize) →
  `zavora-docx` (high-level builder API) → `docx-mcp` (MCP tools).
- **Round-trip rule**: every new `from_xml` must capture unknown children into
  `extra_xml` (or a `RawXml` variant) and re-emit them. No silent drops.
- **No-op defaults**: new optional fields must serialize identically to today
  when unset (protect existing golden output).
- **Namespaces**: declare new namespaces (`m:`, `c:`, `wps:`, `w14:`) on the
  document root via the existing `extra_namespaces` mechanism, or locally on the
  element when self-contained.
- **Testing**: each feature ships (a) a parse→serialize round-trip test in
  `rdocx-oxml`, (b) a builder test in `zavora-docx`, (c) an end-to-end MCP JSON-RPC
  check that inspects the emitted XML. Verify in Word manually for visual features.
- **Each phase = one commit per repo** with tests green before moving on.

---

## Phase 1 — Cheap wins (settings, language, metadata)

### 1.1 Broaden `settings.xml`

**OOXML**: `w:settings` children — `w:defaultTabStop`, `w:zoom`, `w:mirrorMargins`,
`w:proofState`, `w:trackChanges`, `w:compat`, `w:defaultLanguage` (`w:themeFontLang`),
`w:bordersDoNotSurroundHeader/Footer`, `w:doNotExpandShiftReturn`.

**Library (rdocx-oxml)**:
- New `settings.rs` with `CT_Settings { update_fields, even_odd_headers,
  auto_hyphenation, protection, default_tab_stop: Option<Twips>, mirror_margins:
  bool, track_changes: bool, zoom_percent: Option<u32>, theme_font_lang:
  Option<(String,String,String)>, extra_xml: Vec<Vec<u8>> }`.
- Move the inline settings serialization out of `document.rs` into `CT_Settings::to_xml`.
- `from_xml` capturing unknown children → `extra_xml` (so opened docs round-trip).

**zavora-docx**: `Document::set_default_tab_stop(Length)`, `set_mirror_margins(bool)`,
`set_track_changes(bool)`, `set_zoom(u32)`, `set_document_language(&str)`.

**MCP**: extend a `set_document_settings` tool with optional params for each.

**Tests**: round-trip a settings.xml with `mirrorMargins` + `defaultTabStop`;
assert builder emits them; assert opened unknown setting survives.

### 1.2 Settable proofing language (`w:lang`)

**OOXML**: `w:rPr/w:lang@w:val` (+ `w:eastAsia`, `w:bidi`).

**Library**: promote to a typed `CT_RPr.lang: Option<Lang { val, east_asia, bidi }>`
field (already round-trips as raw — replace with typed field, keep raw fallback for
other unknowns). Run builder `Run::language(&str)`.

**MCP**: `language` param on `insert_run`.

**Tests**: `Run::language("fr-FR")` emits `<w:lang w:val="fr-FR"/>`; round-trip.

### 1.3 Extended + app metadata (`docProps/app.xml`)

**OOXML**: `Properties` (Extended) — `Company`, `Application`, `AppVersion`,
`Pages`, `Words`, `Characters`, `Template`.

**Library**: new `app_properties.rs` `CT_AppProperties`; write on save; register
content-type + relationship.

**zavora-docx**: `Document::set_company(&str)`, `set_application(&str)`.

**MCP**: params on a `set_metadata` tool.

**Tests**: app.xml present after save; company round-trips.

---

## Phase 2 — Content controls (SDT) authoring

**OOXML**: `w:sdt` = `w:sdtPr` (`w:alias`, `w:tag`, `w:id`, type:
`w:text`/`w:richText`/`w:dropDownList`/`w:comboBox`/`w:date`/`w:checkbox`(w14)) +
`w:sdtContent` (block or inline).

**Library (rdocx-oxml)**:
- `sdt.rs`: `CT_Sdt { pr: CT_SdtPr, content: Vec<BlockContent> }`, `CT_SdtPr {
  alias, tag, id, lock, kind: SdtKind, placeholder, extra_xml }`.
- `SdtKind` enum: `Text`, `RichText`, `DropDown(Vec<(display,value)>)`,
  `ComboBox(...)`, `Date{format}`, `Checkbox{checked}`, `Picture`.
- Currently SDTs are preserved as `BodyContent::RawXml` / `CellContent::RawXml`.
  Add a `BodyContent::Sdt(CT_Sdt)` variant **only for constructed** controls;
  keep RawXml for parsed-unknown to avoid lossy re-modeling. (Parse path may stay
  raw initially; construction path is the goal.)

**zavora-docx**: `Document::add_text_control(tag, placeholder)`,
`add_dropdown_control(tag, options)`, `add_date_control(tag, format)`,
`add_checkbox_control(tag, checked)`, `add_rich_text_control(tag)`.

**MCP**: `add_content_control` tool — `{ kind, tag, alias?, placeholder?, options?,
checked?, date_format? }`.

**Tests**: each control kind emits valid `w:sdt` with correct `w:sdtPr`; dropdown
lists options; Word opens without repair.

---

## Phase 3 — Math (OMML)  *(highest-impact missing feature)*

**OOXML**: `m:oMath` / `m:oMathPara`. Core elements: `m:r` (math run), `m:f`
(fraction num/den), `m:sup`/`m:sub`/`m:sSubSup`, `m:rad` (radical), `m:nary`
(integral/sum with `m:naryPr`), `m:d` (delimiters), `m:func`, `m:m` (matrix),
`m:acc` (accent), `m:bar`, `m:eqArr`. Math text uses `m:t`. Requires `xmlns:m` on root.

**Library (rdocx-oxml)**:
- `math.rs`: `CT_OMath { elements: Vec<MathNode> }`, recursive `MathNode` enum:
  `Run(String)`, `Fraction{num,den}`, `Sup{base,sup}`, `Sub{base,sub}`,
  `SubSup{base,sub,sup}`, `Radical{deg:Option,radicand}`, `Nary{op,sub,sup,operand}`,
  `Delimiter{beg,end,items}`, `Func{name,arg}`, `Matrix{rows}`, `Accent{chr,base}`,
  `Bar{base}`, `Group(Vec<MathNode>)`, `Raw(Vec<u8>)` (round-trip catch-all).
- Add `RunContent::Math(CT_OMath)` and/or `BodyContent` paragraph-level `oMathPara`.
- `from_xml`: recursive parse; unknown math elements → `Raw`. `to_xml`: emit `m:`-prefixed.

**zavora-docx**: a small builder DSL — `Math::frac(a,b)`, `Math::sup(base,e)`,
`Math::sqrt(x)`, `Math::nary("∑",lo,hi,body)`, `Math::run("x")`, `Math::delim(...)`.
`Paragraph::add_math(MathNode)` and `Document::add_equation(MathNode)`.
- **Stretch**: a LaTeX-subset → MathNode parser (`Math::from_latex("\\frac{a}{b}")`)
  covering fractions, sup/sub, roots, sums/integrals, Greek letters, common operators.

**MCP**: `add_equation` tool — accepts either a structured JSON math tree **or**
`latex` string; converts to OMML.

**Tests**: `frac(1,2)` → `<m:f><m:num>…`; LaTeX `\frac{a}{b}^2` round-trips to
expected OMML; opens in Word as a real editable equation.

---

## Phase 4 — Shapes & text boxes (DrawingML wps)

**OOXML**: `wps:wsp` (shape) inside `a:graphicData uri=".../wordprocessingShape"`,
with `wps:spPr` (geometry `a:prstGeom`, fill, line), `wps:txbx/w:txbxContent`
(paragraphs inside the shape), `wps:bodyPr`. Anchored or inline via existing
`CT_Anchor`/`CT_Inline` wrappers.

**Library (rdocx-oxml)**:
- Extend `drawing.rs`: `GraphicContent` enum = `Picture(...)` | `Shape(CT_Shape)`
  (today it's implicitly always picture). `CT_Shape { geom: PresetGeom, fill:
  Option<Fill>, line: Option<Line>, text: Vec<CT_P>, props: PicProps }`.
- `write_graphic_element` becomes shape-aware (picture vs wsp graphicData uri).

**zavora-docx**: `Document::add_text_box(width,height, paragraphs)`,
`add_shape(PresetGeom, fill, …)`. Reuse `PicProps` for rotation/border/shadow.

**MCP**: `add_text_box` and `add_shape` tools.

**Tests**: text box emits `wps:wsp` + `w:txbxContent` with the paragraph text;
preset shape (rect, ellipse, roundRect, arrow) geometry correct.

---

## Phase 5 — Charts (DrawingML c:chart)

**OOXML**: separate part `word/charts/chart1.xml` (`c:chartSpace` → `c:chart` →
`c:plotArea` → series), referenced from a `graphicData uri=".../chart"` +
relationship. Chart types: bar, column, line, pie, area, scatter.

**Library (rdocx-oxml)**:
- `chart.rs`: `CT_Chart { kind: ChartKind, title, categories: Vec<String>,
  series: Vec<Series{name, values: Vec<f64>}> }`; `to_xml` → chartN.xml.
- Document save: allocate chart part name, content-type
  `…drawingml.chart+xml`, relationship, embed a `graphicData` drawing referencing it.

**zavora-docx**: `Document::add_chart(ChartKind, categories, series, size)`.

**MCP**: `add_chart` tool — `{ kind, categories[], series[{name,values[]}], title?,
width?, height? }`.

**Tests**: chart part created + referenced; pie/bar/line series values present;
Word renders an editable chart.

---

## Phase 6 — Threaded comments & typed track-changes

**OOXML**: `commentsExtended.xml` (`w15:commentEx` parentId/done),
`people.xml` (`w15:person`), `commentsIds.xml`. Track changes: typed `w:ins`/`w:del`
already partial; add `w:rPrChange`/`w:pPrChange` typed models.

**Library**: extend comments to carry author/date/parent/resolved; new parts on save.
Typed `RPrChange`/`PPrChange` (currently raw-preserved).

**zavora-docx**: `Document::add_comment_reply(parent_id, …)`, `resolve_comment(id)`.

**MCP**: `reply_to_comment`, `resolve_comment` tools.

**Tests**: reply links to parent; resolved flag set; round-trip.

---

## Phase 7 — Remaining parts & typed bookmarks/fields

- **`fontTable.xml`**: declare fonts used; optional font embedding (`w:embedRegular`
  + binary font part). Library `font_table.rs`; `Document::declare_font`.
- **Typed bookmarks**: `CT_Bookmark` model + `Document::add_bookmark(name, range)` /
  `cross_reference(name)` instead of raw strings.
- **General fields**: `CT_Field` with instruction + cached result for arbitrary
  fields (REF, SEQ, STYLEREF, DATE, etc.) beyond PAGE/NUMPAGES.
- **`glossary/document.xml`**: building blocks (low priority).
- **Custom XML data binding** (`customXml/`): low priority.

---

## Cross-cutting tasks

- **Namespace registry**: central helper to add `m:`, `c:`, `wps:`, `w15:` to the
  document root only when the corresponding feature is used.
- **MCP tool docs**: every new tool gets a precise description with the
  "call order" guidance pattern already used by `add_toc`.
- **`mcp-server.toml`**: register new tools with risk class.
- **Golden-file tests**: add a corpus of real Word-authored .docx (math, chart,
  SDT, shapes) under `tests/corpus/` and assert load→save is byte-stable for the
  parts we fully model and structurally-valid elsewhere.

## Suggested order

1 → 2 → 3 (math) → 5 (charts) → 4 (shapes) → 6 → 7. Phases 1–2 are low-risk
foundations; math and charts are the headline features; shapes/comments/parts
round out parity.
