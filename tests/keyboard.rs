//! Tests for keyboard input simulation

use bevy::prelude::*;
use serde_json::json;

#[cfg(test)]
#[allow(clippy::expect_used)]
mod keyboard_tests {
    use bevy_brp_extras::{KeyCodeWrapper, ListKeyCodesResponse, SendKeysResponse};
    use strum::IntoEnumIterator;

    use super::*;

    /// Test that all key code variants can be parsed
    #[test]
    fn test_parse_all_key_codes() {
        // Test that all keys can be successfully used in a send_keys request
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Iterate over all key code variants
        for key_wrapper in KeyCodeWrapper::iter() {
            let key = key_wrapper.to_string();
            let params = json!({
                "keys": [&key]
            });

            let result = bevy_brp_extras::keyboard::send_keys_handler(
                bevy::ecs::system::In(Some(params)),
                app.world_mut(),
            );

            assert!(result.is_ok(), "Failed to parse key code: {key}");

            if let Ok(response_value) = result {
                let response: SendKeysResponse =
                    serde_json::from_value(response_value).expect("Failed to deserialize response");
                assert!(response.success);
                assert_eq!(response.keys_sent.len(), 1);
                assert_eq!(response.keys_sent[0], key);
                assert_eq!(response.duration_ms, 100); // default duration
            }
        }
    }

    /// Test invalid key codes return appropriate errors
    #[test]
    fn test_invalid_key_codes() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let invalid_keys = vec![
            "InvalidKey",
            "Key1",  // Should be Digit1
            "Ctrl",  // Should be ControlLeft or ControlRight
            "Shift", // Should be ShiftLeft or ShiftRight
            "F25",   // Function keys only go up to F24
            "",
            "key a", // lowercase and space
            "KEY_A", // Wrong format
        ];

        for invalid_key in invalid_keys {
            let params = json!({
                "keys": [invalid_key]
            });

            let result = bevy_brp_extras::keyboard::send_keys_handler(
                bevy::ecs::system::In(Some(params)),
                app.world_mut(),
            );

            assert!(
                result.is_err(),
                "Expected error for invalid key: {invalid_key}"
            );
        }
    }

    /// Test press-hold-release cycle with different durations
    #[test]
    fn test_press_hold_release_cycle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Test with default duration
        let default_params = json!({
            "keys": ["Space", "Enter"]
        });

        let default_result = bevy_brp_extras::keyboard::send_keys_handler(
            bevy::ecs::system::In(Some(default_params)),
            app.world_mut(),
        );

        assert!(default_result.is_ok());
        if let Ok(response_value) = default_result {
            let response: SendKeysResponse =
                serde_json::from_value(response_value).expect("Failed to deserialize response");
            assert_eq!(response.duration_ms, 100); // default duration
            assert_eq!(response.keys_sent.len(), 2);
        }

        // Test with custom duration
        let custom_params = json!({
            "keys": ["Space", "Enter"],
            "duration_ms": 500
        });

        let custom_result = bevy_brp_extras::keyboard::send_keys_handler(
            bevy::ecs::system::In(Some(custom_params)),
            app.world_mut(),
        );

        assert!(custom_result.is_ok());
        if let Ok(response_value) = custom_result {
            let response: SendKeysResponse =
                serde_json::from_value(response_value).expect("Failed to deserialize response");
            assert_eq!(response.duration_ms, 500);
            assert_eq!(response.keys_sent.len(), 2);
        }
    }

    /// Test `list_key_codes` returns all expected keys with correct categories
    #[test]
    fn test_list_key_codes() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let result = bevy_brp_extras::keyboard::list_key_codes_handler(
            bevy::ecs::system::In(None),
            app.world_mut(),
        );

        assert!(result.is_ok());

        if let Ok(response_value) = result {
            let response: ListKeyCodesResponse =
                serde_json::from_value(response_value).expect("Failed to deserialize response");

            // Should have all the key codes
            assert_eq!(response.total, response.key_codes.len());
            assert!(response.total > 100); // We have over 100 key codes

            // Check that categories are properly assigned
            let categories: std::collections::HashSet<&str> = response
                .key_codes
                .iter()
                .map(|info| info.category.as_str())
                .collect();

            assert!(categories.contains(&"Letters"));
            assert!(categories.contains(&"Digits"));
            assert!(categories.contains(&"Function"));
            assert!(categories.contains(&"Modifiers"));
            assert!(categories.contains(&"Navigation"));
            assert!(categories.contains(&"Editing"));
            assert!(categories.contains(&"Numpad"));
            assert!(categories.contains(&"Special"));
            assert!(categories.contains(&"Punctuation"));

            // Check some specific keys have correct categories
            for key_info in &response.key_codes {
                match key_info.name.as_str() {
                    "KeyA" => assert_eq!(key_info.category, "Letters"),
                    "Digit0" => assert_eq!(key_info.category, "Digits"),
                    "F1" => assert_eq!(key_info.category, "Function"),
                    "ShiftLeft" => assert_eq!(key_info.category, "Modifiers"),
                    "Space" => assert_eq!(key_info.category, "Editing"),
                    _ => {}
                }
            }
        }
    }

    /// Test missing parameters returns appropriate error
    #[test]
    fn test_missing_parameters() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let result = bevy_brp_extras::keyboard::send_keys_handler(
            bevy::ecs::system::In(None),
            app.world_mut(),
        );

        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.message, "Missing request parameters");
        }
    }

    /// Test empty key array
    #[test]
    fn test_empty_keys() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let params = json!({
            "keys": []
        });

        let result = bevy_brp_extras::keyboard::send_keys_handler(
            bevy::ecs::system::In(Some(params)),
            app.world_mut(),
        );

        assert!(result.is_ok());
        if let Ok(response_value) = result {
            let response: SendKeysResponse =
                serde_json::from_value(response_value).expect("Failed to deserialize response");
            assert_eq!(response.keys_sent.len(), 0);
        }
    }

    /// Test that `TimedKeyRelease` component is always created
    #[test]
    fn test_timed_release_always_created() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Test with custom duration
        let params = json!({
            "keys": ["Space", "Enter"],
            "duration_ms": 500
        });

        let result = bevy_brp_extras::keyboard::send_keys_handler(
            bevy::ecs::system::In(Some(params)),
            app.world_mut(),
        );

        assert!(result.is_ok());

        // Check that a TimedKeyRelease component was created
        let mut query = app.world_mut().query::<&bevy_brp_extras::TimedKeyRelease>();
        let count = query.iter(app.world()).count();
        assert_eq!(count, 1, "Expected one TimedKeyRelease component");

        // Verify the component has the correct keys
        if let Some(timed_release) = query.iter(app.world()).next() {
            assert_eq!(timed_release.keys.len(), 2);
        }
    }

    /// Test default duration creates `TimedKeyRelease` component
    #[test]
    fn test_default_duration_creates_timed_release() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Test with default duration (no duration_ms specified)
        let params = json!({
            "keys": ["Space"]
        });

        let result = bevy_brp_extras::keyboard::send_keys_handler(
            bevy::ecs::system::In(Some(params)),
            app.world_mut(),
        );

        assert!(result.is_ok());

        // Check that a TimedKeyRelease component was created with default duration
        let mut query = app.world_mut().query::<&bevy_brp_extras::TimedKeyRelease>();
        let count = query.iter(app.world()).count();
        assert_eq!(
            count, 1,
            "Expected one TimedKeyRelease component with default duration"
        );
    }

    /// Test that empty key array does not create `TimedKeyRelease`
    #[test]
    fn test_empty_keys_no_timed_release() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Test with empty keys array
        let params = json!({
            "keys": [],
            "duration_ms": 500
        });

        let result = bevy_brp_extras::keyboard::send_keys_handler(
            bevy::ecs::system::In(Some(params)),
            app.world_mut(),
        );

        assert!(result.is_ok());

        // Check that no TimedKeyRelease component was created
        let mut query = app.world_mut().query::<&bevy_brp_extras::TimedKeyRelease>();
        let count = query.iter(app.world()).count();
        assert_eq!(
            count, 0,
            "Expected no TimedKeyRelease components when keys array is empty"
        );
    }
}
