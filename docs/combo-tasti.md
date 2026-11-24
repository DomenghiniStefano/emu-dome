# Controller button combos

Research and proposal for the controller's button combinations ("chords"), like Steam's.
Goal: whoever plays with a pad uses the main features without a keyboard.

Numbers in square brackets point to the sources at the bottom.
Where a source is from the community and not official, we say so.
What we did not find in a reliable source is marked **to verify**.

## 1. What Steam does

### Steam Input: "Guide Button Chord Configuration"

- Steam has three global configurations: Big Picture, Desktop and **Guide Button Chord** [1].
- Guide chords are "a series of inputs that perform actions when pressed together with the Guide button" [1].
- The Guide button is "the large central button with the logo, usually reserved for the system" [1].
- It is **the only way** to assign the Guide button in any configuration [1].
- Official examples, with a PlayStation pad [1]:
  - Guide + left stick up: raises the volume.
  - Guide + Triangle (Y): turns the controller off.
- The official documentation does **not** list every default chord [1].
- Community list for the Steam Controller (unofficial) [2]:
  - Guide + right bumper (R1): in-game screenshot.
  - Guide + left bumper (L1): magnifier.
  - Guide + Back/Select: on-screen keyboard.
  - Guide + Y: turns the controller off.
- An old Steam Controller post says instead Guide + Select for the keyboard
  and Guide + right trigger for the screenshot [3]. The chords changed over time: **to verify** on the current client.
- Xbox, PlayStation and generic pads go through Steam Input only if you ticked the matching support box [1].

### Steam Deck: Steam button + ...

Holding Steam (or the "..." button) for a few seconds shows the list of global shortcuts [4][5].
The "..." button counts as Steam. They work in Game Mode [5].

| Combo | Action |
|---|---|
| Hold Steam | Shows the list of shortcuts |
| Steam + B (held) | Force-closes the game |
| Steam + X | On-screen keyboard |
| Steam + L1 | Magnifier |
| Steam + R1 | Screenshot |
| Steam + L2 | Right click |
| Steam + R2 | Left click |
| Steam + right stick | Mouse with the stick |
| Steam + right trackpad | Mouse with the trackpad (click = left click) |
| Steam + left stick up/down | Brightness up/down |
| Steam + d-pad right | Enter |
| Steam + d-pad down | Tab |
| Steam + d-pad left | Esc |

Source: EmuDeck [5] (community). The Steam community guide confirms Steam + X and Steam + L1,
and adds **Steam + A = Quick Access menu with an external controller** [6].
We did not find a Valve support page with the full list: **to verify** on the Deck's on-screen list.

## 2. RetroArch's convention

### "Enable Hotkeys" button (hotkey enable)

- If you assign `Enable Hotkeys` to a button, **you must hold it** for the other shortcuts to work [7][8].
- In the configuration file the option is `input_enable_hotkey_btn`, empty by default [8].
- `input_hotkey_block_delay = 5`: frames to wait before the hotkey button blocks the game's input.
  Needed when that button also has another function [8].

### Combo to open the menu (`input_menu_toggle_gamepad_combo`)

Default value `0` (none). Options [8]:

| Value | Combo |
|---|---|
| 0 | None |
| 1 | Down + Y + L1 + R1 |
| 2 | L3 + R3 |
| 3 | L1 + R1 + Start + Select |
| 4 | Start + Select |
| 5 | L3 + R1 |
| 6 | L1 + R1 |
| 7 | Hold Start (2 seconds) |
| 8 | Hold Select (2 seconds) |
| 9 | Down + Select |
| 10 | L2 + R2 |

`input_quit_gamepad_combo` (quit) has the same options, default `0` [8].

### Default shortcuts

Out of the box RetroArch assigns them to the **keyboard**, not to the pad [7][8]:

| Action | Key | Note |
|---|---|---|
| Save state | F2 | |
| Load state | F4 | |
| Previous / next slot | F6 / F7 | |
| Fast forward (toggle) | Space | |
| Fast forward (hold) | L | |
| Slow motion (hold) | E | `input_slowmotion` [8] |
| Rewind (hold) | R | Rewind must be enabled |
| Pause | P | |
| Screenshot | F8 | |
| Quit | Esc | |

On the pad the convention comes from the distributions and the autoconfiguration profiles [7]:

- RetroPie: **Select** is the hotkey button, Select + Start quits, Select + R1 saves [9].
- JELOS: Select + Start (twice) quits, Select + R1 saves, Select + L1 loads [10].
- EmuDeck, a scheme shared by several emulators (community) [5]:

| Combo | Action |
|---|---|
| Select + R3 | Quick menu |
| Select + R1 | Save state |
| Select + L1 | Load state |
| Select + R2 | Fast forward (in PPSSPP: rewind) |
| Select + L2 | Rewind or slow motion, depending on the emulator |
| Select + A | Pause |
| Select + B | Screenshot |
| Select + L3 | Restart the emulation |
| Select + Start | Stop the emulation |
| Select + X | FPS in RetroArch, disc change in DuckStation |
| Select + Y | Disc change in RetroArch |

### The other emulators, briefly

- **DuckStation**: Select + Start quits, Select + R2 fast forward (EmuDeck table) [11].
- **Dolphin**: Select + Start quits; no combo for the menu (EmuDeck table) [11].
- **PPSSPP**: RetroDECK reports a bug with multi-button shortcuts in the flatpak version
  and falls back to Esc for the menu [12]. Community, **to verify** on the current version.
- **melonDS**: "Swap Screens" is assigned in Settings > Input > Hotkeys [13].
  On Steam Deck EmuDeck also puts it on Select + Y [13]. Community sources.
- **Azahar**: PR #1717 adds controller shortcuts to the Qt frontend [14]:
  - up to **two** inputs per shortcut, axes and back buttons included;
  - if a button has both a shortcut and a normal action, **both fire**;
  - buttons checked 60 times a second;
  - scope per single controller, per controller type or for all.

  The PR lists no default combos.
- **Citra**: no official source found on pad combos: **to verify**.

## 3. Conflicts

### Steam

- With Steam Input on, the Guide button belongs to Steam: it is the only place where it can be assigned [1].
  So Steam's Guide chords (screenshot, keyboard, power off...) fire **before** our app.
  If we use them too, two actions fire, or ours never arrives: **to verify** with SDL3 on Windows.
- Steam Guide chords to avoid (community) [2][5]: Guide + R1, Guide + L1, Guide + Select, Guide + Y,
  Guide + X (Deck), Guide + B held (Deck), Guide + L2/R2 (Deck), Guide + d-pad (Deck), Guide + sticks (volume, brightness, mouse).
- On Steam Deck the Steam + ... combos work for every game in Game Mode, non-Steam ones included [5].
  There the Steam button is not ours.
- The "Guide Button Focuses Steam" option brings Steam to the foreground on the Guide button.
  Community reports disagree on how far it can be turned off [15]: **to verify**.

### Windows: Xbox Game Bar

- In Settings > Gaming > Xbox Game Bar there is the toggle
  "Open Xbox Game Bar using this button on a controller" [16][17].
  When on, the Xbox button opens the Game Bar and steals our focus.
- The default state differs between sources (on in Windows 10, off in Windows 11): **to verify** [17].
- According to a Microsoft moderator it only works with XInput controllers [17].
- Windows 11 also has the "Controller Bar" on the Xbox button when you are not in a game [18]. Community.

### PlayStation (DualSense)

- Steam treats the PS button as Guide: the official chord examples use the PlayStation button itself [1].
- With DS4Windows or InputMapper, Steam sees an emulated Xbox 360 controller [1].
- Behaviour of the PS button on Windows without Steam (does SDL3 receive it? does anyone intercept it?): **to verify**.

### Android

- `KEYCODE_HOME` is "handled by the framework and **never delivered to applications**" [19].
- The central button of many pads arrives instead as `KEYCODE_BUTTON_MODE` ("the controller's Mode button") [19].
  Whether the system lets it through to the app depends on the pad and the Android version: **to verify** on the test phones.
- Android advises passing the events you do not use to `super`, so the compatibility layers do not break [20].

## 4. Proposal for EmuDome

> Superseded on one point by the decision in section 5: the modifier is **L3**, not Select.
> The rest of the proposal (two ways to open the menu, no destructive combos, everything
> remappable later) stands.

### Principles

- **Two ways to open the quick menu**:
  - Guide/Home pressed alone, when it reaches the app;
  - a combo that does not use Guide, for when Steam, the Game Bar or Android capture it.
- **Combo button ("M")**: **Select/Back** by default, like RetroArch, RetroPie, JELOS and EmuDeck [8][9][10][5].
  It works everywhere, with Steam and on Android too.
- Guide as the combo button only if the user chooses it. Guide + button combos collide with Steam's (section 3).
- Select is also a GBA, DS and 3DS button. A rule like RetroArch's `input_hotkey_block_delay` is needed [8]:
  - press M and then a combo button: the action starts, the game receives neither M nor the other button;
  - press M and release with no combo: the game receives Select.

  Side effect: a Select held for long in the game arrives late. A decision to take, maybe an option.
- Destructive actions (quitting, closing the game) have **no** combo: they go through the quick menu.
  RetroArch and the others use Select + Start to quit [9][10][5]: we use it for the menu, which does no harm.
- **Everything remappable** from the settings engine (Phase 1): levels Default → Global → Console → Game,
  plus the per-device axis. The UI comes from the single schema in Rust, not written by hand.
- Where a feature does not exist (screen swap on GBA, rewind before Phase 1),
  the combo does nothing and the menu shows it disabled with the reason.

### Default combos

M = combo button (Select/Back by default). "Risk" is the conflict if M becomes Guide.

| Combo | Action | Precedent | Risk if M = Guide |
|---|---|---|---|
| Guide (alone) | Open/close the quick menu | Home button already used in EmuDome | **High**: Steam [1][15], Game Bar [16], Android [19] |
| M + Start | Open/close the quick menu (fallback) | RetroArch combo 4 [8] | Low |
| M + R1 | Quick save to the current slot | RetroPie, JELOS, EmuDeck [9][10][5] | **High**: Steam screenshot [2][5] |
| M + L1 | Quick load from the current slot | JELOS, EmuDeck [10][5] | **High**: Steam magnifier [2][5] |
| M + d-pad right | Next slot | (RetroArch: F7 on the keyboard [8]) | **High** on the Deck: Enter [5] |
| M + d-pad left | Previous slot | (RetroArch: F6 on the keyboard [8]) | **High** on the Deck: Esc [5] |
| M + R2 (held) | Fast forward while held | EmuDeck [5] | **High** on the Deck: left click [5] |
| M + d-pad up | Fast forward on/off | (RetroArch: Space [8]) | Low, to verify |
| M + d-pad down | Slow motion on/off | (RetroArch: E held [8]) | **High** on the Deck: Tab [5] |
| M + L2 (held) | Rewind (Phase 1) | EmuDeck [5] | **High** on the Deck: right click [5] |
| M + A | Pause | EmuDeck [5] | Medium: on the Deck Steam + A opens Quick Access [6] |
| M + B | Screenshot | EmuDeck [5] | **High** on the Deck: B held closes the game [5] |
| M + Y | Swap screens (NDS/3DS) | EmuDeck for melonDS [13] | **High**: turns the controller off [1][2] |
| M + R3 | Next screen layout (NDS/3DS) | None found | Low, to verify |
| M + L3 | Fullscreen on/off | None found | Low, to verify |
| M + X | FPS indicator | RetroArch on EmuDeck [5] | **High** on the Deck: keyboard [5] |

Notes on the table:

- With M = Select almost nothing collides with Steam: Steam uses Guide, not Select.
  Exception: Steam Controller, Guide + Select = keyboard [2]. It does not affect us while M is Select.
- With M = Guide almost everything collides. That is why Guide stays the default **only** for the quick menu.
- Pads without L3/R3 (many SNES-style ones) lose layout and fullscreen: those stay in the quick menu.
- The d-pad under M does not reach the game: that is fine, because M held blocks the game's input.

### To verify before implementing

- SDL3 on Windows with Steam open: does the Guide button reach the app? With Steam Input on and off.
- SDL3 with the Game Bar on: does Guide reach the app and also open the Game Bar?
- Android: does `KEYCODE_BUTTON_MODE` reach the app on common pads (Xbox, DualSense, 8BitDo)?
- M's block time (RetroArch uses 5 frames [8]) and how a held Select behaves in games.
- Whether to detect Steam or the Game Bar and show a warning ("Steam uses the Guide button: open the menu with Select + Start").

## 5. Decision and implementation

### The modifier is L3

- **L3** (left stick click) is the combo modifier ("M") by default; `pad.mod` changes it and `pad.combos` the
  combos.
- Why not Select: Select is a GBA, DS and 3DS button, so it would need RetroArch's block delay
  (section 4) and a held Select would reach games late. **No GBA, NDS or 3DS game uses a stick click**,
  so L3 can be taken whole: no delay, no ambiguity.
- Why not Guide: Steam, the Game Bar and Android take it (section 3). Guide alone still opens the menu.
- Cost: pads without stick clicks (many SNES-style ones) have no combos; they still have Guide and the menu.

### Rules as implemented

- **L3 never reaches the core.** While L3 is held **nothing** reaches the game (face, shoulders, d-pad):
  the game sees a released pad. A button pressed with L3 stays away from the game until it is released,
  even if L3 is released first.
- Each combo fires **once per press** of the second button. A button already held when L3 goes down does not fire.
- Two hold combos: L3 + R2 (fast forward) and L3 + L2 (rewind) last until R2 / L2 is released
  (with or without L3). With `a11y.toggle` («Attiva e disattiva invece di tenere premuto») rewind
  goes on at one press and off at the next; the router still sends press and release, `HotkeyRunner`
  (`app/src-tauri/src/hotkeys.rs`) decides.
- A reserved combo (`Hotkey::status` `later`) would do nothing, its button kept away from the game like any
  button pressed with L3; today no row is reserved (a test holds it).
- While the quick menu is open only the menu combo (L3 + Start) and Guide work; the d-pad / left stick,
  A and B navigate the menu (see `app/src/nav.ts`). Y alone is the `search` step there: outside a
  game it opens the search everywhere (X01, #32); in game Y is the game's button, L3 + Y the swap.
- The left stick past half travel is the d-pad; the triggers past half travel are L2/R2.

### Implemented table

The table is `DEFAULT_COMBOS` in `crates/emudome-core/src/hotkeys.rs`, with `DEFAULT_MODIFIER = L3`,
in the order of the shortcut lists (`Hotkey::ALL`). Every row is active; the `dualScreen` rows fire and act on
DS and 3DS games only (`app/src-tauri/src/screens.rs`). `pad.combos` and `pad.mod` build other `Bindings` from it
(`crates/emudome-core/src/bindings.rs`). The actions run in `app/src-tauri/src/hotkeys.rs` (`HotkeyRunner`),
shared with the keyboard.
C01 «Scorciatoie da ricordare», C03 and the quick menu's G10 read it through `controls_bindings`;
the G12 overlay will read it the same way.

| Combo | Action | Status |
|---|---|---|
| Guide (alone) | Open/close the quick menu | done |
| L3 + Start | Open/close the quick menu (fallback for Steam / Game Bar) | done |
| L3 + R1 | Quick save to the current slot | done |
| L3 + L1 | Quick load from the current slot | done |
| L3 + d-pad right / left | Next / previous slot, 1 to 9, wrapping; toast "Slot N" | done |
| L3 + R2 (held) | Fast forward (`speed.ffval`) while held, then back to the previous speed | done |
| L3 + d-pad up | Fast forward on/off (`speed.ffval` ↔ 1×) | done |
| L3 + d-pad down | Slow motion on/off (`speed.slow` ↔ 1×) | done |
| L3 + L2 (held) | Rewind (`speed.rew`, `speed.rewlen`; toggled with `a11y.toggle`) | done (Trello #42) |
| L3 + A | Pause on/off without the menu ("Paused" chip) | done |
| L3 + B | Screenshot: PNG at the core's resolution in `saves/screenshots/<console>/<hex>/`, toast G13 | done (Trello #42, #85) |
| L3 + X | FPS indicator on/off | done |
| L3 + Y | Swap screens (NDS/3DS): toggled, or swapped while held with `screens.swap` = hold | done (Trello #28; `dualScreen`: DS and 3DS games only) |
| L3 + R3 | Next screen layout of `screens.cycle` (NDS/3DS); not in C01's list, only in C03. Under the touch layout's placement (#134) the layout stays and a toast says why (`emu://hotkey` `layoutHeld`) | done (Trello #28; `dualScreen`) |
| L3 + Select | Close/open the lid (NDS/3DS), also from the quick menu; no combo with `pad.lid` = quick menu only | done on NDS (Trello #28: melonDS DS's L2 + Y chord); the 3DS core has none (`pad.lid` unsupported on 3DS) |
| (none) | Fullscreen on/off: keyboard F11 and the quick menu only, no controller combo | done |

### Keyboard table

The keyboard's fixed keys in game are one table too: `KEYBOARD_SHORTCUTS` in
`crates/emudome-core/src/keyboard.rs`, generated for the interface (`app/src/generated/keyboard-shortcuts.json`,
read by `app/src/lib/keyboard.ts`). The webview forwards the keys raw to Rust, which routes them and carries them
out with the controllers' runner (`app/src-tauri/src/hotkeys.rs`); Esc and F11 stay in the page. C01 lists them
under «Tasti della tastiera», C03 on the keyboard's tab; the G12 overlay will read them from there. A key does
what the controller hotkey with the same id does.

| Key | Action | Status |
|---|---|---|
| Esc | Open the quick menu; in the menu, back | done |
| F1 | Quick save to the current slot | done |
| F4 | Quick load from the current slot | done |
| F2 / F3 | Previous / next slot (the same current slot as L3 + d-pad left / right) | done |
| Tab (held) | Fast forward (`speed.ffval`) while held; with `speed.ffmode` = toggle, on at one press and off at the next (the quick menu and the window losing the focus also turn it off) | done (toggle: Trello #99) |
| Backspace (held) | Rewind (toggled with `a11y.toggle`) | done (Trello #42); Select's default key moved to the right Shift |
| F9 | FPS indicator on/off | done |
| F11 | Fullscreen on/off | done |
| F12 | Screenshot | done (Trello #42) |

The current slot lives in Rust (`Pads::slot`, `app/src-tauri/src/pads.rs`): the controller thread and
the keyboard's F1–F4 use the same one, and the quick menu opens on it (command `current_slot`).
On the phone it is the quick menu's «slot rapido»: «Salva rapido» / «Carica rapido» use it, and «Altro ›
Salvataggi › Rendi rapido» sets it to the chosen slot (command `set_slot`).

It is each game's own (Trello #110): every move (L3 + d-pad left / right, F2 / F3, «Rendi rapido») is
recorded in the game's saves folder (`quick-slot.json`, `emudome_core::quick_slot`, written by the
session's I/O worker), so Drive sync carries it with the saves, and a game opens on the slot it was last
moved to (slot 1 for a game never moved, or when the file cannot be read; such a file is left as it is
until the slot is moved again).

Known limits of this version:

- Backspace was Select's default key (`KEYBOARD_DEFAULTS`, `bindings.rs`) until rewind took it
  (Trello #42): Select's default is now the right Shift, RetroArch's. A keyboard remap saved
  with Backspace on Select is kept, and then the rewind key wins (it is checked first).
- Not verified yet on hardware: whether Guide reaches the app with Steam or the Game Bar open (section 3).

## Sources

1. Steamworks, *Getting Started for Players* (Steam Input, Guide Button Chord Configuration):
   https://partner.steamgames.com/doc/features/steam_controller/getting_started_for_players
2. r/SteamController wiki (community), list of Guide chords:
   https://nyc1.lr.ggtyler.dev/r/SteamController/wiki/index
3. Steam Controller discussion (community, dated):
   https://steamcommunity.com/app/353370/discussions/0/490123832541797583
4. Steam Deck discussions (community), hold Steam or "..." for the list:
   https://steamcommunity.com/app/1675200/discussions/0/3186864655212063850/
5. EmuDeck, *Hotkeys* SteamOS (community), Deck shortcuts and the Select + ... scheme:
   https://emudeck.github.io/controls-and-hotkeys/steamos/hotkeys
6. Steam community guide, *Steam Deck Controller Guide - A Visual Introduction*:
   https://steamcommunity.com/sharedfiles/filedetails/?id=2804823261
7. Libretro Docs, *Getting started: Input and controls*:
   https://docs.libretro.com/guides/input-and-controls/
8. RetroArch, reference `retroarch.cfg`:
   https://raw.githubusercontent.com/libretro/RetroArch/master/retroarch.cfg
9. RetroPie Docs, *RetroArch Configuration*:
   https://retropie.org.uk/docs/RetroArch-Configuration
10. JELOS, controls documentation (copy on git.wbrawner.com):
    https://git.wbrawner.com/wbrawner/jelos.org/raw/commit/d1ab5b7e0e448f990be44e87193e355a5911f0f8/docs/play/controls.md
11. EmuDeck, *Hotkeys* Windows (community):
    https://emudeck.github.io/controls-and-hotkeys/windows/hotkeys/
12. RetroDECK, *Steam Deck Hotkeys and controls* (community):
    https://repo.retrodeck.net/Xargon/RetroDECK/src/branch/feat/ngage/wiki-rtd/Steam-Deck-Hotkeys-and-controls.md
13. EmuDeck, melonDS (community): https://emudeck.github.io/emulators/steamos/melonds/ and
    https://www.pokemoncoders.com/how-to-switch-screens-on-melonds/
14. Azahar, PR #1717 (controller shortcuts): https://github.com/azahar-emu/azahar/pull/1717
15. Steam discussions on "Guide Button Focuses Steam" (community):
    https://steamcommunity.com/discussions/forum/1/2299597907724592000
16. Microsoft Support, Xbox Game Bar with a screen reader (exact name of the toggle):
    https://support.microsoft.com/en-us/accessibility/windows/use-a-screen-reader-to-record-your-screen-with-xbox-game-bar
17. Microsoft Q&A, preventing the Game Bar from opening on the Xbox button:
    https://learn.microsoft.com/en-us/answers/questions/5892017/need-to-prevent-gamebar-from-opening-ever-under-an
    and ElevenForum (community):
    https://www.elevenforum.com/t/enable-or-disable-open-game-bar-using-xbox-button-on-controller-in-windows-11.4290/
18. ElevenForum, Controller Bar on the Xbox button (community):
    https://elevenforum.com/t/enable-or-disable-open-controller-bar-using-xbox-button-on-controller-in-windows.14424/
19. Android Developers, `KeyEvent` (`KEYCODE_HOME`, `KEYCODE_BUTTON_MODE`):
    https://developer.android.com/reference/android/view/KeyEvent
20. Android Developers, *Handle controller actions*:
    https://developer.android.com/develop/ui/views/touch-and-input/game-controllers/controller-input
