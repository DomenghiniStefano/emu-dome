# Layout editor: game screens and touch controls on one canvas (Trello #134) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** C06 «Modifica la disposizione» moves and resizes the game screens (`main`, or `top` / `bottom`) with the touch controls; the game draws them exactly there over the whole window and maps the DS / 3DS touch screen from wherever it is; stored per device, per console, per orientation in `touch.tscreens`, never synced.

**Architecture:** `emudome_core::touch` stores the screens in their own key `touch.tscreens` beside `touch.tedit`; `touch_set_layout` writes both and the orientation's choice in one `store.apply`. The touch layer sends the placement of the layout in use through a new command into `ScreenSession`, which then answers `layout()` with a custom arrangement over the whole window (the layer sends `NO_INSET`); renderer, webview canvas and `pointer` all go through `ScreenSession::layout()`, so they agree. The editor treats screens as one more kind of element: `ScreenFrame` (from `screen-editor/`) draws them, the pure geometry is `components/touch/editorScreens.ts`.

**Tech Stack:** Rust (emudome-core, Tauri 2 shell), ts-rs contract, Vue 3 + TypeScript, Vitest, Tailwind v4 design-system tokens.

**Spec:** `docs/superpowers/specs/2026-11-24-layout-editor-screens-design.md`

## Global Constraints

- No `unwrap()` / `expect()` outside tests; no `_ =>` on our enums (every `match` on `Kind`, `ScreenId`, `Arrangement` lists all variants); no glob imports; no `cfg(target_os)` outside `emudome-platform`.
- Every user-visible string in both `app/src/i18n/en.json` and `it.json`; Rust sends codes and values. A refused placement is `ErrorCode::SettingInvalidValue { key: "touch.tscreens" }`.
- Settings only from `crates/emudome-core/src/settings/catalog.rs`, typed keys in `settings/keys.rs`; every entry in one section and with `settings.<key>.name` / `.desc` texts.
- After any DTO / `Kind` / catalogue change: `$env:EMUDOME_BLESS=1; cargo test -p emudome-core contract` and `cargo test -p emudome contract`; review and commit `app/src/generated/`.
- Interface: design-system tokens only, `@/` imports, controller navigation via `nav.ts` (`useNavScope` with `own`), Vitest for pure TS modules. `ScreenFrame` is imported from `@/components/screen-editor/`.
- Owner's decisions: per device (`.device()`), per console (`C` levels only), per orientation (`portrait/` / `landscape/` items), never synced; no «Solo questo gioco» (#135).
- Editor ground from Settings: flat `bg-bg`; dot grid removed (`bg-touch-grid` utility, `--spacing-touch-grid`); `GRID_PX` stays as the snap step.
- `screens.custom` / C09 unchanged: `CustomLayout::is_valid` still refuses `main`.
- Commits: short conventional subject only (`feat(touch): ...`), no body, no trailer. Each task ends green on its filters plus `cargo clippy --workspace --all-targets -- -D warnings`, `cd app; pnpm typecheck`, `pnpm lint`. Work on a branch in `C:\Projects\emu-dome.wt\layout-editor-screens`.

## Review Focus

1. **Old `touch.tedit` with no `touch.tscreens`** (or a broken `tscreens`) still gives the user's controls; only the screens fall back to none. Task 1 `a_bad_screen_list_drops_only_the_screens`, `an_old_layout_with_no_screens_reads_as_before`.
2. **Turning the phone** switches to the other orientation's placement or none, never carries the upright rectangles to the side. Task 5 `activeLayout keeps each orientation's own screens`.
3. **Controls hidden vs faded**: a controller with `touch.tpad` hides the controls and withdraws the placement; `touch.thide` only fades them and the quick menu only covers them, neither withdraws it. Task 5 `playPlacement withdraws only when the controls are out of use`.
4. **Quick menu inset vs placement, and C06 left without saving**: with the menu open the placement makes room for it *keeping its proportions*; closed, the layer sends `NO_INSET` again; leaving C06 without saving puts the layer's saved placement back. Task 3 `a_phone_placement_makes_room_for_the_quick_menu`, Task 4 `the editor's placement wins while open and the layer's comes back`, Task 5 `playInset keeps no edge while a placement is in use`.
5. **DS bottom-screen touch from a moved and swapped screen**: `pointer_at` hits the bottom screen where the user put it and follows a swap. Task 3 `a_phone_placement_swaps_and_maps_the_touch_screen_from_where_it_is`.

---

### Task 1: Core storage: `touch.tscreens`, `Kind::TouchScreens`, screens in the `Layout` DTO

**Files:**
- Modify: `crates/emudome-core/src/screens.rs` (`ScreenId::from_id` / `id` `pub(crate)` ~50–65; `console_screens`, `placement_fits` after `CustomLayout` ~1078)
- Modify: `crates/emudome-core/src/touch.rs` (`Layout` ~186–215 gains `screens`; `Layouts` ~217–298 screen parsing; `Setup` ~300–337 gains `screen_shapes`; tests from ~339)
- Modify: `crates/emudome-core/src/settings/schema.rs` (`Kind::TouchScreens` after ~221; arm in `accepts_value` ~518; every exhaustive list ~433, ~457, ~534)
- Modify: `crates/emudome-core/src/settings/keys.rs` (`stored()` ~51–62, `TOUCH_SCREENS` after ~147, `int()` list ~273)
- Modify: `crates/emudome-core/src/settings/catalog.rs` (entry after ~656; section `touch/layout` keys ~1125)
- Modify: `crates/emudome-core/src/settings/tests.rs` (exhaustive list ~988; new test after ~330)
- Modify: `crates/emudome-core/src/contract.rs` (kind map ~421, colours list ~452)
- Modify: `app/src/lib/settingsModel.ts` (`isShown` ~64–67, switch ~225–231), `app/src/components/details/gameOverrides.ts` (~34–37), `app/src/lib/settingsModel.test.ts`
- Modify: TS literals `{ controls: [] }` → `{ controls: [], screens: [] }` in `app/src/components/touch/{touchLayout.ts,useTouchEditor.ts,TouchLayer.vue,touchLayout.test.ts,editor.test.ts}`; `TouchSetup` mocks gain `screenShapes`
- Modify: `app/src/i18n/en.json`, `it.json` (`settings.touch.tscreens.name` / `.desc`)
- Regenerate: `app/src/generated/*`

**Interfaces:**
- Consumes: `screens::{CustomScreen, ScreenId, Split, console_frame}`, `core_profile::CoreId`.
- Produces (Rust):
  - `pub(crate) fn ScreenId::from_id(id: &str) -> Option<ScreenId>`, `pub(crate) fn ScreenId::id(self) -> &'static str`
  - `pub fn screens::console_screens(console: ConsoleId) -> &'static [ScreenId]`: Gb / Gbc / Gba `[Main]`, Nds / N3ds `[Top, Bottom]`
  - `pub fn screens::placement_fits(screens: &[CustomScreen], ids: &[ScreenId]) -> bool`: empty true; else each id of `ids` exactly once, rectangles finite, inside 0..1 (with `EDGE_SLACK`), `w >= PLACEMENT_MIN_W` (0.08), `h >= PLACEMENT_MIN_H` (0.04) (mirrors `screenEditor.ts` `MIN_SIZE`, `MIN_SIZE / 2`)
  - `touch::Layout { controls, pub screens: Vec<CustomScreen> }`; TS `TouchLayout = { controls: TouchControl[], screens: CustomScreen[] }`
  - `Layout::is_valid` also requires `placement_fits(&self.screens, ids)` for ids in {none, `[Main]`, `[Top, Bottom]`}; `Layout::fits(console)` also `placement_fits(&self.screens, console_screens(console))`
  - `pub fn Layouts::read(tedit: &[String], tscreens: &[String]) -> Layouts` (each key on its own; a part that does not read is none)
  - `pub fn Layouts::screen_items(&self) -> Vec<String>`: `"<orientation>/<id>:<x>,<y>,<w>,<h>"`, portrait first, back to front, 4 decimals
  - `pub fn Layouts::valid_screen_items(items: &[String]) -> bool` (schema check)
  - `pub fn Layouts::fitting(self, console: ConsoleId) -> Layouts` (drops an orientation's screens that do not fit)
  - `Layouts::to_items` unchanged (controls only)
  - `#[ts(rename = "TouchScreenShape")] pub struct ScreenShape { pub id: ScreenId, pub width: u32, pub height: u32 }`; `Setup` gains `pub screen_shapes: Vec<ScreenShape>` from `CoreId::for_console(c).profile().split.screens(console_frame(c))`
  - `Kind::TouchScreens` (serde `touchScreens`), `keys::TOUCH_SCREENS: ListKey = "touch.tscreens"`
- Produces (TS): `isShown` false for `kind.type === 'touchScreens'` (no row of its own; it goes with the `touch.tedit` row).

- [ ] **Step 1: Failing tests** in `touch.rs` `mod tests` (import `crate::screens::{CustomScreen, ScreenId}`):

```rust
fn scr(id: ScreenId, x: f32, y: f32, w: f32, h: f32) -> CustomScreen { CustomScreen { id, x, y, w, h } }

#[test]
fn screens_are_stored_beside_the_controls_and_read_back() {
    let layouts = Layouts::default().with(Orientation::Portrait, Layout {
        controls: vec![control(ControlId::Menu, 0.5)],
        screens: vec![scr(ScreenId::Top, 0.0, 0.05, 1.0, 0.35), scr(ScreenId::Bottom, 0.0, 0.45, 1.0, 0.35)],
    });
    assert_eq!(layouts.to_items(), ["portrait/menu:0.5,0.5,100,100,tap"]);
    assert_eq!(layouts.screen_items(), ["portrait/top:0,0.05,1,0.35", "portrait/bottom:0,0.45,1,0.35"]);
    assert_eq!(Layouts::read(&layouts.to_items(), &layouts.screen_items()), layouts);
}

#[test]
fn an_old_layout_with_no_screens_reads_as_before() {
    let read = Layouts::read(&items(&["landscape/menu:0.5,0.5,100,100,tap"]), &[]);
    assert_eq!(read.landscape.controls.len(), 1);
    assert!(read.landscape.screens.is_empty() && read.portrait.screens.is_empty());
}

#[test]
fn a_bad_screen_list_drops_only_the_screens() {
    let read = Layouts::read(&items(&["portrait/menu:0.5,0.5,100,100,tap"]), &items(&["portrait/top:0,0,1,0.5"]));
    assert_eq!(read.portrait.controls.len(), 1);
    assert!(read.portrait.screens.is_empty());
    let only_screens = Layouts::read(&[], &items(&["portrait/main:0,0,1,0.6"]));
    assert_eq!(only_screens.portrait.screens, [scr(ScreenId::Main, 0.0, 0.0, 1.0, 0.6)]);
}

#[test]
fn bad_screen_lists_are_refused() {
    for good in [&[][..], &["landscape/main:0.25,0,0.5,1"], &["portrait/top:0,0,1,0.4", "portrait/bottom:0,0.5,1,0.4"]] {
        assert!(Layouts::valid_screen_items(&items(good)), "{good:?}");
    }
    for bad in [
        &["portrait/main:0,0,1,0.5", "portrait/main:0,0.5,1,0.5"][..],
        &["portrait/main:0,0,1,0.5", "portrait/top:0,0.5,1,0.4"],
        &["portrait/top:0,0,1,0.5"],
        &["portrait/main:0.5,0,0.6,0.5"],
        &["portrait/main:0,0,0.05,0.5"],
        &["portrait/main:0,0,0.5,0.03"],
        &["sideways/main:0,0,1,1"],
        &["portrait/left:0,0,1,1"],
        &["portrait/main:0,0,1"],
        &["portrait/main:0,0,1,1,1"],
        &["portrait/main:NaN,0,1,1"],
    ] {
        assert!(!Layouts::valid_screen_items(&items(bad)), "{bad:?}");
    }
}

#[test]
fn screens_fit_only_their_console() {
    let main = Layout { controls: vec![control(ControlId::Menu, 0.5)], screens: vec![scr(ScreenId::Main, 0.0, 0.0, 1.0, 0.5)] };
    assert!(main.fits(ConsoleId::Gba) && main.fits(ConsoleId::Gb));
    assert!(!main.fits(ConsoleId::Nds));
    let two = Layout { screens: vec![scr(ScreenId::Top, 0.0, 0.0, 1.0, 0.4), scr(ScreenId::Bottom, 0.0, 0.5, 1.0, 0.4)], ..main.clone() };
    assert!(two.fits(ConsoleId::Nds) && two.fits(ConsoleId::N3ds) && !two.fits(ConsoleId::Gbc));
    let stored = Layouts::default().with(Orientation::Portrait, two).fitting(ConsoleId::Gba);
    assert!(stored.portrait.screens.is_empty());
    assert_eq!(stored.portrait.controls.len(), 1);
}

#[test]
fn the_setup_names_each_screen_with_its_size() {
    let shape = |id, width, height| ScreenShape { id, width, height };
    assert_eq!(Setup::new(ConsoleId::Gba, Layouts::default()).screen_shapes, [shape(ScreenId::Main, 240, 160)]);
    assert_eq!(Setup::new(ConsoleId::Nds, Layouts::default()).screen_shapes, [shape(ScreenId::Top, 256, 192), shape(ScreenId::Bottom, 256, 192)]);
    assert_eq!(Setup::new(ConsoleId::N3ds, Layouts::default()).screen_shapes, [shape(ScreenId::Top, 400, 240), shape(ScreenId::Bottom, 320, 240)]);
}
```

  In `settings/tests.rs`, `touch_screens_stay_on_this_device_at_the_console_level` (modelled on `touch_layouts_stay_on_this_device_per_console_and_survive_a_restart`): `entry("touch.tscreens").unwrap().per_device`; `matches!(kind, Kind::TouchScreens)`; levels console only; `set` at Global and Game is `Err`; `["portrait/main:0,0,1,0.5"]` at console `Ok`; `["portrait/main:0,0,2,0.5"]` `Err`; `!synced_text(&dir).contains("touch.tscreens")`.
  In `settingsModel.test.ts`: `it('hides the touch screens entry, edited with the touch.tedit row')`: `isShown({ phase: 1, kind: { type: 'touchScreens' } } as never, { currentPhase: 1 } as never)` is `false`.

- [ ] **Step 2: Run, expect FAIL** (no `screens` field, no `Layouts::read`, no `Kind::TouchScreens`): `cargo test -p emudome-core touch::`, `cargo test -p emudome-core settings::tests::touch_screens`
- [ ] **Step 3: Implement** the signatures above. The item parser mirrors `Layouts::from_items` (`split_once(':')`, `split_once('/')`, four `f32`). Catalogue: `e("touch.tscreens", Kind::TouchScreens, DefaultValue::List(&[]), C).device().phone()` right after `touch.tedit`, with a comment saying why it is a key of its own (an old build drops a `tedit` with unknown items); section `touch/layout` keys `["touch.tedit", "touch.tscreens", "touch.tport", "touch.tland", "touch.tskin"]`. `touchScreens` arm in `contract.rs` and every exhaustive `Kind` match. TS: `touchScreens` case in the `notYetWired` and `gameOverrides` switches; `isShown` = `entry.phase <= schema.currentPhase && entry.kind.type !== 'touchScreens'`. Texts: en «Screen positions» / «Where the game screens go, set in the touch layout editor.»; it «Posizione degli schermi» / «Dove stanno gli schermi del gioco, scelto nell’editor della disposizione.». Regenerate the contract.
- [ ] **Step 4: Run, expect PASS:** `cargo test -p emudome-core touch::`, `cargo test -p emudome-core settings::`, `cargo test -p emudome-core contract`, `cargo test -p emudome contract`; `cd app; pnpm test src/lib/settingsModel.test.ts src/i18n/parity.test.ts src/components/touch/touchLayout.test.ts`, `pnpm typecheck`, `pnpm lint`, clippy.
- [ ] **Step 5: Commit** `feat(touch): store the screens' placement beside the touch layout`

---

### Task 2: `touch_set_layout` writes `tedit` + `tscreens` + `tport` / `tland` in one write; `touch_layouts` returns the screens

**Files:**
- Modify: `crates/emudome-core/src/settings/mod.rs` (new `set_many` after `set`, ~226), `settings/tests.rs`
- Modify: `app/src-tauri/src/touch.rs` (~14–71)

**Interfaces:**
- `pub fn Settings::set_many(&mut self, scope: &Scope, values: &[(&str, Value)]) -> Result<Level, ErrorCode>`: validates every key and value first (known, level accepted, `accepts_value_on`), then one `store.apply(&file, &[Change…])` (or all into the session). All or nothing.
- `touch_layouts(app, console) -> Result<Setup, ErrorCode>`: resolves `TOUCH_LAYOUTS` and `TOUCH_SCREENS` at `Scope::Console`, `Layouts::read(&tedit, &tscreens).fitting(console)`, `Setup::new`.
- `touch_set_layout(app, console, orientation, layout: Layout)` keeps its signature: refuses with `SettingInvalidValue { key: "touch.tedit" }` when `controls` is empty or `!layout.fits(console)`; under one lock reads both keys, `stored.with(orientation, layout)`, `set_many(&scope, &[(tedit, to_items), (tscreens, screen_items), (choice, "custom")])`; `emit_changed(&app, &[tedit, tscreens, choice], level)`. `screens: []` clears that orientation's placement (the editor's reset then «Salva»).

- [ ] **Step 1: Failing test** in `settings/tests.rs`:

```rust
#[test]
fn set_many_writes_every_value_or_none() {
    let dir = TestDir::new("set-many");
    let mut s = settings(&dir);
    let tedit = Value::List(vec!["portrait/menu:0.5,0.5,100,100,tap".to_owned()]);
    let screens = Value::List(vec!["portrait/main:0,0,1,0.5".to_owned()]);
    let bad = Value::List(vec!["portrait/main:0,0,2,0.5".to_owned()]);
    assert!(s.set_many(&console_scope(), &[("touch.tedit", tedit.clone()), ("touch.tscreens", bad)]).is_err());
    assert_eq!(value_of(&mut s, "touch.tedit", &console_scope()).0, Some(Value::List(vec![])));
    s.set_many(&console_scope(), &[("touch.tedit", tedit.clone()), ("touch.tscreens", screens.clone()), ("touch.tport", Value::Text("custom".into()))]).unwrap();
    assert_eq!(value_of(&mut s, "touch.tedit", &console_scope()).0, Some(tedit));
    assert_eq!(value_of(&mut s, "touch.tscreens", &console_scope()).0, Some(screens));
    assert!(s.set_many(&Scope::Global, &[("touch.tport", Value::Text("custom".into())), ("touch.tedit", Value::List(vec![]))]).is_err());
}
```

  (Check what `value_of` gives for an untouched `touch.tedit`, default or `None`, and assert that.)
- [ ] **Step 2: Run, expect FAIL:** `cargo test -p emudome-core settings::tests::set_many` (no method `set_many`).
- [ ] **Step 3: Implement** `set_many` (one `Change` per key, all for `file_of(scope)`); rewrite both shell commands; `touch_set_layout` no longer calls `set` twice.
- [ ] **Step 4: Run, expect PASS:** `cargo test -p emudome-core settings::`, `cargo test -p emudome contract`, clippy.
- [ ] **Step 5: Commit** `feat(touch): save the controls and the screens in one write`

---

### Task 3: `ScreenSession` phone placement (one and two screens), status, touch mapping, start placement

**Files:**
- Modify: `crates/emudome-core/src/screens.rs` (`ScreenSession` ~1243–1356 new field; `ScreenStatus` ~1211–1241; `compose`'s custom branch ~626; new `phone_start` after `preview` ~1103; tests from ~1358)
- Modify (TS mocks of the status): `app/src/components/quick-menu/quickMenu.test.ts` (~117, ~195), `app/src/design-system/kit/features/game/mock.ts` (~54)
- Regenerate: `app/src/generated/ScreenStatus.ts`

**Interfaces:**
- `ScreenSession` private field `phone: Vec<CustomScreen>` (empty: none).
- `pub fn ScreenSession::set_phone_placement(&mut self, screens: Vec<CustomScreen>) -> Option<bool>`: `None` when `!placement_fits(&screens, ids)` with `ids = if dual_screen { [Top, Bottom] } else { [Main] }`, else `Some(changed)`.
- `ScreenSession::layout()` with a placement: `Layout { arrangement: Arrangement::Phone(rects), swap: self.swapped, ..self.settings.layout(self.current, self.swapped) }` — a new `Arrangement::Phone(Vec<(ScreenId, Rect)>)` variant, placed like `Custom` but **uniformly**: the whole placement (fractions of the window) is scaled by `min(usable.w / width, usable.h / height)` and centred in the usable part, so with no inset it is exactly the user's rectangles and with the quick menu's inset it makes room keeping its proportions. Rotation: the existing custom rule. `aspect` / `scale` do not apply. Every `match` on `Arrangement` gets the new arm.
- `next_layout()` returns `false`, `set_layout()` `None` while a placement is held; swap and the lid as before.
- `ScreenStatus` gains `pub phone_placement: bool` (TS `phonePlacement`).
- `pub fn phone_start(settings: &ScreenSettings, console: ConsoleId, area: Area) -> Vec<CustomScreen>`: every screen of `console_screens(console)` as fractions of `area.width` × `area.height` (clamped like `CustomLayout::preset`'s `fraction`), composed by `preview`; when that leaves a screen out (`top_only` first in the cycle), `Arrangement::Auto` instead.

- [ ] **Step 1: Failing tests** (`screens.rs` tests; existing helpers `area`, `rect`, `DS`, `FRAME`, `settings`, `list`):

```rust
fn cs(id: ScreenId, x: f32, y: f32, w: f32, h: f32) -> CustomScreen { CustomScreen { id, x, y, w, h } }

#[test]
fn a_phone_placement_is_composed_over_the_whole_window() {
    let mut s = ScreenSession::new(ScreenSettings::default(), false, false);
    assert_eq!(s.set_phone_placement(vec![cs(ScreenId::Main, 0.0, 0.1, 1.0, 0.4)]), Some(true));
    let p = compose(&s.layout(), &Split::Whole.screens(240, 160), area(360.0, 780.0));
    assert_eq!(p[0].dst, rect(0.0, 78.0, 360.0, 312.0));
    assert!(s.status().phone_placement);
    assert_eq!(s.set_phone_placement(vec![]), Some(true));
    assert!(!s.status().phone_placement);
}

#[test]
fn a_phone_placement_must_be_the_games_screens() {
    let mut one = ScreenSession::new(ScreenSettings::default(), false, false);
    assert_eq!(one.set_phone_placement(vec![cs(ScreenId::Top, 0.0, 0.0, 1.0, 0.5), cs(ScreenId::Bottom, 0.0, 0.5, 1.0, 0.5)]), None);
    let mut two = ScreenSession::new(ScreenSettings::default(), true, false);
    assert_eq!(two.set_phone_placement(vec![cs(ScreenId::Main, 0.0, 0.0, 1.0, 0.5)]), None);
    assert_eq!(two.set_phone_placement(vec![cs(ScreenId::Top, 0.0, 0.0, 1.0, 0.5)]), None);
    assert!(!two.status().phone_placement);
}

#[test]
fn a_phone_placement_swaps_and_maps_the_touch_screen_from_where_it_is() {
    let mut s = ScreenSession::new(ScreenSettings::default(), true, false);
    s.set_phone_placement(vec![cs(ScreenId::Top, 0.0, 0.5, 1.0, 0.25), cs(ScreenId::Bottom, 0.0, 0.0, 1.0, 0.25)]).unwrap();
    let bottom = PxRect { x: 0, y: 192, w: 256, h: 192 };
    let want = Some(pointer_value(bottom, FRAME, 128, 96));
    assert_eq!(pointer_at(&s.layout(), DS, FRAME, area(360.0, 780.0), 180.0, 97.5), want);
    assert!(s.toggle_swap());
    assert_eq!(pointer_at(&s.layout(), DS, FRAME, area(360.0, 780.0), 180.0, 97.5), None);
    assert_eq!(pointer_at(&s.layout(), DS, FRAME, area(360.0, 780.0), 180.0, 487.5), want);
}

#[test]
fn a_phone_placement_makes_room_for_the_quick_menu() {
    let mut s = ScreenSession::new(ScreenSettings::default(), false, false);
    s.set_phone_placement(vec![cs(ScreenId::Main, 0.0, 0.0, 1.0, 0.5)]).unwrap();
    // 800 × 400 window, the menu on the right half: the placement (800 × 200 at full size)
    // is scaled by 0.5 into the left 400 × 400 and centred vertically, proportions kept.
    let menu = Area { width: 800.0, height: 400.0, inset: Inset { right: 0.5, ..Inset::NONE } };
    assert_eq!(compose(&s.layout(), &Split::Whole.screens(240, 160), menu)[0].dst, rect(0.0, 100.0, 400.0, 100.0));
}

#[test]
fn the_layout_cycle_waits_while_a_phone_placement_is_in_use() {
    let mut s = ScreenSession::new(ScreenSettings::default(), true, false);
    s.set_phone_placement(vec![cs(ScreenId::Top, 0.0, 0.0, 1.0, 0.4), cs(ScreenId::Bottom, 0.0, 0.5, 1.0, 0.4)]).unwrap();
    let shown = s.status().layout;
    assert!(!s.next_layout());
    assert_eq!(s.set_layout(LayoutId::Side), None);
    assert_eq!(s.status().layout, shown);
    s.set_phone_placement(vec![]).unwrap();
    assert!(s.next_layout());
}

#[test]
fn the_phone_start_has_every_screen_of_the_console() {
    let ids = |v: Vec<CustomScreen>| v.iter().map(|s| s.id).collect::<Vec<_>>();
    let start = phone_start(&ScreenSettings::default(), ConsoleId::Nds, area(360.0, 780.0));
    assert_eq!(ids(start.clone()), [ScreenId::Top, ScreenId::Bottom]);
    assert!(start.iter().all(|s| s.x >= 0.0 && s.x + s.w <= 1.0 && s.y + s.h <= 1.0));
    assert!(start[0].y < start[1].y);
    let top_only = settings(&[("screens.cycle", list(&["top_only"]))]);
    assert_eq!(ids(phone_start(&top_only, ConsoleId::N3ds, area(360.0, 780.0))), [ScreenId::Top, ScreenId::Bottom]);
    assert_eq!(ids(phone_start(&ScreenSettings::default(), ConsoleId::Gba, area(360.0, 780.0))), [ScreenId::Main]);
}
```

- [ ] **Step 2: Run, expect FAIL:** `cargo test -p emudome-core screens::`
- [ ] **Step 3: Implement** the above (the `Phone` placement's uniform fit is the one algorithm here: bounding box = the window `(0,0,1,1)` in window pixels, scale `k = min(usable.w / area.width, usable.h / area.height)`, offset centres it in `usable`; each rect `x' = usable.x + (usable.w - k·W)/2 + r.x·k·W`, same for y). `phonePlacement: false` in the TS status mocks. Regenerate the contract.
- [ ] **Step 4: Run, expect PASS:** `cargo test -p emudome-core screens::`, `cargo test -p emudome-core contract`, `cd app; pnpm typecheck`, clippy.
- [ ] **Step 5: Commit** `feat(screens): the phone's own placement of the screens in the session`

---

### Task 4: Bridge: send and withdraw the placement; the editor's placement over the layer's

**Files:**
- Modify: `app/src-tauri/src/screens.rs` (add `screens_set_phone`; replace `screens_preview` ~344–357 with `screens_phone_start`), `app/src-tauri/src/lib.rs` (command list ~555–564)
- Modify: `app/src/emu.ts` (`screens` ~345–379)
- Create: `app/src/components/game/placement.ts`, `placement.test.ts`
- Modify: `app/src/components/game/useGameSession.ts` (by `setInset` ~165–176; `begin` ~55–79)

**Interfaces:**
- Rust: `#[tauri::command] pub fn screens_set_phone(app: AppHandle, screens: Vec<CustomScreen>) -> Result<ScreenStatus, ErrorCode>` (no game: `Ok(ScreenStatus::default())`; `None` from the session → `Err(SettingInvalidValue { key: keys::TOUCH_SCREENS.as_str() })`; else `change(...)`, which publishes to the renderer and `emu://screens`). `#[tauri::command] pub fn screens_phone_start(app, scope: Scope, console: ConsoleId, area: PointerArea) -> Vec<CustomScreen>` (`screens::phone_start` at the settings `scope` resolves). `screens_preview` removed (its only caller is replaced in Task 6).
- TS `emu.screens`: `setPhone: (screens: CustomScreen[]) => invoke<ScreenStatus>('screens_set_phone', { screens })`; `phoneStart: (scope: SettingsScope, console: ConsoleId, area: PointerArea) => invoke<CustomScreen[]>('screens_phone_start', { scope, console, area })`; `preview` removed.
- `placement.ts`: `export function effectivePlacement(layer: readonly CustomScreen[], editor: readonly CustomScreen[] | null): CustomScreen[]` (the editor's while open, `null` when closed); `export function samePlacement(a: readonly CustomScreen[], b: readonly CustomScreen[]): boolean`.
- `useGameSession`: `setPlacement(from: 'layer' | 'editor', screens: CustomScreen[] | null)` sends `effectivePlacement` when it changes (`report(emu.screens.setPhone(...))`); `launches: Ref<number>` goes up in `begin()`, which also forgets what was sent (the new Rust session holds none); the placement is sent before the inset within one change.

- [ ] **Step 1: Failing test** `placement.test.ts`:

```ts
const top = { id: 'top', x: 0, y: 0, w: 1, h: 0.4 } as const
const bottom = { id: 'bottom', x: 0, y: 0.5, w: 1, h: 0.4 } as const
it("the editor's placement wins while open and the layer's comes back", () => {
  expect(effectivePlacement([top, bottom], [bottom])).toEqual([bottom])
  expect(effectivePlacement([top, bottom], [])).toEqual([])
  expect(effectivePlacement([top, bottom], null)).toEqual([top, bottom])
})
it('tells the same placement apart from a moved one', () => {
  expect(samePlacement([top, bottom], [{ ...top }, { ...bottom }])).toBe(true)
  expect(samePlacement([top, bottom], [{ ...top, x: 0.1 }, bottom])).toBe(false)
  expect(samePlacement([top], [])).toBe(false)
})
```

- [ ] **Step 2: Run, expect FAIL:** `cd app; pnpm test src/components/game/placement.test.ts`
- [ ] **Step 3: Implement** the commands, registrations, bindings, `placement.ts`, `useGameSession` additions; point `useScreenPreview.ts` at `phoneStart` (or do Task 6's `useStartScreens` here) so everything compiles.
- [ ] **Step 4: Run, expect PASS:** `pnpm test src/components/game/placement.test.ts`, `pnpm typecheck`, `pnpm lint`, `cargo test -p emudome contract`, clippy.
- [ ] **Step 5: Commit** `feat(screens): send the phone placement to the open game`

---

### Task 5: `TouchLayer` sends the placement of the layout in use

**Files:**
- Modify: `app/src/components/touch/touchLayout.ts` (`activeLayout` ~305–320; helpers after `reservedInset` ~543), `touchLayout.test.ts`
- Modify: `app/src/components/touch/TouchLayer.vue` (emits ~30–37, `shown` ~72–82, inset watch ~97–114)
- Modify: `app/src/App.vue` (~138–145)

**Interfaces:**
- `activeLayout(...)` returns `{ controls: saved, screens: setup[orientation].screens }` when the choice is `custom` and controls are saved, else `{ controls: default, screens: [] }`.
- `export function playPlacement(layout: TouchLayout, controlsInUse: boolean): CustomScreen[]`
- `export function playInset(layout: TouchLayout, shown: boolean, reserved: () => InsetPx): InsetPx`: `NO_INSET` when not shown or `layout.screens.length > 0`, else `reserved()`.
- `TouchLayer` emits `placement: [CustomScreen[]]` with `controlsInUse = touchShown({ ...inputs, menuOpen: false })` (the menu and `thide`'s fade do not withdraw it); `[]` on unmount.
- `App.vue`: `<TouchLayer :key="session.launches.value" ... @placement="s => session.setPlacement('layer', s)">`

- [ ] **Step 1: Failing tests** in `touchLayout.test.ts`:

```ts
const main = { id: 'main', x: 0, y: 0.1, w: 1, h: 0.4 } as const
const side = { id: 'main', x: 0.25, y: 0, w: 0.5, h: 0.7 } as const
it("activeLayout keeps each orientation's own screens", () => {
  const setup = { ...gbaSetup, portrait: { controls: [menu], screens: [main] }, landscape: { controls: [menu], screens: [side] } }
  expect(activeLayout('gba', setup, 'portrait', 'custom', upright).screens).toEqual([main])
  expect(activeLayout('gba', setup, 'landscape', 'custom', sideways).screens).toEqual([side])
  expect(activeLayout('gba', setup, 'portrait', 'default', upright).screens).toEqual([])
  expect(activeLayout('gba', { ...setup, portrait: { controls: [], screens: [main] } }, 'portrait', 'custom', upright).screens).toEqual([])
})
it('playPlacement withdraws only when the controls are out of use', () => {
  const layout = { controls: [menu], screens: [main] }
  expect(playPlacement(layout, true)).toEqual([main])
  expect(playPlacement(layout, false)).toEqual([])
  expect(touchShown({ touch: true, inGame: true, menuOpen: false, editing: false, hideWithPad: true, controllers: 1 })).toBe(false)
  expect(touchShown({ touch: true, inGame: true, menuOpen: false, editing: false, hideWithPad: false, controllers: 1 })).toBe(true)
})
it('playInset keeps no edge while a placement is in use', () => {
  const band = { ...NO_INSET, bottom: 220, align: 'bottom' as const }
  expect(playInset({ controls: [menu], screens: [main] }, true, () => band)).toEqual(NO_INSET)
  expect(playInset({ controls: [menu], screens: [] }, true, () => band)).toEqual(band)
  expect(playInset({ controls: [menu], screens: [] }, false, () => band)).toEqual(NO_INSET)
})
```

  (`gbaSetup`, `menu`, `upright`, `sideways`: the file's fixtures or new ones; `touchShown` from `./touchVisibility`.)
- [ ] **Step 2: Run, expect FAIL:** `cd app; pnpm test src/components/touch/touchLayout.test.ts`
- [ ] **Step 3: Implement** the helpers; in `TouchLayer` the inset watch keeps its `menuOpen` early return and emits `playInset(layout, shown, () => reservedInset(...))`; a second watch on `[layout, controlsInUse]` emits the placement (no `menuOpen` gate); wire `App.vue`.
- [ ] **Step 4: Run, expect PASS:** `pnpm test src/components/touch/touchLayout.test.ts`, `pnpm typecheck`, `pnpm lint`
- [ ] **Step 5: Commit** `feat(touch): play the screens where the touch layout puts them`

---

### Task 6: Editor model: screens as elements in `useTouchEditor`

**Files:**
- Create: `app/src/components/touch/editorScreens.ts`, `editorScreens.test.ts`
- Replace: `app/src/components/touch/useScreenPreview.ts` → `useStartScreens.ts` (same `latest()` pattern, `emu.screens.phoneStart`, returns `Ref<CustomScreen[]>`)
- Modify: `app/src/components/touch/useTouchEditor.ts` (selection, gestures, `own`, `onKey`, `resetLayout`, `save`, returned API)

**Interfaces (`editorScreens.ts`, pure; reuses `contain`, `resizeTo`, `round4`, `MIN_SIZE`, `Proportion` from `@/components/screen-editor/screenEditor`, `GRID_PX`, `Area` from `./touchLayout`):**

```ts
export type ElementId = ScreenId | TouchControlId
export const isScreen: (id: ElementId) => id is ScreenId
export function screensShown(layout: TouchLayout, start: readonly CustomScreen[]): CustomScreen[]
export function elementIds(layout: TouchLayout, start: readonly CustomScreen[]): ElementId[]        // screens first, then controls
export function nextElement(ids: readonly ElementId[], current: ElementId | null, dir: -1 | 1): ElementId | null
export function withScreen(layout: TouchLayout, start: readonly CustomScreen[], s: CustomScreen): TouchLayout // all screens or none
export function ratioOf(shapes: readonly TouchScreenShape[], id: ScreenId, swapped: boolean): number
export function moveScreen(s: CustomScreen, leftPx: number, topPx: number, area: Area, snap: boolean): CustomScreen
export function nudgeScreen(s: CustomScreen, dx: -1 | 0 | 1, dy: -1 | 0 | 1, area: Area): CustomScreen
export function resizeScreen(s: CustomScreen, widthPx: number, heightPx: number, keep: boolean, ratio: number, area: Area): CustomScreen
export const screenSizePct: (s: CustomScreen) => number
export function centreGuides(s: CustomScreen, area: Area): { across: boolean; down: boolean }
```

`useTouchEditor`: `selected: Ref<ElementId | null>`; `startScreens = useStartScreens(console, scope, area)`; `shownScreens = computed(screensShown(draft, start))`; `keep = ref(true)`; `swapped` (from `emu.screens.status()` when `live`, else `false`); new `pressScreen(id: ScreenId, e: PointerEvent, corner: boolean)`, `setScreenSize(pct: number, final: boolean)`, `setKeep(on: boolean)`; `pickNext` via `nextElement(elementIds(...))`; `own()` on a screen: d-pad `nudgeScreen`, right stick `resizeScreen` by ±`GRID_PX`, X does nothing; `resetLayout` → `{ ...defaultLayout(...), screens: [] }`; `placement = computed(() => draft.value.screens)` (live while dragging); `inset` = `NO_INSET` while `draft.screens.length > 0`; `save` sends `draft`.

- [ ] **Step 1: Failing tests** `editorScreens.test.ts`:

```ts
const area = { width: 360, height: 780 }
const top = { id: 'top', x: 0, y: 0, w: 1, h: 0.35 } as const
const bottom = { id: 'bottom', x: 0, y: 0.45, w: 1, h: 0.35 } as const
const menu = { id: 'menu', x: 0.5, y: 0.9, size: 100, opacity: 100, behaviour: 'tap' } as const
const a = { ...menu, id: 'a' } as const

it('picks the screens first, then the controls, wrapping', () => {
  const ids = elementIds({ controls: [menu, a], screens: [] }, [top, bottom])
  expect(ids).toEqual(['top', 'bottom', 'menu', 'a'])
  expect(nextElement(ids, 'a', 1)).toBe('top')
  expect(nextElement(ids, 'top', -1)).toBe('a')
  expect(nextElement([], null, 1)).toBe(null)
})
it('moves one screen and keeps them all, or none', () => {
  const moved = { ...top, y: 0.1 }
  expect(withScreen({ controls: [menu], screens: [] }, [top, bottom], moved).screens).toEqual([moved, bottom])
  expect(screensShown({ controls: [menu], screens: [] }, [top, bottom])).toEqual([top, bottom])
  const h = commit(historyOf({ controls: [menu], screens: [] }), withScreen({ controls: [menu], screens: [] }, [top, bottom], moved))
  expect(undo(h).present.screens).toEqual([])
})
it('snaps a dragged screen to the 8 px grid and keeps it inside', () => {
  const main = { id: 'main', x: 0, y: 0, w: 0.5, h: 0.25 } as const
  expect(moveScreen(main, 101, 203, area, true)).toEqual({ ...main, x: 0.2889, y: 0.2564 })
  expect(moveScreen(main, 400, 0, area, true).x).toBe(0.5)
})
it('resizes keeping the screen’s proportions, never under the minimum', () => {
  const main = { id: 'main', x: 0, y: 0, w: 0.5, h: 0.25 } as const
  expect(resizeScreen(main, 360, 0, true, 1.5, area)).toEqual({ ...main, w: 1, h: 0.3077 })
  expect(resizeScreen(main, 180, 390, false, 1.5, area)).toEqual({ ...main, w: 0.5, h: 0.5 })
  expect(resizeScreen(main, 10, 10, false, 1.5, area).w).toBe(0.08)
})
it('takes the ratio of the picture the frame shows', () => {
  const shapes = [{ id: 'top', width: 400, height: 240 }, { id: 'bottom', width: 320, height: 240 }] as const
  expect(ratioOf(shapes, 'top', false)).toBeCloseTo(400 / 240)
  expect(ratioOf(shapes, 'top', true)).toBeCloseTo(320 / 240)
})
it('shows the window-centre guides when a screen is centred', () => {
  expect(centreGuides({ id: 'main', x: 0.25, y: 0, w: 0.5, h: 0.2 }, area)).toEqual({ across: true, down: false })
})
```

- [ ] **Step 2: Run, expect FAIL:** `cd app; pnpm test src/components/touch/editorScreens.test.ts`
- [ ] **Step 3: Implement** `editorScreens.ts`, `useStartScreens.ts`, the `useTouchEditor` changes (the size slider: `resizeScreen(s, pct / 100 * width, s.h * height, keep, ratio, area)`, live while held, one undo step on commit).
- [ ] **Step 4: Run, expect PASS:** `pnpm test src/components/touch/editorScreens.test.ts src/components/touch/editor.test.ts`, `pnpm typecheck`, `pnpm lint`
- [ ] **Step 5: Commit** `feat(touch): the editor moves and resizes the game screens`

---

### Task 7: Editor view: frames on the canvas, screen tools, live preview, flat ground, no dot grid

**Files:**
- Modify: `app/src/components/touch/TouchEditor.vue` (ground ~117 `live ? '' : 'bg-bg'`; remove the grid div ~122 and `TouchPreviewScreens`; `ScreenFrame` per `shownScreens`; guides; camera edge; `placement` emit)
- Modify: `app/src/components/touch/editorToolbar.ts` (+ pure `toolDisabled`), `editor.test.ts`, `TouchToolbar.vue` (`disabled()` moves out, prop `screenPicked`), `TouchToolPanel.vue` (size panel for a screen: Slider + Switch «Mantieni le proporzioni»)
- Modify: `app/src/components/quick-menu/QuickMenu.vue` (~507 `@placement="s => emit('placement', s)"` + emit declaration); `App.vue` (`@placement="s => session.setPlacement('editor', s)"` on QuickMenu)
- Delete: `app/src/components/touch/TouchPreviewScreens.vue`
- Modify: `app/src/design-system/styles/utilities.css` (remove `bg-touch-grid`, ~87–92), `app/src/design-system/tokens/spacing.css` (remove `--spacing-touch-grid`, ~256–257)
- Modify: `app/src/i18n/en.json`, `it.json`

**Interfaces:**
- `export interface ToolState { canUndo: boolean; canRedo: boolean; picked: boolean; screenPicked: boolean; canRemove: boolean; canAdd: boolean }`; `export function toolDisabled(tool: Tool, s: ToolState): boolean` (opacity, behaviour, remove disabled while `screenPicked`; size enabled).
- `TouchEditor` emits `placement: [CustomScreen[] | null]` (`draft.screens` while `live` and loaded; `null` on unmount).
- `ScreenFrame`: `screen`, `label` = `t('touch.editor.screen.<id>')` of the picture it holds (`other(id)` when swapped), `selected`, `placeholder = !live`, `@press="(e, corner) => editor.pressScreen(id, e, corner)"`. The section's `pointerDown` ignores targets inside `[data-editor-screen]`.
- Guides: two 1 px `bg-accent` lines from `centreGuides` while `gesturing` (window centre only).
- Camera edge: a dashed `border-line-strong` band the size of the `safe-*` edges, `pointer-events-none`, editor only.
- Texts: `touch.editor.screen.main` «Schermo» / "Screen"; `.top` «Schermo superiore» / "Top screen"; `.bottom` «Schermo inferiore» / "Bottom screen"; `touch.editor.screenOnly` «Uno schermo si può solo spostare e ridimensionare.» / "A screen can only be moved and resized." (the reason on the disabled tools); «Mantieni le proporzioni» reuses `screenEditor.keep`.

- [ ] **Step 1: Failing test** in `editor.test.ts`:

```ts
describe("the editor's tools on a screen", () => {
  const base = { canUndo: true, canRedo: false, picked: true, screenPicked: true, canRemove: false, canAdd: true }
  it('keeps the size and disables what a screen cannot have', () => {
    expect(toolDisabled('size', base)).toBe(false)
    expect(toolDisabled('opacity', base)).toBe(true)
    expect(toolDisabled('behaviour', base)).toBe(true)
    expect(toolDisabled('remove', base)).toBe(true)
    expect(toolDisabled('opacity', { ...base, screenPicked: false })).toBe(false)
    expect(toolDisabled('redo', base)).toBe(true)
  })
})
```

- [ ] **Step 2: Run, expect FAIL:** `cd app; pnpm test src/components/touch/editor.test.ts`
- [ ] **Step 3: Implement** the view; delete `TouchPreviewScreens.vue`, `bg-touch-grid`, `--spacing-touch-grid`; grep `app/src` for `touch-grid` and `TouchPreviewScreens`: nothing left.
- [ ] **Step 4: Run, expect PASS:** `pnpm test src/components/touch src/i18n/parity.test.ts`, `pnpm typecheck`, `pnpm lint`, `node scripts/scan-conventions.mjs`
- [ ] **Step 5: Commit** `feat(touch): draw the screens in the layout editor on a flat ground`

---

### Task 8: G09 is honest while a phone placement is in use

**Files:**
- Modify: `app/src/components/quick-menu/screenCards.ts` (`layoutCard` ~117–133), `quickMenu.test.ts`
- Modify: `app/src/i18n/en.json`, `it.json`

**Interfaces:** `layoutCard(c)` with `c.status.phonePlacement` returns `value: { key: 'qm.layoutPhone.value' }`, `sub: { key: 'qm.layoutPhone.why' }`, `trial: false`, `prev: null`, `next: null`. Rust already refuses `next_layout` / `set_layout` (Task 3).

- [ ] **Step 1: Failing test:**

```ts
it('shows the layout card disabled with its reason while the touch layout places the screens', () => {
  const card = layoutCard({ status: { ...status, phonePlacement: true }, row: undefined, consoleName: 'DS', set: vi.fn() })
  expect(card.value).toEqual({ key: 'qm.layoutPhone.value' })
  expect(card.sub).toEqual({ key: 'qm.layoutPhone.why' })
  expect(card.prev).toBe(null)
  expect(card.next).toBe(null)
})
```

- [ ] **Step 2: Run, expect FAIL:** `cd app; pnpm test src/components/quick-menu/quickMenu.test.ts`
- [ ] **Step 3: Implement** the early return. Texts: it «Dalla disposizione touch» / «Gli schermi sono dove li hai messi in Modifica la disposizione.»; en "From the touch layout" / "The screens are where you put them in Edit layout."
- [ ] **Step 4: Run, expect PASS:** `pnpm test src/components/quick-menu/quickMenu.test.ts src/i18n/parity.test.ts`, `pnpm typecheck`, `pnpm lint`
- [ ] **Step 5: Commit** `feat(quick-menu): say when the touch layout places the screens`

---

### Task 9: Docs and final check

**Files:**
- Modify: `CLAUDE.md` (`screens.rs` description: `ScreenSession`'s phone placement, `Arrangement::Phone`, `phone_start`, `console_screens`, `placement_fits`; `touch.rs`: `touch.tscreens` beside `touch.tedit`, `Layouts::read`; shell `touch.rs` / `screens.rs`: `set_many`, `screens_set_phone`, `screens_phone_start` replaces `screens_preview`; the `touch` folder: `editorScreens.ts`, `useStartScreens`, `ScreenFrame` reused, flat `bg-bg`, no grid; Screens section: a phone placement is placed over the whole window while the touch controls are in use, scaled uniformly to make room for the quick menu)
- Modify: `docs/known-gaps.md` (C06 entries: screens are editable frames on the theme's ground; remaining gaps: guides only to the window centre, the layout per game (#135))

- [ ] **Step 1:** Edit the docs so they say what the code does (rule 9).
- [ ] **Step 2:** `./scripts/check.ps1 --fast` → green.
- [ ] **Step 3: Commit** `docs(touch): the layout editor places the game screens`
- [ ] **Step 4:** On the card: the builder's calls (command shape, window-centre guides, camera edge band, rotation = custom rule, G09 disabled with its reason), the new IT / EN texts, the A71 UAT steps from the spec's «Done when».

---

## Decisions on the drafting's open points (Claude Code, 2026-11-24)

1. `touch.thide` only fades the controls; hiding is `touch.tpad` with a controller. The placement is withdrawn only when the controls are hidden (`tpad`), never on the fade (the spec's «`touch.thide`» meant this).
2. The quick menu's inset **scales the placement uniformly** (new `Arrangement::Phone`, Task 3), never stretches it: the owner's own placement must not be distorted.
3. `touch.tscreens` sits in section `touch/layout` and is hidden in the interface by its kind (`isShown`).
4. `screens_preview` is replaced by `screens_phone_start`; `TouchPreviewScreens` goes, the placeholders are `ScreenFrame`s.
5. Starting placement with a game open: the first layout of the session's cycle, not swapped (`phoneStart` at scope `session`).
6. Resetting the `touch.tedit` row leaves `touch.tscreens` stored, harmless (screens used only with saved controls); the editor's reset + «Salva» writes `[]`.
7. Minimum size: 0.08 wide, 0.04 tall (as `screenEditor.ts`).
8. Inset and placement travel as two calls, the placement first; a combined command only if the A71 shows a flicker.
9. `TouchLayer` keyed on `session.launches` so a new game gets the placement again.
