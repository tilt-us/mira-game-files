use crate::utils::input_utils::KeyType::{CombinedKey, MultiKey, SingleKey};
use bevy::log::{debug, warn};
use bevy_input::prelude::{ButtonInput, KeyCode};

/// Represents different types of keys that can be used in a system or application.
///
/// This enumeration defines three variants:
///
/// * `SingleKey`: A single key represented as a `String`.
/// * `MultiKey`: A collection of keys represented as a `Vec<String>`.
/// * `CombinedKey`: A pair of keys represented as a tuple `(String, String)`.
#[derive(Debug, PartialEq)]
pub enum KeyType {
    SingleKey(String),
    MultiKey(Vec<String>),
    CombinedKey((String, String)),
}

/// Fetches a `KeyType` representation based on the given named key.
///
/// # Arguments
/// * `named_key` - A string slice representing the input key. The structure of the key determines the type of `KeyType` returned:
///   - If `named_key` is empty, it logs a warning and returns `None`.
///   - If `named_key` contains the delimiter `"|"`, it is treated as a list of keys and a `KeyType::MultiKey` is returned.
///   - If `named_key` contains the delimiter `"+"`, it is treated as a combined key. A maximum of two keys is considered, with a warning logged if more than two keys are provided. A `KeyType::CombinedKey` is returned.
///   - Otherwise, it is treated as a single key and a `KeyType::SingleKey` is returned.
///   - `named_key` is trimmed before processing.
///
/// # Returns
/// * `Option<KeyType>`: An enum representing the type of key (`SingleKey`, `MultiKey`, or `CombinedKey`).
///   - `None` is returned if the input `named_key` is empty.
///
/// # Enum Variants
/// * `KeyType::SingleKey(String)` - A single key represented as a string.
/// * `KeyType::MultiKey(Vec<String>)` - A list of multiple keys, separated by `"|"`.
/// * `KeyType::CombinedKey((String, String))` - A combination of two keys, separated by `"+"`.
///
/// # Logging
/// * Logs warnings for the following scenarios:
///   - If `named_key` is empty.
///   - If a combined key contains more than two keys (extra keys are ignored).
/// * Logs debug information for parsed key lists.
///
/// # Example
/// ```ignore
/// let single = fetch_key_code("A");
/// let multi = fetch_key_code("A|B|C");
/// let combined = fetch_key_code("A+B");
/// let empty = fetch_key_code("");
/// ```
pub fn fetch_key_code(named_key: &str) -> Option<KeyType> {
    if named_key.is_empty() {
        warn!("Key name is empty!");
        return None;
    }

    // If this contains, the key is key list.
    if named_key.contains("|") {
        let key_list: Vec<&str> = named_key.trim().split("|").collect();
        debug!("Fetched Key list: {:?}", key_list);

        return Some(MultiKey(
            key_list.iter().map(|key| key.to_string()).collect(),
        ));
    }

    // If this contains, the key is combined key.
    if named_key.contains("+") {
        let key_list: Vec<&str> = named_key.trim().split("+").collect();
        debug!("Fetched Combined Key list: {:?}", key_list);
        if key_list.len() > 2 {
            warn!("Combined key is too long! (Max 2 keys) other ones later ar ignored!");
        }

        return Some(CombinedKey((
            key_list[0].to_string(),
            key_list[1].to_string(),
        )));
    }

    // If this contains, the key is single key.
    Some(SingleKey(named_key.trim().to_string()))
}

/// Detects if a specified key or combination of keys has been pressed.
///
/// # Arguments
///
/// * `named_key` - A `String` representing the key(s) to detect. This can be a single key,
///   a combination of two keys, or a list of multiple keys.
/// * `keyboard` - A reference to a `ButtonInput<KeyCode>` object, which provides access
///   to the state of the keyboard for detecting key inputs.
///
/// # Returns
///
/// * `true` if the specified key or combination of keys has been pressed.
/// * `false` otherwise.
///
/// # Behavior
///
/// 1. The function uses the `fetch_key_code` helper to determine the type of key (single, combined, or multiple).
/// 2. For each detected key type:
///    - **SingleKey**: Converts the string key name into a `KeyCode` using
///      `convert_string_to_key_code` and checks if it was just pressed.
///    - **CombinedKey**: Converts both string key names into `KeyCode`s,
///      and checks if both were just pressed simultaneously.
///    - **MultiKey**: Iterates over the list of key names, converts each into a `KeyCode`,
///      and returns `true` if anyone has been just pressed.
/// 3. If no matching key is pressed or the key type is `None`, the function returns `false`.
///
/// # Panics
///
/// The function will panic if any key name in `named_key` cannot be converted
/// to a valid `KeyCode` using `convert_string_to_key_code`. The panic message
/// includes information about the missing key.
///
/// # Example
///
/// ```ignore
/// let key_status = detect_key_action(String::from("A"), &keyboard_input);
/// if key_status {
///     println!("A was just pressed!");
/// }
/// ```
pub fn detect_key_action(named_key: String, keyboard: &ButtonInput<KeyCode>) -> bool {
    let key_type = fetch_key_code(&named_key);
    if let Some(key_type) = key_type {
        match key_type {
            SingleKey(key_name) => {
                let key_code = convert_string_to_key_code(&key_name)
                    .expect(format!("Key not found! ( {} )", key_name).as_str());
                return keyboard.just_pressed(key_code);
            }

            CombinedKey((key_name_1, key_name_2)) => {
                let key_code_1 = convert_string_to_key_code(&key_name_1)
                    .expect(format!("Key not found! ( {} )", key_name_1).as_str());
                let key_code_2 = convert_string_to_key_code(&key_name_2)
                    .expect(format!("Key not found! ( {} )", key_name_2).as_str());
                return keyboard.just_pressed(key_code_1) && keyboard.just_pressed(key_code_2);
            }

            MultiKey(key_list) => {
                for key_name in key_list {
                    let key_code = convert_string_to_key_code(&key_name)
                        .expect(format!("Key not found! ( {} )", key_name).as_str());
                    if keyboard.just_pressed(key_code) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Converts a string representation of a key name to an `Option<KeyCode>`.
///
/// # Parameters
/// - `key_name`: A string slice containing the name of the key as
///   described in the function's match arms (case-sensitive).
///
/// # Returns
/// - `Some(KeyCode)`: If a matching `KeyCode` variant is found.
/// - `None`: If the input `key_name` does not match any known key.
///
/// # Supported Keys
/// The function supports:
/// - Modifier keys: "Escape", "Backspace", "Enter", "Space", "Tab",
///   "ShiftLeft", "ShiftRight", "ControlLeft", "ControlRight", "AltLeft",
///   "AltRight", "CapsLock", "NumLock".
/// - Arrow keys: "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight".
/// - Function keys: "F1" to "F24".
/// - Digits: "0" to "9".
/// - Alphabet keys: "A" to "Z".
/// - Special keys: "Delete", "PrintScreen", "Pause", "Insert", "Home",
///   "PageUp", "PageDown", "End".
///
/// # Examples
/// ```ignore
/// use crate::convert_string_to_key_code;
/// use crate::KeyCode;
///
/// assert_eq!(convert_string_to_key_code("Enter"), Some(KeyCode::Enter));
/// assert_eq!(convert_string_to_key_code("F5"), Some(KeyCode::F5));
/// assert_eq!(convert_string_to_key_code("Z"), Some(KeyCode::KeyZ));
/// assert_eq!(convert_string_to_key_code("UnknownKey"), None);
/// ```
///
/// # Notes
/// - Key names are case-sensitive.
/// - Some keys (e.g., "ControlLeft" and "CtrlLeft") are interchangeable
///   when matched against their corresponding `KeyCode` variant.
pub fn convert_string_to_key_code(key_name: &str) -> Option<KeyCode> {
    match key_name {
        "Escape" => Some(KeyCode::Escape),
        "Backspace" => Some(KeyCode::Backspace),
        "Enter" => Some(KeyCode::Enter),
        "Space" => Some(KeyCode::Space),
        "Tab" => Some(KeyCode::Tab),
        "Delete" => Some(KeyCode::Delete),
        "CapsLock" => Some(KeyCode::CapsLock),
        "NumLock" => Some(KeyCode::NumLock),
        "PrintScreen" => Some(KeyCode::PrintScreen),
        "Pause" => Some(KeyCode::Pause),
        "Insert" => Some(KeyCode::Insert),
        "Home" => Some(KeyCode::Home),
        "PageUp" => Some(KeyCode::PageUp),
        "End" => Some(KeyCode::End),
        "PageDown" => Some(KeyCode::PageDown),
        "ShiftLeft" => Some(KeyCode::ShiftLeft),
        "ShiftRight" => Some(KeyCode::ShiftRight),
        "ControlLeft" | "CtrlLeft" => Some(KeyCode::ControlLeft),
        "ControlRight" | "CtrlRight" => Some(KeyCode::ControlRight),
        "AltLeft" => Some(KeyCode::AltLeft),
        "AltRight" => Some(KeyCode::AltRight),
        "ArrowUp" => Some(KeyCode::ArrowUp),
        "ArrowDown" => Some(KeyCode::ArrowDown),
        "ArrowLeft" => Some(KeyCode::ArrowLeft),
        "ArrowRight" => Some(KeyCode::ArrowRight),
        "A" => Some(KeyCode::KeyA),
        "B" => Some(KeyCode::KeyB),
        "C" => Some(KeyCode::KeyC),
        "D" => Some(KeyCode::KeyD),
        "E" => Some(KeyCode::KeyE),
        "F" => Some(KeyCode::KeyF),
        "G" => Some(KeyCode::KeyG),
        "H" => Some(KeyCode::KeyH),
        "I" => Some(KeyCode::KeyI),
        "J" => Some(KeyCode::KeyJ),
        "K" => Some(KeyCode::KeyK),
        "L" => Some(KeyCode::KeyL),
        "M" => Some(KeyCode::KeyM),
        "N" => Some(KeyCode::KeyN),
        "O" => Some(KeyCode::KeyO),
        "P" => Some(KeyCode::KeyP),
        "Q" => Some(KeyCode::KeyQ),
        "R" => Some(KeyCode::KeyR),
        "S" => Some(KeyCode::KeyS),
        "T" => Some(KeyCode::KeyT),
        "U" => Some(KeyCode::KeyU),
        "V" => Some(KeyCode::KeyV),
        "W" => Some(KeyCode::KeyW),
        "X" => Some(KeyCode::KeyX),
        "Y" => Some(KeyCode::KeyY),
        "Z" => Some(KeyCode::KeyZ),
        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "F4" => Some(KeyCode::F4),
        "F5" => Some(KeyCode::F5),
        "F6" => Some(KeyCode::F6),
        "F7" => Some(KeyCode::F7),
        "F8" => Some(KeyCode::F8),
        "F9" => Some(KeyCode::F9),
        "F10" => Some(KeyCode::F10),
        "F11" => Some(KeyCode::F11),
        "F12" => Some(KeyCode::F12),
        "F13" => Some(KeyCode::F13),
        "F14" => Some(KeyCode::F14),
        "F15" => Some(KeyCode::F15),
        "F16" => Some(KeyCode::F16),
        "F17" => Some(KeyCode::F17),
        "F18" => Some(KeyCode::F18),
        "F19" => Some(KeyCode::F19),
        "F20" => Some(KeyCode::F20),
        "F21" => Some(KeyCode::F21),
        "F22" => Some(KeyCode::F22),
        "F23" => Some(KeyCode::F23),
        "F24" => Some(KeyCode::F24),
        "0" => Some(KeyCode::Digit0),
        "1" => Some(KeyCode::Digit1),
        "2" => Some(KeyCode::Digit2),
        "3" => Some(KeyCode::Digit3),
        "4" => Some(KeyCode::Digit4),
        "5" => Some(KeyCode::Digit5),
        "6" => Some(KeyCode::Digit6),
        "7" => Some(KeyCode::Digit7),
        "8" => Some(KeyCode::Digit8),
        "9" => Some(KeyCode::Digit9),
        _ => None,
    }
}
