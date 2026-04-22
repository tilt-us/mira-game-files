use bevy_input::prelude::{ButtonInput, KeyCode};
use game_shared::utils::input_utils::{
    KeyType, convert_string_to_key_code, detect_key_action, fetch_key_code,
};

use KeyType::{CombinedKey, MultiKey, SingleKey};

fn keyboard_with_pressed(keys: &[KeyCode]) -> ButtonInput<KeyCode> {
    let mut keyboard = ButtonInput::default();
    for key in keys {
        keyboard.press(*key);
    }
    keyboard
}

#[test]
fn fetch_key_code_returns_none_for_empty_input() {
    assert_eq!(fetch_key_code(""), None);
}

#[test]
fn fetch_key_code_parses_single_multi_and_combined() {
    assert_eq!(fetch_key_code("  A  "), Some(SingleKey("A".to_string())));
    assert_eq!(
        fetch_key_code("A|B|C"),
        Some(MultiKey(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string()
        ]))
    );
    assert_eq!(
        fetch_key_code("A+B+X"),
        Some(CombinedKey(("A".to_string(), "B".to_string())))
    );
}

#[test]
fn detect_key_action_for_single_key() {
    let keyboard = keyboard_with_pressed(&[KeyCode::KeyA]);
    assert!(detect_key_action("A".to_string(), &keyboard));
    assert!(!detect_key_action("B".to_string(), &keyboard));
}

#[test]
fn detect_key_action_for_combined_key() {
    let both_pressed = keyboard_with_pressed(&[KeyCode::KeyA, KeyCode::KeyB]);
    assert!(detect_key_action("A+B".to_string(), &both_pressed));

    let one_pressed = keyboard_with_pressed(&[KeyCode::KeyA]);
    assert!(!detect_key_action("A+B".to_string(), &one_pressed));
}

#[test]
fn detect_key_action_for_multi_key() {
    let keyboard = keyboard_with_pressed(&[KeyCode::KeyB]);
    assert!(detect_key_action("A|B|C".to_string(), &keyboard));
    assert!(!detect_key_action("X|Y|Z".to_string(), &keyboard));
}

#[test]
fn convert_string_to_key_code_maps_known_values_and_aliases() {
    assert_eq!(convert_string_to_key_code("A"), Some(KeyCode::KeyA));
    assert_eq!(convert_string_to_key_code("Enter"), Some(KeyCode::Enter));
    assert_eq!(
        convert_string_to_key_code("CtrlLeft"),
        Some(KeyCode::ControlLeft)
    );
    assert_eq!(convert_string_to_key_code("UnknownKey"), None);
}
