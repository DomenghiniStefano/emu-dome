use super::{valid_list, AbLayout, Family, PadConfig, Pos, Table, REMAPPABLE};
use crate::hotkeys::Hotkey;
use crate::input::Button;
use crate::session::Speed;
use crate::settings::Value;

fn config(values: &[(&str, Value)]) -> PadConfig {
    PadConfig::from_settings(|key| {
        values
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.clone())
    })
}

fn list(items: &[&str]) -> Value {
    Value::List(items.iter().map(|s| (*s).to_owned()).collect())
}

fn mask(buttons: &[Button]) -> u16 {
    buttons.iter().fold(0, |m, b| m | b.bit())
}

#[test]
fn ids_round_trip() {
    for p in Pos::ALL {
        assert_eq!(Pos::from_id(p.id()), Some(p));
        assert_eq!(Pos::of_button(p.button()), p);
    }
    for f in Family::ALL {
        assert_eq!(Family::from_id(f.id()), Some(f));
    }
}

#[test]
fn by_default_a_controller_is_positional() {
    let map = PadConfig::default().pad_map(Family::Xbox);
    // The right face button is the game's A, the bottom one its B.
    assert_eq!(map.apply(Pos::East.bit()), Button::A.bit());
    assert_eq!(map.apply(Pos::South.bit()), Button::B.bit());
    assert_eq!(map.apply(Pos::North.bit()), Button::X.bit());
    assert_eq!(map.apply(Pos::West.bit()), Button::Y.bit());
    assert_eq!(map.apply(Pos::L2.bit()), Button::L2.bit());
    assert_eq!(map.apply(Pos::R3.bit()), Button::R3.bit());
}

#[test]
fn printed_layout_swaps_the_faces_only_on_pads_without_nintendo_letters() {
    let c = config(&[("pad.ab", Value::Text("printed".into()))]);
    assert_eq!(c.layout, AbLayout::Printed);
    let xbox = c.pad_map(Family::Xbox);
    assert_eq!(xbox.apply(Pos::South.bit()), Button::A.bit());
    assert_eq!(xbox.apply(Pos::East.bit()), Button::B.bit());
    assert_eq!(xbox.apply(Pos::West.bit()), Button::X.bit());
    assert_eq!(xbox.apply(Pos::North.bit()), Button::Y.bit());
    let switch = c.pad_map(Family::Switch);
    assert_eq!(switch.apply(Pos::East.bit()), Button::A.bit());
}

#[test]
fn a_remap_override_applies_to_its_family_only() {
    let c = config(&[(
        "pad.remap",
        list(&["xbox:a=south", "xbox:b=east", "switch:l=r1"]),
    )]);
    let xbox = c.pad_map(Family::Xbox);
    assert_eq!(xbox.apply(Pos::South.bit()), Button::A.bit());
    assert_eq!(xbox.apply(Pos::East.bit()), Button::B.bit());
    let ps = c.pad_map(Family::PlayStation);
    assert_eq!(ps.apply(Pos::East.bit()), Button::A.bit());
    // Switch: L and R both on R1 (the interface offers the swap; the engine keeps both).
    let switch = c.pad_map(Family::Switch);
    assert_eq!(switch.apply(Pos::R1.bit()), mask(&[Button::L, Button::R]));
    assert_eq!(switch.apply(Pos::L1.bit()), 0);
}

#[test]
fn none_leaves_a_game_button_without_a_source() {
    let c = config(&[("pad.remap", list(&["generic:select=none"]))]);
    let table = c.remap_table(Family::Generic);
    let select = table.iter().find(|(b, _, _)| *b == Button::Select).unwrap();
    assert_eq!(select.1, None);
    assert!(select.2);
    assert_eq!(c.pad_map(Family::Generic).apply(Pos::Back.bit()), 0);
}

#[test]
fn an_override_equal_to_the_default_is_not_a_change() {
    let c = config(&[("pad.remap", list(&["xbox:a=east"]))]);
    let table = c.remap_table(Family::Xbox);
    assert!(table.iter().all(|(_, _, changed)| !changed));
}

#[test]
fn the_keyboard_table_has_the_design_keys_and_overrides() {
    let c = config(&[("pad.remap", list(&["keyboard:start=Space"]))]);
    let table = c.remap_table(Family::Keyboard);
    assert_eq!(table.len(), REMAPPABLE.len());
    let get = |b: Button| table.iter().find(|(x, _, _)| *x == b).unwrap().clone();
    assert_eq!(get(Button::A).1.as_deref(), Some("KeyX"));
    assert_eq!(
        get(Button::Start),
        (Button::Start, Some("Space".into()), true)
    );
}

#[test]
fn invalid_lists_are_refused() {
    let ok = |items: &[&str], table| {
        valid_list(
            table,
            &items.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>(),
        )
    };
    assert!(ok(&["xbox:a=south", "keyboard:a=KeyJ"], Table::Remap));
    assert!(ok(&[], Table::Remap));
    assert!(!ok(&["xbox:a=KeyJ"], Table::Remap), "a key on a controller");
    assert!(!ok(&["keyboard:a=Key J"], Table::Remap), "not a key code");
    assert!(
        !ok(&["xbox:l3=south"], Table::Remap),
        "L3 is not remappable"
    );
    assert!(!ok(&["gamecube:a=south"], Table::Remap), "unknown family");
    assert!(
        !ok(&["xbox:a=south", "xbox:a=east"], Table::Remap),
        "same button twice"
    );
    assert!(ok(&["save=north", "pause=none"], Table::Combos));
    assert!(!ok(&["save=north", "save=south"], Table::Combos));
    assert!(!ok(&["rewind=l2"], Table::Combos));
}

#[test]
fn invalid_items_in_a_stored_list_are_ignored() {
    let c = config(&[("pad.remap", list(&["xbox:a=nope", "xbox:b=north"]))]);
    assert_eq!(c.remap, vec!["xbox:b=north".to_owned()]);
}

#[test]
fn combos_follow_the_modifier_and_the_overrides() {
    let c = config(&[
        ("pad.mod", Value::Text("select".into())),
        ("pad.combos", list(&["save=north", "fps=none"])),
    ]);
    let b = c.bindings();
    assert_eq!(b.modifier, Button::Select);
    assert!(b.combos.contains(&(Button::X, Hotkey::SaveState)));
    assert!(!b.combos.iter().any(|(_, h)| *h == Hotkey::FpsOverlay));
    // Untouched combos keep their default.
    assert!(b.combos.contains(&(Button::L, Hotkey::LoadState)));
}

#[test]
fn a_combo_on_the_modifier_itself_is_dropped() {
    let c = config(&[
        ("pad.mod", Value::Text("r3".into())),
        ("pad.combos", list(&["pause=r3"])),
    ]);
    assert!(!c.bindings().combos.iter().any(|(_, h)| *h == Hotkey::Pause));
}

#[test]
fn the_default_bindings_are_the_routers_defaults() {
    assert_eq!(
        PadConfig::default().bindings(),
        crate::hotkeys::Bindings::default()
    );
}

#[test]
fn confirm_and_repeat_come_from_the_settings() {
    let c = config(&[
        ("pad.confirm", Value::Text("bottom".into())),
        ("pad.repeat", Value::Text("fast".into())),
    ]);
    let b = c.bindings();
    assert_eq!(b.confirm, Button::B);
    assert!(b.repeat_delay < crate::hotkeys::REPEAT_DELAY);
}

#[test]
fn the_stick_respects_the_dead_zone_and_the_threshold() {
    let c = config(&[
        ("pad.dead", Value::Int(20)),
        ("pad.stickdpad", Value::Int(50)),
    ]);
    let full = 32767;
    assert_eq!(c.stick_dpad_mask(0, 0), 0);
    assert_eq!(c.stick_dpad_mask(full / 2 + 400, 0), Button::Right.bit());
    assert_eq!(c.stick_dpad_mask(full / 2 - 400, 0), 0);
    assert_eq!(c.stick_dpad_mask(0, -full), Button::Up.bit());
    assert_eq!(
        c.stick_dpad_mask(full, full),
        mask(&[Button::Right, Button::Down])
    );
    // A threshold inside the dead zone: the dead zone wins.
    let low = config(&[
        ("pad.dead", Value::Int(30)),
        ("pad.stickdpad", Value::Int(10)),
    ]);
    assert_eq!(low.stick_dpad_mask(full / 4, 0), 0);
    assert_eq!(low.stick_dpad_mask(full / 2, 0), Button::Right.bit());
}

#[test]
fn the_trigger_threshold_is_a_share_of_the_travel() {
    let c = config(&[("pad.trig", Value::Int(50))]);
    assert_eq!(c.trigger_threshold(), 16383);
    let wrong = config(&[("pad.trig", Value::Text("x".into()))]);
    assert_eq!(wrong.trigger, 50);
}

#[test]
fn fast_forward_and_slow_motion_follow_the_speed_settings() {
    // Defaults of `speed.ffval` (4×) and `speed.slow` (0.5×).
    let default = config(&[]);
    assert_eq!(default.fast_forward, Speed::Factor(4.0));
    assert_eq!(default.slow_motion, Speed::Factor(0.5));
    let set = config(&[
        ("speed.ffval", Value::Text("max".into())),
        ("speed.slow", Value::Text("0.25".into())),
    ]);
    assert_eq!(set.fast_forward, Speed::Max);
    assert_eq!(set.slow_motion, Speed::Factor(0.25));
    let three = config(&[("speed.ffval", Value::Text("3".into()))]);
    assert_eq!(three.fast_forward, Speed::Factor(3.0));
}

#[test]
fn a_speed_value_out_of_place_falls_back_to_the_default() {
    let odd = config(&[
        ("speed.ffval", Value::Text("fast".into())),
        ("speed.slow", Value::Text("0".into())),
    ]);
    assert_eq!(odd.fast_forward, Speed::Factor(4.0));
    assert_eq!(odd.slow_motion, Speed::Factor(0.5));
    assert_eq!(Speed::from_option("max"), Some(Speed::Max));
    assert_eq!(Speed::from_option("0.75"), Some(Speed::Factor(0.75)));
    assert_eq!(Speed::from_option("-2"), None);
    assert_eq!(Speed::from_option("NaN"), None);
}
