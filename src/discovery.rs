//! Format discovery engine for BRP component introspection
//!
//! This module provides the core functionality for discovering component formats
//! through Bevy's reflection system and generating format information for BRP operations.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::reflect::TypeInfo;
use bevy::remote::{BrpError, BrpResult, error_codes};
use serde_json::{Value, json};

use crate::format::{FieldInfo, FormatInfo, MutationInfo, SpawnInfo};

/// Discover format information for a given component type
///
/// This function uses Bevy's reflection system to analyze a component type
/// and generate format information that can be used for proper BRP operations.
pub fn discover_component_format(world: &World, type_name: &str) -> Option<FormatInfo> {
    let type_registry = world.resource::<AppTypeRegistry>();

    // Get type info within a smaller scope to release the registry lock early
    let type_info = {
        let registry = type_registry.read();
        registry.get_with_type_path(type_name)?.type_info().clone()
    };

    // Generate format info based on the type structure
    match &type_info {
        TypeInfo::Struct(struct_info) => {
            let spawn_format = generate_spawn_format_for_struct(struct_info);
            let mutation_info = generate_mutation_info_for_struct(struct_info);

            Some(FormatInfo {
                type_name: type_name.to_string(),
                spawn_format,
                mutation_info,
            })
        }
        TypeInfo::TupleStruct(tuple_struct_info) => {
            let spawn_format = generate_spawn_format_for_tuple_struct(tuple_struct_info);
            let mutation_info = generate_mutation_info_for_tuple_struct(tuple_struct_info);

            Some(FormatInfo {
                type_name: type_name.to_string(),
                spawn_format,
                mutation_info,
            })
        }
        _ => {
            // For other types, provide a basic format
            Some(FormatInfo {
                type_name:     type_name.to_string(),
                spawn_format:  SpawnInfo {
                    example:     json!({}),
                    description: format!("Basic format for {type_name}"),
                },
                mutation_info: MutationInfo {
                    fields:      HashMap::new(),
                    description: format!("No mutation fields available for {type_name}"),
                },
            })
        }
    }
}

/// Generate spawn format information for a struct type
fn generate_spawn_format_for_struct(struct_info: &bevy::reflect::StructInfo) -> SpawnInfo {
    let mut example_obj = serde_json::Map::new();

    for field in struct_info.iter() {
        let field_name = field.name();
        let example_value = generate_example_value_for_type(field.type_path());
        example_obj.insert(field_name.to_string(), example_value);
    }

    SpawnInfo {
        example:     Value::Object(example_obj),
        description: format!("Struct with {} fields", struct_info.field_len()),
    }
}

/// Generate mutation info for a struct type
fn generate_mutation_info_for_struct(struct_info: &bevy::reflect::StructInfo) -> MutationInfo {
    let mut fields = HashMap::new();

    for field in struct_info.iter() {
        let field_name = field.name();
        let path = format!(".{field_name}");
        let example_value = generate_example_value_for_type(field.type_path());

        fields.insert(
            field_name.to_string(),
            FieldInfo {
                path,
                value_type: field.type_path().to_string(),
                example: example_value,
                description: format!("Field '{}' of type {}", field_name, field.type_path()),
            },
        );
    }

    MutationInfo {
        fields,
        description: format!("Struct with {} mutable fields", struct_info.field_len()),
    }
}

/// Generate spawn format information for a tuple struct type
fn generate_spawn_format_for_tuple_struct(
    tuple_struct_info: &bevy::reflect::TupleStructInfo,
) -> SpawnInfo {
    let mut example_array = Vec::new();

    for field in tuple_struct_info.iter() {
        let example_value = generate_example_value_for_type(field.type_path());
        example_array.push(example_value);
    }

    SpawnInfo {
        example:     Value::Array(example_array),
        description: format!("Tuple struct with {} fields", tuple_struct_info.field_len()),
    }
}

/// Generate mutation info for a tuple struct type
fn generate_mutation_info_for_tuple_struct(
    tuple_struct_info: &bevy::reflect::TupleStructInfo,
) -> MutationInfo {
    let mut fields = HashMap::new();

    for (index, field) in tuple_struct_info.iter().enumerate() {
        let field_key = format!("field_{index}");
        let path = format!(".{index}");
        let example_value = generate_example_value_for_type(field.type_path());

        fields.insert(
            field_key,
            FieldInfo {
                path,
                value_type: field.type_path().to_string(),
                example: example_value,
                description: format!("Tuple field {} of type {}", index, field.type_path()),
            },
        );
    }

    MutationInfo {
        fields,
        description: format!(
            "Tuple struct with {} mutable fields",
            tuple_struct_info.field_len()
        ),
    }
}

/// Generate example values for common types
fn generate_example_value_for_type(type_path: &str) -> Value {
    match type_path {
        "f32" | "f64" => json!(1.0),
        "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => {
            json!(1)
        }
        "bool" => json!(true),
        "alloc::string::String" | "&str" => json!("example"),
        path if path.contains("Vec2") => json!([1.0, 2.0]),
        path if path.contains("Vec3") => json!([1.0, 2.0, 3.0]),
        path if path.contains("Vec4") => json!([1.0, 2.0, 3.0, 4.0]),
        path if path.contains("Quat") => json!([0.0, 0.0, 0.0, 1.0]),
        path if path.contains("Transform") => json!({
            "translation": [0.0, 0.0, 0.0],
            "rotation": [0.0, 0.0, 0.0, 1.0],
            "scale": [1.0, 1.0, 1.0]
        }),
        path if path.contains("LinearRgba") => json!({
            "red": 1.0,
            "green": 1.0,
            "blue": 1.0,
            "alpha": 1.0
        }),
        path if path.contains("Name") => json!("EntityName"),
        _ => json!(null),
    }
}

/// Discover formats for multiple component types
pub fn discover_multiple_formats(
    world: &World,
    type_names: &[String],
) -> HashMap<String, FormatInfo> {
    let mut formats = HashMap::new();

    for type_name in type_names {
        if let Some(format_info) = discover_component_format(world, type_name) {
            formats.insert(type_name.clone(), format_info);
        }
    }

    formats
}

/// Get common component type names for discovery
#[must_use]
pub fn get_common_component_types() -> Vec<String> {
    vec![
        "bevy_transform::components::transform::Transform".to_string(),
        "bevy_core::name::Name".to_string(),
        "bevy_render::color::LinearRgba".to_string(),
        "bevy_sprite::sprite::Sprite".to_string(),
        "bevy_render::camera::camera::Camera".to_string(),
    ]
}

/// Handler for format discovery requests
///
/// Discovers component format information for use with BRP operations
pub fn handler(In(params): In<Option<Value>>, world: &mut World) -> BrpResult {
    // Parse parameters - types parameter is required
    let type_names = if let Some(params) = params {
        if let Some(types) = params.get("types") {
            // Extract type names from parameters
            match types {
                Value::Array(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(std::string::ToString::to_string)
                    .collect(),
                Value::String(s) => vec![s.clone()],
                _ => {
                    return Err(BrpError {
                        code:    error_codes::INVALID_PARAMS,
                        message: "Parameter 'types' must be a string or array of strings"
                            .to_string(),
                        data:    None,
                    });
                }
            }
        } else {
            return Err(BrpError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing required 'types' parameter. Specify component types to get format information for.".to_string(),
                data: None,
            });
        }
    } else {
        return Err(BrpError {
            code: error_codes::INVALID_PARAMS,
            message: "Missing required 'types' parameter. Specify component types to get format information for.".to_string(),
            data: None,
        });
    };

    // Discover formats for the requested types
    let formats = discover_multiple_formats(world, &type_names);

    // Return the discovered formats
    Ok(json!({
        "success": true,
        "formats": formats,
        "requested_types": type_names,
        "discovered_count": formats.len()
    }))
}
