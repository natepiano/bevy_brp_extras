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
    discover_component_format_with_debug(world, type_name, &mut Vec::new())
}

/// Discover format information for a given component type with debug info
fn discover_component_format_with_debug(
    world: &World,
    type_name: &str,
    debug_info: &mut Vec<String>,
) -> Option<FormatInfo> {
    debug_info.push(format!("Discovering format for type: {type_name}"));
    let type_registry = world.resource::<AppTypeRegistry>();

    // Get type info within a smaller scope to release the registry lock early
    let type_info = {
        let registry = type_registry.read();
        if let Some(registration) = registry.get_with_type_path(type_name) {
            debug_info.push(format!("Found type in registry: {type_name}"));
            registration.type_info().clone()
        } else {
            debug_info.push(format!("Type not found in registry: {type_name}"));
            return None;
        }
    };

    // Generate format info based on the type structure
    debug_info.push(format!(
        "Type kind: {:?}",
        match &type_info {
            TypeInfo::Struct(_) => "Struct",
            TypeInfo::TupleStruct(_) => "TupleStruct",
            TypeInfo::Enum(_) => "Enum",
            TypeInfo::List(_) => "List",
            TypeInfo::Array(_) => "Array",
            TypeInfo::Map(_) => "Map",
            TypeInfo::Tuple(_) => "Tuple",
            _ => "Other",
        }
    ));

    match &type_info {
        TypeInfo::Struct(struct_info) => {
            let spawn_format =
                generate_spawn_format_for_struct_with_debug(struct_info, world, debug_info);
            let mutation_info =
                generate_mutation_info_for_struct_with_debug(struct_info, world, debug_info);

            Some(FormatInfo {
                type_name: type_name.to_string(),
                spawn_format,
                mutation_info,
            })
        }
        TypeInfo::TupleStruct(tuple_struct_info) => {
            let spawn_format = generate_spawn_format_for_tuple_struct_with_debug(
                tuple_struct_info,
                world,
                debug_info,
            );
            let mutation_info = generate_mutation_info_for_tuple_struct_with_debug(
                tuple_struct_info,
                world,
                debug_info,
            );

            Some(FormatInfo {
                type_name: type_name.to_string(),
                spawn_format,
                mutation_info,
            })
        }
        TypeInfo::Enum(enum_info) => {
            let spawn_format = SpawnInfo {
                example:     generate_enum_example_with_debug(enum_info, world, debug_info),
                description: format!("Enum with {} variants", enum_info.variant_len()),
            };
            let mutation_info = MutationInfo {
                fields:      HashMap::new(), // Enums don't have direct mutation paths
                description: "Enum types must be replaced entirely".to_string(),
            };
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
fn generate_spawn_format_for_struct(
    struct_info: &bevy::reflect::StructInfo,
    world: &World,
) -> SpawnInfo {
    generate_spawn_format_for_struct_with_debug(struct_info, world, &mut Vec::new())
}

/// Generate spawn format information for a struct type with debug info
fn generate_spawn_format_for_struct_with_debug(
    struct_info: &bevy::reflect::StructInfo,
    world: &World,
    debug_info: &mut Vec<String>,
) -> SpawnInfo {
    debug_info.push(format!(
        "Generating spawn format for struct: {}",
        struct_info.type_path()
    ));
    let mut example_obj = serde_json::Map::new();

    for field in struct_info.iter() {
        let field_name = field.name();
        debug_info.push(format!(
            "  Field '{}' of type: {}",
            field_name,
            field.type_path()
        ));
        // Use recursive discovery instead of generate_example_value_for_type
        let example_value =
            discover_type_format_recursive_with_debug(world, field.type_path(), debug_info);
        example_obj.insert(field_name.to_string(), example_value);
    }

    SpawnInfo {
        example:     Value::Object(example_obj),
        description: format!("Struct with {} fields", struct_info.field_len()),
    }
}

/// Generate mutation info for a struct type
fn generate_mutation_info_for_struct(
    struct_info: &bevy::reflect::StructInfo,
    world: &World,
) -> MutationInfo {
    generate_mutation_info_for_struct_with_debug(struct_info, world, &mut Vec::new())
}

fn generate_mutation_info_for_struct_with_debug(
    struct_info: &bevy::reflect::StructInfo,
    world: &World,
    debug_info: &mut Vec<String>,
) -> MutationInfo {
    let mut fields = HashMap::new();

    for field in struct_info.iter() {
        let field_name = field.name();
        let path = format!(".{field_name}");
        let example_value =
            discover_type_format_recursive_with_debug(world, field.type_path(), debug_info);

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
    world: &World,
) -> SpawnInfo {
    generate_spawn_format_for_tuple_struct_with_debug(tuple_struct_info, world, &mut Vec::new())
}

/// Generate spawn format information for a tuple struct type with debug info
fn generate_spawn_format_for_tuple_struct_with_debug(
    tuple_struct_info: &bevy::reflect::TupleStructInfo,
    world: &World,
    debug_info: &mut Vec<String>,
) -> SpawnInfo {
    debug_info.push(format!(
        "Generating spawn format for tuple struct: {}",
        tuple_struct_info.type_path()
    ));
    // For newtype structs (single field), return the field value directly
    if tuple_struct_info.field_len() == 1 {
        debug_info.push("  Newtype struct (single field)".to_string());
        if let Some(field) = tuple_struct_info.field_at(0) {
            debug_info.push(format!("  Field type: {}", field.type_path()));
            let example_value =
                discover_type_format_recursive_with_debug(world, field.type_path(), debug_info);
            return SpawnInfo {
                example:     example_value,
                description: format!("Newtype wrapper around {}", field.type_path()),
            };
        }
    }

    // For multi-field tuple structs, use array format
    let mut example_array = Vec::new();
    for field in tuple_struct_info.iter() {
        let example_value =
            discover_type_format_recursive_with_debug(world, field.type_path(), debug_info);
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
    world: &World,
) -> MutationInfo {
    generate_mutation_info_for_tuple_struct_with_debug(tuple_struct_info, world, &mut Vec::new())
}

fn generate_mutation_info_for_tuple_struct_with_debug(
    tuple_struct_info: &bevy::reflect::TupleStructInfo,
    world: &World,
    debug_info: &mut Vec<String>,
) -> MutationInfo {
    let mut fields = HashMap::new();

    for (index, field) in tuple_struct_info.iter().enumerate() {
        let field_key = format!("field_{index}");
        let path = format!(".{index}");
        let example_value =
            discover_type_format_recursive_with_debug(world, field.type_path(), debug_info);

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

/// Recursively discover format for any type using reflection
fn discover_type_format_recursive(world: &World, type_path: &str) -> Value {
    discover_type_format_recursive_with_debug(world, type_path, &mut Vec::new())
}

/// Recursively discover format for any type using reflection with debug info
fn discover_type_format_recursive_with_debug(
    world: &World,
    type_path: &str,
    debug_info: &mut Vec<String>,
) -> Value {
    debug_info.push(format!("Discovering recursive format for: {type_path}"));
    // Try to get type from registry
    let type_registry = world.resource::<AppTypeRegistry>();
    let registry = type_registry.read();

    if let Some(registration) = registry.get_with_type_path(type_path) {
        debug_info.push(format!("Found type in registry: {type_path}"));
        match registration.type_info() {
            TypeInfo::Struct(info) => generate_struct_example_with_debug(info, world, debug_info),
            TypeInfo::TupleStruct(info) => {
                generate_tuple_struct_example_with_debug(info, world, debug_info)
            }
            TypeInfo::Enum(info) => generate_enum_example_with_debug(info, world, debug_info),
            TypeInfo::List(_) | TypeInfo::Array(_) => {
                debug_info.push(format!("Type is List/Array: {type_path}"));
                json!([])
            }
            TypeInfo::Map(_) => {
                debug_info.push(format!("Type is Map: {type_path}"));
                json!({})
            }
            TypeInfo::Tuple(info) => generate_tuple_example_with_debug(info, world, debug_info),
            _ => {
                debug_info.push(format!("Unknown type kind, using primitive: {type_path}"));
                generate_primitive_example(type_path)
            }
        }
    } else {
        debug_info.push(format!(
            "Type not in registry, using primitive example: {type_path}"
        ));
        let result = generate_primitive_example(type_path);
        debug_info.push(format!("Primitive example result: {:?}", result));
        result
    }
}

/// Generate examples for primitive/unregistered types
fn generate_primitive_example(type_path: &str) -> Value {
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
        _ => json!(null),
    }
}

/// Generate example for struct types
fn generate_struct_example(struct_info: &bevy::reflect::StructInfo, world: &World) -> Value {
    generate_struct_example_with_debug(struct_info, world, &mut Vec::new())
}

fn generate_struct_example_with_debug(
    struct_info: &bevy::reflect::StructInfo,
    world: &World,
    debug_info: &mut Vec<String>,
) -> Value {
    let mut example_obj = serde_json::Map::new();

    for field in struct_info.iter() {
        let field_name = field.name();
        let example_value =
            discover_type_format_recursive_with_debug(world, field.type_path(), debug_info);
        example_obj.insert(field_name.to_string(), example_value);
    }

    Value::Object(example_obj)
}

/// Generate example for tuple struct types
fn generate_tuple_struct_example(
    tuple_struct_info: &bevy::reflect::TupleStructInfo,
    world: &World,
) -> Value {
    generate_tuple_struct_example_with_debug(tuple_struct_info, world, &mut Vec::new())
}

fn generate_tuple_struct_example_with_debug(
    tuple_struct_info: &bevy::reflect::TupleStructInfo,
    world: &World,
    debug_info: &mut Vec<String>,
) -> Value {
    // For newtype structs (single field), return the field value directly
    if tuple_struct_info.field_len() == 1 {
        if let Some(field) = tuple_struct_info.field_at(0) {
            return discover_type_format_recursive(world, field.type_path());
        }
    }

    // For multi-field tuple structs, use array format
    let mut example_array = Vec::new();
    for field in tuple_struct_info.iter() {
        let example_value =
            discover_type_format_recursive_with_debug(world, field.type_path(), debug_info);
        example_array.push(example_value);
    }
    Value::Array(example_array)
}

/// Generate example for tuple types
fn generate_tuple_example(tuple_info: &bevy::reflect::TupleInfo, world: &World) -> Value {
    generate_tuple_example_with_debug(tuple_info, world, &mut Vec::new())
}

fn generate_tuple_example_with_debug(
    tuple_info: &bevy::reflect::TupleInfo,
    world: &World,
    debug_info: &mut Vec<String>,
) -> Value {
    let mut example_array = Vec::new();

    for field in tuple_info.iter() {
        let example_value =
            discover_type_format_recursive_with_debug(world, field.type_path(), debug_info);
        example_array.push(example_value);
    }

    Value::Array(example_array)
}

/// Generate example for enum types
fn generate_enum_example(enum_info: &bevy::reflect::EnumInfo, world: &World) -> Value {
    generate_enum_example_with_debug(enum_info, world, &mut Vec::new())
}

fn generate_enum_example_with_debug(
    enum_info: &bevy::reflect::EnumInfo,
    world: &World,
    debug_info: &mut Vec<String>,
) -> Value {
    debug_info.push(format!(
        "Generating enum example for: {}",
        enum_info.type_path()
    ));
    use bevy::reflect::VariantInfo;

    // Pick first variant as example (or could be configurable)
    if let Some(first_variant) = enum_info.iter().next() {
        debug_info.push(format!("  Using first variant: {}", first_variant.name()));
        match first_variant {
            VariantInfo::Unit(v) => {
                debug_info.push(format!("  Unit variant: {}", v.name()));
                json!(v.name())
            }
            VariantInfo::Tuple(v) => {
                debug_info.push(format!("  Tuple variant with {} fields", v.field_len()));
                // For single-field tuple variants, unwrap the value
                if v.field_len() == 1 {
                    if let Some(field) = v.field_at(0) {
                        debug_info.push(format!("    Single field type: {}", field.type_path()));
                        let field_value = discover_type_format_recursive_with_debug(
                            world,
                            field.type_path(),
                            debug_info,
                        );
                        debug_info.push(format!("    Field value: {:?}", field_value));
                        return json!({ v.name(): field_value });
                    }
                }

                // For multi-field tuple variants, use array format
                let fields: Vec<Value> = v
                    .iter()
                    .map(|field| discover_type_format_recursive(world, field.type_path()))
                    .collect();
                json!({ v.name(): fields })
            }
            VariantInfo::Struct(v) => {
                let mut fields = serde_json::Map::new();
                for field in v.iter() {
                    fields.insert(
                        field.name().to_string(),
                        discover_type_format_recursive(world, field.type_path()),
                    );
                }
                json!({ v.name(): fields })
            }
        }
    } else {
        json!({})
    }
}

/// Discover formats for multiple component types
pub fn discover_multiple_formats(
    world: &World,
    type_names: &[String],
) -> HashMap<String, FormatInfo> {
    discover_multiple_formats_with_debug(world, type_names, &mut Vec::new())
}

/// Discover formats for multiple component types with debug info
pub fn discover_multiple_formats_with_debug(
    world: &World,
    type_names: &[String],
    debug_info: &mut Vec<String>,
) -> HashMap<String, FormatInfo> {
    let mut formats = HashMap::new();

    for type_name in type_names {
        debug_info.push(format!("\n=== Discovering format for: {} ===", type_name));
        if let Some(format_info) =
            discover_component_format_with_debug(world, type_name, debug_info)
        {
            formats.insert(type_name.clone(), format_info);
        } else {
            debug_info.push(format!("Failed to discover format for: {}", type_name));
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

    // Check if debug mode is enabled
    let mut debug_info = Vec::new();
    let include_debug = crate::debug_mode::is_debug_enabled();

    // Discover formats for the requested types
    let formats = if include_debug {
        discover_multiple_formats_with_debug(world, &type_names, &mut debug_info)
    } else {
        discover_multiple_formats(world, &type_names)
    };

    // Build response
    let mut response = json!({
        "success": true,
        "formats": formats,
        "requested_types": type_names,
        "discovered_count": formats.len()
    });

    // Add debug info if enabled
    if include_debug && !debug_info.is_empty() {
        response["debug_info"] = json!(debug_info);
    }

    // Return the discovered formats
    Ok(response)
}
