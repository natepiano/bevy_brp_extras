//! Keyboard input simulation for BRP extras

use std::str::FromStr;
use std::time::Duration;

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult, error_codes};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

/// Maximum duration for holding keys in milliseconds (1 minute)
const MAX_KEY_DURATION_MS: u32 = 60_000;

/// Default duration for holding keys in milliseconds
const DEFAULT_KEY_DURATION_MS: u32 = 100;

/// Component that tracks keys that need to be released after a duration
#[derive(Component)]
pub struct TimedKeyRelease {
    /// The key codes to release
    pub keys:  Vec<KeyCode>,
    /// Timer tracking the remaining duration
    pub timer: Timer,
}

/// Wrapper enum for Bevy's `KeyCode` with strum derives for string conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, EnumIter, Display)]
#[strum(serialize_all = "PascalCase")]
#[allow(missing_docs)]
pub enum KeyCodeWrapper {
    // Letters
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,

    // Digits
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,

    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    // Modifiers
    AltLeft,
    AltRight,
    ControlLeft,
    ControlRight,
    ShiftLeft,
    ShiftRight,
    SuperLeft,
    SuperRight,

    // Navigation
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    End,
    Home,
    PageDown,
    PageUp,

    // Editing
    Backspace,
    Delete,
    Enter,
    Escape,
    Insert,
    Space,
    Tab,

    // Numpad
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadMultiply,
    NumpadSubtract,
    NumpadDecimal,
    NumpadEnter,

    // Media and special
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    BrowserBack,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    CapsLock,
    NumLock,
    ScrollLock,
    PrintScreen,
    Pause,
    MediaPlayPause,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,

    // Punctuation and symbols
    Backquote,
    Backslash,
    BracketLeft,
    BracketRight,
    Comma,
    Equal,
    Minus,
    Period,
    Quote,
    Semicolon,
    Slash,
}

/// Macro to generate key code mappings
macro_rules! key_code_mapping {
    ($self:expr, $($variant:ident),* $(,)?) => {
        match $self {
            $(Self::$variant => KeyCode::$variant,)*
        }
    };
}

impl KeyCodeWrapper {
    /// Convert the wrapper to a Bevy `KeyCode`
    #[must_use]
    pub const fn to_key_code(self) -> KeyCode {
        key_code_mapping!(
            self,
            // Letters
            KeyA,
            KeyB,
            KeyC,
            KeyD,
            KeyE,
            KeyF,
            KeyG,
            KeyH,
            KeyI,
            KeyJ,
            KeyK,
            KeyL,
            KeyM,
            KeyN,
            KeyO,
            KeyP,
            KeyQ,
            KeyR,
            KeyS,
            KeyT,
            KeyU,
            KeyV,
            KeyW,
            KeyX,
            KeyY,
            KeyZ,
            // Digits
            Digit0,
            Digit1,
            Digit2,
            Digit3,
            Digit4,
            Digit5,
            Digit6,
            Digit7,
            Digit8,
            Digit9,
            // Function keys
            F1,
            F2,
            F3,
            F4,
            F5,
            F6,
            F7,
            F8,
            F9,
            F10,
            F11,
            F12,
            F13,
            F14,
            F15,
            F16,
            F17,
            F18,
            F19,
            F20,
            F21,
            F22,
            F23,
            F24,
            // Modifiers
            AltLeft,
            AltRight,
            ControlLeft,
            ControlRight,
            ShiftLeft,
            ShiftRight,
            SuperLeft,
            SuperRight,
            // Navigation
            ArrowDown,
            ArrowLeft,
            ArrowRight,
            ArrowUp,
            End,
            Home,
            PageDown,
            PageUp,
            // Editing
            Backspace,
            Delete,
            Enter,
            Escape,
            Insert,
            Space,
            Tab,
            // Numpad
            Numpad0,
            Numpad1,
            Numpad2,
            Numpad3,
            Numpad4,
            Numpad5,
            Numpad6,
            Numpad7,
            Numpad8,
            Numpad9,
            NumpadAdd,
            NumpadDivide,
            NumpadMultiply,
            NumpadSubtract,
            NumpadDecimal,
            NumpadEnter,
            // Media and special
            AudioVolumeDown,
            AudioVolumeMute,
            AudioVolumeUp,
            BrowserBack,
            BrowserForward,
            BrowserHome,
            BrowserRefresh,
            BrowserSearch,
            CapsLock,
            NumLock,
            ScrollLock,
            PrintScreen,
            Pause,
            MediaPlayPause,
            MediaStop,
            MediaTrackNext,
            MediaTrackPrevious,
            // Punctuation and symbols
            Backquote,
            Backslash,
            BracketLeft,
            BracketRight,
            Comma,
            Equal,
            Minus,
            Period,
            Quote,
            Semicolon,
            Slash,
        )
    }

    /// Get the category for this key code
    #[allow(clippy::enum_glob_use)]
    #[must_use]
    pub const fn category(&self) -> &'static str {
        use KeyCodeWrapper::*;
        match self {
            // Letters
            KeyA | KeyB | KeyC | KeyD | KeyE | KeyF | KeyG | KeyH | KeyI | KeyJ | KeyK | KeyL
            | KeyM | KeyN | KeyO | KeyP | KeyQ | KeyR | KeyS | KeyT | KeyU | KeyV | KeyW | KeyX
            | KeyY | KeyZ => "Letters",

            // Digits
            Digit0 | Digit1 | Digit2 | Digit3 | Digit4 | Digit5 | Digit6 | Digit7 | Digit8
            | Digit9 => "Digits",

            // Function keys
            F1 | F2 | F3 | F4 | F5 | F6 | F7 | F8 | F9 | F10 | F11 | F12 | F13 | F14 | F15
            | F16 | F17 | F18 | F19 | F20 | F21 | F22 | F23 | F24 => "Function",

            // Modifiers
            AltLeft | AltRight | ControlLeft | ControlRight | ShiftLeft | ShiftRight
            | SuperLeft | SuperRight => "Modifiers",

            // Navigation
            ArrowDown | ArrowLeft | ArrowRight | ArrowUp | End | Home | PageDown | PageUp => {
                "Navigation"
            }

            // Editing
            Backspace | Delete | Enter | Escape | Insert | Space | Tab => "Editing",

            // Numpad
            Numpad0 | Numpad1 | Numpad2 | Numpad3 | Numpad4 | Numpad5 | Numpad6 | Numpad7
            | Numpad8 | Numpad9 | NumpadAdd | NumpadDivide | NumpadMultiply | NumpadSubtract
            | NumpadDecimal | NumpadEnter => "Numpad",

            // Media and special
            AudioVolumeDown | AudioVolumeMute | AudioVolumeUp | BrowserBack | BrowserForward
            | BrowserHome | BrowserRefresh | BrowserSearch | CapsLock | NumLock | ScrollLock
            | PrintScreen | Pause | MediaPlayPause | MediaStop | MediaTrackNext
            | MediaTrackPrevious => "Special",

            // Punctuation and symbols
            Backquote | Backslash | BracketLeft | BracketRight | Comma | Equal | Minus | Period
            | Quote | Semicolon | Slash => "Punctuation",
        }
    }
}

/// Request structure for `send_keys`
#[derive(Debug, Deserialize)]
pub struct SendKeysRequest {
    /// Array of key codes to send
    pub keys:        Vec<String>,
    /// Duration in milliseconds to hold the keys before releasing
    #[serde(default = "default_duration")]
    pub duration_ms: u32,
}

const fn default_duration() -> u32 {
    DEFAULT_KEY_DURATION_MS
}

/// Response structure for `send_keys`
#[derive(Debug, Serialize, Deserialize)]
pub struct SendKeysResponse {
    /// Whether the operation was successful
    pub success:     bool,
    /// List of keys that were sent
    pub keys_sent:   Vec<String>,
    /// Duration in milliseconds the keys were held
    pub duration_ms: u32,
}

/// Validate key codes and return the parsed key codes
fn validate_keys(keys: &[String]) -> Result<Vec<(String, KeyCode)>, BrpError> {
    let mut validated_keys = Vec::new();

    for key_str in keys {
        match parse_key_code(key_str) {
            Ok(key_code) => {
                validated_keys.push((key_str.clone(), key_code));
            }
            Err(e) => {
                return Err(BrpError {
                    code:    error_codes::INVALID_PARAMS,
                    message: format!("Invalid key code '{key_str}': {e}"),
                    data:    None,
                });
            }
        }
    }

    Ok(validated_keys)
}

/// Create keyboard events from validated key codes
fn create_keyboard_events(
    key_codes: &[KeyCode],
    press: bool,
) -> Vec<bevy::input::keyboard::KeyboardInput> {
    let state = if press {
        ButtonState::Pressed
    } else {
        ButtonState::Released
    };

    key_codes
        .iter()
        .map(|&key_code| bevy::input::keyboard::KeyboardInput {
            state,
            key_code,
            logical_key: bevy::input::keyboard::Key::Unidentified(
                bevy::input::keyboard::NativeKey::Unidentified,
            ),
            window: Entity::PLACEHOLDER,
            repeat: false,
            text: None,
        })
        .collect()
}

/// Handler for `send_keys` requests
///
/// Simulates keyboard input by sending key press/release events
///
/// # Errors
///
/// Returns `BrpError` if:
/// - Request parameters are missing
/// - Request format is invalid
/// - Any key code is invalid or unknown
pub fn send_keys_handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    // Parse the request
    let request: SendKeysRequest = if let Some(params) = params {
        serde_json::from_value(params).map_err(|e| BrpError {
            code:    error_codes::INVALID_PARAMS,
            message: format!("Invalid request format: {e}"),
            data:    None,
        })?
    } else {
        return Err(BrpError {
            code:    error_codes::INVALID_PARAMS,
            message: "Missing request parameters".to_string(),
            data:    None,
        });
    };

    // Validate key codes
    let validated_keys = validate_keys(&request.keys)?;
    let valid_key_strings: Vec<String> = validated_keys.iter().map(|(s, _)| s.clone()).collect();
    let key_codes: Vec<KeyCode> = validated_keys.iter().map(|(_, kc)| *kc).collect();

    // Validate duration doesn't exceed maximum
    if request.duration_ms > MAX_KEY_DURATION_MS {
        return Err(BrpError {
            code:    error_codes::INVALID_PARAMS,
            message: format!(
                "Duration {}ms exceeds maximum allowed duration of {}ms (1 minute)",
                request.duration_ms, MAX_KEY_DURATION_MS
            ),
            data:    None,
        });
    }

    // Always send press events first
    let press_events = create_keyboard_events(&key_codes, true);
    for event in press_events {
        world.send_event(event);
    }

    // Always spawn an entity to handle the timed release
    if !key_codes.is_empty() {
        world.spawn(TimedKeyRelease {
            keys:  key_codes,
            timer: Timer::new(
                Duration::from_millis(u64::from(request.duration_ms)),
                TimerMode::Once,
            ),
        });
    }

    Ok(json!(SendKeysResponse {
        success:     true,
        keys_sent:   valid_key_strings,
        duration_ms: request.duration_ms,
    }))
}

/// Response structure for `list_key_codes`
#[derive(Debug, Serialize, Deserialize)]
pub struct ListKeyCodesResponse {
    /// List of available key codes with their categories
    pub key_codes: Vec<KeyCodeInfo>,
    /// Total number of key codes
    pub total:     usize,
}

/// Information about a key code
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyCodeInfo {
    /// The name of the key code (e.g., "`KeyA`", "`Space`")
    pub name:     String,
    /// The category of the key (e.g., "Letters", "Modifiers")
    pub category: String,
}

/// Handler for `list_key_codes` requests
///
/// Returns all available key codes organized by category
///
/// # Errors
///
/// This function is marked as infallible but returns `BrpResult` for API consistency
#[allow(clippy::unnecessary_wraps)]
pub fn list_key_codes_handler(In(_): In<Option<Value>>, _world: &mut World) -> BrpResult {
    let key_codes: Vec<KeyCodeInfo> = KeyCodeWrapper::iter()
        .map(|key| KeyCodeInfo {
            name:     key.to_string(),
            category: key.category().to_string(),
        })
        .collect();

    let total = key_codes.len();

    Ok(json!(ListKeyCodesResponse { key_codes, total }))
}

/// Parse a string into a `KeyCode`
fn parse_key_code(s: &str) -> Result<KeyCode, String> {
    KeyCodeWrapper::from_str(s)
        .map(KeyCodeWrapper::to_key_code)
        .map_err(|_| format!("Unknown key code: {s}"))
}

/// System that processes timed key releases
#[allow(clippy::needless_pass_by_value)]
pub fn process_timed_key_releases(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TimedKeyRelease)>,
    mut keyboard_events: EventWriter<bevy::input::keyboard::KeyboardInput>,
) {
    for (entity, mut timed_release) in &mut query {
        timed_release.timer.tick(time.delta());

        if timed_release.timer.finished() {
            // Send release events for all keys
            for &key_code in &timed_release.keys {
                let event = bevy::input::keyboard::KeyboardInput {
                    state: ButtonState::Released,
                    key_code,
                    logical_key: bevy::input::keyboard::Key::Unidentified(
                        bevy::input::keyboard::NativeKey::Unidentified,
                    ),
                    window: Entity::PLACEHOLDER,
                    repeat: false,
                    text: None,
                };
                keyboard_events.write(event);
            }

            // Remove the component after releasing
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::app::App;

    use super::*;

    #[test]
    #[allow(clippy::expect_used)]
    fn test_duration_validation_exceeds_maximum() {
        // Create a minimal Bevy app
        let mut app = App::new();

        // Create a request with duration exceeding the maximum
        let params = json!({
            "keys": ["KeyA"],
            "duration_ms": 70_000  // 70 seconds, exceeds 60 second maximum
        });

        // Call the handler
        let result = send_keys_handler(In(Some(params)), app.world_mut());

        // Verify it returns an error
        assert!(result.is_err());

        let error = result.expect_err("Expected an error but got success");
        assert_eq!(error.code, error_codes::INVALID_PARAMS);
        assert!(error.message.contains("exceeds maximum allowed duration"));
        assert!(error.message.contains("60000ms"));
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_duration_validation_within_maximum() {
        // Create a minimal Bevy app
        let mut app = App::new();

        // Create a request with duration within the maximum
        let params = json!({
            "keys": ["KeyA"],
            "duration_ms": 30_000  // 30 seconds, within 60 second maximum
        });

        // Call the handler
        let result = send_keys_handler(In(Some(params)), app.world_mut());

        // Verify it succeeds
        assert!(result.is_ok());

        let response = result.expect("Expected success but got error");
        assert_eq!(response["success"], true);
        assert_eq!(response["duration_ms"], 30_000);
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_default_duration() {
        // Create a minimal Bevy app
        let mut app = App::new();

        // Create a request without specifying duration_ms
        let params = json!({
            "keys": ["KeyA", "KeyB", "Space"]
        });

        // Call the handler
        let result = send_keys_handler(In(Some(params)), app.world_mut());

        // Verify it succeeds
        assert!(result.is_ok());

        let response = result.expect("Expected success but got error");
        assert_eq!(response["success"], true);
        assert_eq!(response["duration_ms"], 100); // Should use default of 100ms
        assert_eq!(response["keys_sent"], json!(["KeyA", "KeyB", "Space"]));
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn test_zero_duration() {
        // Create a minimal Bevy app
        let mut app = App::new();

        // Create a request with zero duration (should be valid)
        let params = json!({
            "keys": ["Enter"],
            "duration_ms": 0
        });

        // Call the handler
        let result = send_keys_handler(In(Some(params)), app.world_mut());

        // Verify it succeeds
        assert!(result.is_ok());

        let response = result.expect("Expected success but got error");
        assert_eq!(response["success"], true);
        assert_eq!(response["duration_ms"], 0);
    }
}
