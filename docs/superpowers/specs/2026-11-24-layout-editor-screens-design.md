# Layout editor: the game screens and the touch controls on one canvas (Trello #134)

## Intent

The owner tried the C06 touch editor (#111 / #131) on the phone and wants, in the same editor,
to move and resize the game screens (the «monitors») as freely as the touch controls. Today C06
moves only the controls; the screens are placed by the layout chosen in «Schermi e immagine»
(and C09, a separate editor, only for DS / 3DS and saved for the console's games on every device).
On a phone the screens and the controls are one composition: the user wants to decide both,
together, where the game will really be.

Also asked: the editor's ground with no game behind (from Settings) becomes a **flat theme
background** (dark in Scuro / Nero, light in Chiaro) and the **dot grid goes away**.

## Goals

- One editor, C06 «Modifica la disposizione»: the game screens (GB / GBC / GBA one, `main`; DS
  and 3DS two, `top` / `bottom`) and the touch controls are all elements you pick, drag and resize
  on the same canvas, phone upright and on its side.
- The game uses exactly the placement saved: the screens where the user put them, measured on
  the whole window (a screen may sit under or over the controls), the touch on the DS / 3DS bottom
  screen mapped from wherever it is.
- Same controller support as today (L1 / R1 pick, d-pad moves, right stick resizes, A saves,
  B back), keyboard and touch (drag, pinch, corner).
- «Torna alla disposizione predefinita» resets controls and screens together.
- Flat ground from Settings, no dot grid (`bg-touch-grid` removed with its token if nothing else
  uses it).

## Non-goals

- No «Solo questo gioco»: the layout is per console only (owner, 2026-11-24). The general
  question about the per-game level is #135.
- No sync: the placement stays on this device (owner: syncing a phone's placement creates
  problems).
- C09 and `screens.custom` stay as they are, for the computer and the TV; nothing of theirs moves.
- No editor on desktop / TV (C06 is the phone's: touch controls exist only there).

## Anti-goals

- A screen placement that shows in the editor but not in game (the #131 item 3 trap: the editor
  and the game must compose from the same data in Rust).
- Losing a saved touch layout: old `touch.tedit` values (controls only) keep working unchanged.
- Two saves that can disagree (controls new, screens old).

## Decided (owner)

- Stored **with the touch controls**, in `touch.tedit`: per device, per console, per
  orientation, never synced (approach A below).
- Per console only.
- Flat theme background, no dots.

## Approaches

- **A. Stored with the touch controls (chosen; refined after the builder check: a key of its own
  beside `touch.tedit`, saved together, see «Stored value»).** `touch.tedit` items gain the screens' rectangles per
  orientation; one write on «Salva»; one validation; the reset and the orientation switch act on
  one thing.
- B. A new per-device setting for the screens alone, beside `touch.tedit`: two writes per save,
  two values to keep in step (reset, orientation, validation). No gain over A.
- C. `screens.custom` with an orientation axis: synced and per game, which the owner rejected (it
  would reach other phones and the PC).

## Design

### Stored value (Rust, `emudome_core::touch`)

- Builder check (2026-11-24): older debug builds on the same phone read the same `device.json`
  and `Layouts::from_items` refuses unknown items, so screen items inside `touch.tedit` would make
  an old build drop the whole layout (and wipe the screens on its next save). Hence, with the
  owner's OK, the screens get **their own key beside it**: `touch.tscreens`, `Kind` of its own,
  console level, `.device()`, `.phone()`, never synced. It is still one save for the user:
  `touch_set_layout` writes `touch.tedit`, `touch.tscreens` and `touch.tport` / `tland` under the
  same settings lock, as it already writes `tedit` + `tport` today. Reset writes both.
- Value: one item per screen, `"<orientation>/<screen id>:<x>,<y>,<w>,<h>"` (fractions 0..1 of
  the whole window, 4 decimals), back to front; none for an orientation = no placement of the
  user's (the screens go where the chosen screen layout puts them, as today).
- `Layout` (the DTO) gains `screens: Vec<CustomScreen>` so the interface edits and sends one
  thing; storage splits it into the two keys.
- Valid: each screen id once, ids the console has (`main` for GB / GBC / GBA, `top` / `bottom`
  for DS / 3DS), all of the console's screens or none, each rectangle inside 0..1 with a minimum
  size (as `screenEditor.ts`'s `MIN_SIZE`). Unit tests: round trip, old values, bad items refused,
  a `touch.tedit` without `touch.tscreens` and the other way round.
- One-screen consoles: `CustomLayout` / `ScreenSession` accept only DS / 3DS screens today
  (`is_valid`, `preset`, `set_layout`, swap); the phone placement needs its own path in
  `ScreenSession` that also covers `main`.

### The game uses it (Rust, `screens.rs` + `app/src-tauri/src/screens.rs`)

- The phone's placement is a custom arrangement **in the whole window**: no inset applied to it
  (the controls may overlap it by the user's choice). The safe area (camera, bars): the user
  places freely inside the whole window; the editor shows the camera's edge so they can avoid it
  (builder's call how).
- Who tells Rust: the touch layer already sends the inset it keeps clear (`session.setInset` →
  `emu.setVideoInset`) while it is shown. It sends, in the same way, the screens of the layout in
  use (or none); Rust's `ScreenSession` then composes `Arrangement::Custom` over the whole window
  instead of the chosen layout while one is set. So the placement holds exactly while the touch
  controls are in use (hidden for a controller, `touch.thide`, the layout chosen in «Schermi e
  immagine» applies again, as today). Touch mapping (`pointer`, `touch_at`) goes through the
  same composition, so the bottom screen maps from wherever it is.
- Swap (`screens.swap`) keeps working: each rectangle shows the other screen's picture, as
  custom layouts do today. Rotation (`screens.rot`) of a phone placement: builder decides with the
  existing custom rule; say it on the card.
- The status the interface reads (`ScreenStatus`) says the phone placement is in use, so the
  quick menu's layout cycle (G09) does not pretend to change it: builder checks what G09 shows
  and makes it honest (disabled with the reason, X05, or the cycle leaves the placement; ask the
  owner only if neither is clean).

### The editor (interface, `components/touch/`)

- Elements = the screens + the controls. Selection, drag, pinch and L1 / R1 go over both
  (screens first in the order, then the controls). A screen is drawn as a frame (the C09
  `ScreenFrame` look, its accent corner to resize): over the game (`live`) the game itself moves
  as it is dragged (the placement sent live, as #131 sends the inset); from Settings the
  placeholders of `TouchPreviewScreens`.
- Starting placement when none is saved: where Rust puts the screens now for the layout being
  edited (`screens_preview` / the open game's placements), turned into fractions of the window.
- Screen tools in the toolbar: size (slider) and «Mantieni le proporzioni» (on by default, the
  screen's own ratio, swapped if swapped); opacity, behaviour and remove are disabled for a screen
  (a screen cannot be removed). Undo / redo cover screens like controls.
- Snapping: the 8 px grid stays as a snap step but is no longer drawn; while dragging, guides
  appear when an element's edge or centre lines up with the window's centre or another element
  (builder may keep only the window-centre guides if the rest is costly; say it on the card).
- «Torna alla disposizione predefinita»: the default controls and no screens of the user's.
- Ground from Settings: `bg-bg` (the theme's background token), the dot grid removed
  (`bg-touch-grid` utility and `--spacing-touch-grid` if unused; `GRID_PX` stays as the snap).
- New texts (IT / EN) listed on the card for the owner: e.g. screen names in the editor
  («Schermo», «Schermo superiore», «Schermo inferiore»), «Mantieni le proporzioni» (reuse C09's).

## Downstream cases (rule: state changes)

- Old `touch.tedit` values (controls only): parse, play and edit as today; screens start from
  the current composition on first edit.
- A layout saved with screens, then `touch.tport` / `tland` set back to `default`: the placement
  is not used (the default layout has none) and comes back with `custom`.
- Controller connected with `touch.thide`: the touch layer hides, the placement is withdrawn,
  the chosen screen layout applies; disconnected: back.
- Phone turned: the other orientation's layout and placement (or none).
- Console with one screen vs two: the stored ids are checked against the console (`fits`).
- Saved while a game is open (quick menu): the placement applies at once; the session's screen
  status updates.

## Calls made after the builder check (Claude Code, 2026-11-24)

- Quick menu open over the game: the menu's inset wins as today (the game makes room for it); the
  placement comes back when it closes. C06 opened live sends its own placement as it sends its
  inset today.
- Proportions: each screen fills its rectangle (as C09); `video.scale` / `video.aspect` do not
  apply to a phone placement (as for custom layouts today). «Mantieni le proporzioni» on by default.
- GB / GBC / GBA in scope with DS / 3DS (the card says every console).
- «Salva» with a game open writes at the console level at once, as C06 does today; not through the
  quick menu's «in prova».
- `ScreenFrame` reused from `components/screen-editor/`, imported from there (CLAUDE.md: a
  component another feature reuses stays with its feature).

## Left to the builder

- Exact command / event shape for sending the placement; how the editor shows the camera edge;
  guides' extent; the rotation rule; G09's honest display. Each choice written on the card.

## Done when

`./scripts/check.ps1` green on the merge commit; card in «To test (UAT)» with what to try on the
A71: move and resize the screens of a GBA and a DS game, upright and on the side, play (touch on
the DS bottom screen), reopen the game, reset.
