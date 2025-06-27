# BRP Resource Mutation Issue: ClearColor Analysis

## Problem Summary
The `mcp__brp__bevy_get_resource` tool returns flattened resource data that obscures the actual structure needed for mutation operations, particularly for tuple structs containing enums.

## Tool Usage and Results

### 1. Resource Retrieval
**Tool Used:** `mcp__brp__bevy_get_resource`
**Parameters:**
```json
{
  "resource": "bevy_render::camera::clear_color::ClearColor"
}
```

**Response Received:**
```json
{
  "status": "success",
  "message": "Retrieved resource: bevy_render::camera::clear_color::ClearColor",
  "data": {
    "data": {
      "value": {
        "Srgba": {
          "alpha": 1.0,
          "blue": 0.1843137294054031,
          "green": 0.1725490242242813,
          "red": 0.16862745583057404
        }
      }
    },
    "resource": "bevy_render::camera::clear_color::ClearColor"
  }
}
```

### 2. Mutation Attempts

#### Failed Attempt: Direct Enum Field Access
**Tool Used:** `mcp__brp__bevy_mutate_resource`
**Parameters:**
```json
{
  "resource": "bevy_render::camera::clear_color::ClearColor",
  "path": ".0.Srgba.red",
  "value": 0.5
}
```

**Error Response:**
```json
{
  "status": "error",
  "message": "Error accessing element with `.Srgba` access(offset 3): Expected variant field access to access a Struct variant, found a Tuple variant instead.",
  "data": {
    "error_code": -23501
  }
}
```

#### Successful Workaround: Full Value Replacement
**Tool Used:** `mcp__brp__bevy_mutate_resource`
**Parameters:**
```json
{
  "resource": "bevy_render::camera::clear_color::ClearColor",
  "path": ".0",
  "value": {"Srgba": {"red": 1, "green": 0, "blue": 0, "alpha": 1}}
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Successfully mutated resource: bevy_render::camera::clear_color::ClearColor"
}
```

## Root Cause Analysis

### Actual ClearColor Structure
From Bevy source code (`/Users/natemccoy/rust/bevy/crates/bevy_render/src/camera/clear_color.rs:37`):
```rust
#[derive(Resource, Clone, Debug, Deref, DerefMut, ExtractResource, Reflect)]
pub struct ClearColor(pub Color);
```

This is a **tuple struct** containing a `Color` enum at position `0`.

### The Problem
The `mcp__brp__bevy_get_resource` response flattens the structure by:
1. Omitting the tuple struct wrapper (the `0` field)
2. Presenting the inner `Color` enum directly under a generic `"value"` key
3. Making it unclear that ClearColor is actually `ClearColor(Color)` not `ClearColor { value: Color }`

## Desired Fix

### What `mcp__brp__bevy_get_resource` Should Return
To properly expose the tuple struct nature, the response should be:
```json
{
  "status": "success",
  "message": "Retrieved resource: bevy_render::camera::clear_color::ClearColor",
  "data": {
    "data": {
      "0": {
        "Srgba": {
          "alpha": 1.0,
          "blue": 0.1843137294054031,
          "green": 0.1725490242242813,
          "red": 0.16862745583057404
        }
      }
    },
    "resource": "bevy_render::camera::clear_color::ClearColor"
  }
}
```

### Benefits of This Fix
1. **Clear Structure**: Shows that field `0` contains the Color enum
2. **Correct Mutation Paths**: Makes it obvious that `.0` is needed to access the Color
3. **Consistency**: Matches the actual Rust structure of tuple structs
4. **Debugging**: Easier to understand why `.0.Srgba.red` fails (enum variant traversal limitation)

## Impact
This issue affects any tuple struct resource in Bevy, making it difficult to understand the correct mutation paths without examining the source code. The fix would improve the developer experience for BRP-based tools and debugging.