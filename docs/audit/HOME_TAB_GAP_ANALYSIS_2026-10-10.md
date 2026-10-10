# Home tab gap analysis + reported-bug triage (2026-10-10)

Source: user's Word Home-tab screenshot (side chat, 2026-10-10 ~18:49 PKT) + the
user's in-chat commission: "add all these tools in home tab, find all missing tools…
ctrl+A and all other shortcuts not working… increase/decrease font size buttons not
working… changing fonts on all selected text only changes the font of the last written
text in urdu… find all other issues and fix them".

Method: read `crates/ui-egui/src/ribbon.rs::home()` (lines 134–330) tool-by-tool against
the screenshot; reproduced the three reported bugs headlessly with the debug CLI
(`target/debug/goharscribe-cli run … --save`, inspected with `inspect`).

## 1. Home tab: nothing missing — the tools are already there

| Word group (screenshot) | GoharScribe Home tab | Status |
|---|---|---|
| Clipboard: Paste(⌄), Cut, Copy, Format Painter | Paste (menu: Paste, Keep Text Only), Cut, Copy, Format Painter | ✅ complete |
| Font: font ⌄, size ⌄, A⁺, A⁻, Aa⌄ (change case), clear formatting | font/size combos, format.growFont, format.shrinkFont, change-case menu (5 modes), format.clear | ✅ complete |
| Font row 2: B I U(⌄), strike, x₂, x², text effects, highlight ⌄, font color ⌄ | format.bold/italic/underline(+8 styles)/strikethrough/subscript/superscript, effects menu, highlight split-button w/ color grid, font-color split-button | ✅ complete |
| Paragraph: bullets ⌄, numbering ⌄, multilevel ⌄, outdent, indent, sort, ¶ | para.bullets/numbering/multilevel (all with libraries), para.outdent/indent, para.sort, view.marks | ✅ complete |
| Paragraph row 2: align L/C/R/J, line spacing ⌄, shading ⌄, borders ⌄ | para.alignLeft/Center/Right/justify, line-spacing menu, shading split, borders split | ✅ complete |
| Styles gallery (Normal, No Spacing, Heading 1/2, Title, Subtitle, Subtle Em…) | `previews::style_gallery` (dynamic from `doc.styles.gallery()`) | ✅ complete |
| Editing: Find ⌄, Replace, Select ⌄ | ui.dialog find/replace, Select menu (All/Paragraph/Sentence) | ✅ complete |
| Adobe Acrobat group (Create Adobe PDF, Request Signatures) | — | N/A: third-party add-in, not Word core. GoharScribe exports PDF natively. Do NOT copy. |

**Honest verdict for the user:** the Home tab does not need new tools. Every control in
their screenshot exists and is wired to a real engine command. The real work is that
three of those controls are broken at the UI layer (see §3). Telling the user "nothing
to add" matters — it reframes the fix pass correctly.

## 2. Engine commands verified working (headless reproduction, this session)

- `select.all` → selects whole doc ✅
- `select.all` + `format.growFont` → every run grew (sample: all runs → 36.0) ✅
- `select.all` + `format.shrinkFont` → shrank back ✅
- Urdu text (`"یہ اردو متن ہے"` + `" اور یہ مزید"`), `select.all` + `format.font={"name":"Noto Nastaliq Urdu"}` → the whole range got the font (single merged run 0..46, `font` set) ✅
- `Paragraph::format` splits runs at range boundaries correctly (`crates/doc/src/para.rs:504`); `format_range` covers all paragraphs in range (`crates/doc/src/edit.rs:161`).

(engine unit tests cover the same paths: `crates/engine/src/tests.rs:390-412`.)

**Conclusion: all three reported bugs live in `crates/ui-egui`, not the engine.**

## 3. Prime suspects for the three reported bugs (UI layer)

### (a) Ctrl+A and other shortcuts "not working"
- `select.all` is registered `Mod+A` (`crates/engine/src/cmd/caret.rs:42`); `keys.rs::dispatch`
  normalizes Ctrl→Mod on Windows and `by_shortcut` matches — plumbing is correct.
- BUT `canvas_events` only runs when `app.canvas.focused` (`canvas.rs:346`), and
  `global_shortcuts` bails when the canvas isn't focused AND egui wants keyboard input
  (`keys.rs`, `global_shortcuts`).
- The user's flow is ribbon-heavy (click font combo → press Ctrl+A). After any ribbon
  click the canvas loses focus; if focus sits in the font/size combo's text field,
  Ctrl+A selects the field's text instead of the document. This exactly matches
  "shortcuts not working" right after using the ribbon.
- Check: log `app.canvas.focused` + `ctx.egui_wants_keyboard_input()` after a ribbon
  click on Windows; consider routing Mod+A/C/V/X/Z through `global_shortcuts`
  regardless of text-field focus, or restoring canvas focus after ribbon actions.

### (b) Increase/Decrease Font Size buttons "not working"
- Wiring is correct: `ribbon.rs` home() → `small(…, "format.growFont"/"format.shrinkFont", …)`;
  ids exist in the 389-command registry; default `enabled: always`
  (`crates/engine/src/lib.rs`, `CommandSpec::new`).
- Engine proven working (§2). So the click either never reaches `app.run`, or
  `app.run` errors and the error is swallowed: `widgets.rs::small()` does
  `if resp.clicked() && on { let _ = app.run(id, params); }` — every ribbon button
  discards errors. "Not working" is currently undiagnosable by design.
- Check: does the button render enabled (accent) or disabled (grey)? If disabled,
  `enabled()` is lying; if enabled-but-dead, `app.run` is failing — surface the error.

### (c) Font change on selection only changes "the last written text" (Urdu)
- Engine `format.font` applies to the full selected range in Urdu text (§2), so the
  selection the UI hands the engine is wrong — not the formatting.
- Suspects: (1) mouse drag-selection in RTL/BIDI text maps visual coords to wrong
  logical offsets — hit testing for RTL was just reworked in `96cb347` ("BIDI cluster
  grouping and RTL visual ordering for hit testing"); verify the selection-extension
  path in `canvas.rs::mouse()` stores the full logical range for Urdu paragraphs;
  (2) clicking the font combo collapses/moves `session.sel` before `format.font`
  runs, so `target()` (`format.rs:118`) falls back to word-at-caret or pending props.
- Check via control channel: after drag-selecting Urdu text, dump `session.sel`
  before and after opening the font combo.

## 4. Other issues found (static sweep, this session)

- **17 dead buttons on other tabs** — ids referenced in `ribbon.rs` but absent from the
  389-command registry, so they render disabled and do nothing:
  - Draw tab: `draw.pen`, `draw.pencil`, `draw.highlighter`, `draw.eraser`, `draw.lasso`,
    `draw.select`, `draw.inkToShape`, `draw.inkToMath`, `draw.replay`, `arrange.group`
  - Insert tab: `insert.canvas`, `insert.chart`, `insert.icon`, `insert.smartArt`
  - Review tab: `review.blockAuthors`, `review.translate`
  - View tab: `view.arrangeAll`
  - Fix: implement, or remove/hide the buttons until implemented (dead buttons violate
    the no-mock-theater bar).
- **All ribbon buttons swallow errors** (`let _ = app.run(…)` in `widgets.rs::small`,
  `split`, `menu_button`, `mi`). Add error surfacing (status bar toast) — without it,
  every future "button not working" report is undebuggable.
- Note: `combo()` in `keys.rs` uppercases nothing — `by_shortcut` is case-insensitive,
  fine. `format.growFont1`/`shrinkFont1` (Ctrl+]/[) exist but have no ribbon button;
  consider adding to the A⁺/A⁻ split-button menu for discoverability.

## 5. Suggested verification order for the fix pass
1. Reproduce on the canvas with Urdu text: drag-select → dump `session.sel` → change
   font → check which runs changed. (Settles §3c.)
2. Click any ribbon button → check `canvas.focused` → press Ctrl+A. (Settles §3a.)
3. Click A⁺ with an active selection → check `app.run("format.growFont")` result
   instead of discarding it. (Settles §3b.)
