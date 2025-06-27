# BRP Testing Instructions

## Overview
When following this command, you should:
1. Read all test scenarios in the `<TestScenarios>` section below
2. Create a todo list with each test scenario as a separate todo item
3. For each todo item, follow the `<ExecuteTest>` template
4. At the end, follow the `<EndOfTest>` template

<TestScenarios>

- Test all MCP tools starting with "mcp__brp__"
- Use bevy_brp_extras extras_test_example.rs example (has bevy_brp_extras - Tier 2 validation) or nateroids (no bevy_brp_extras - Tier 3/4 fallback validation) for testing
- When switching between test apps, you need to shutdown the previous app before starting the next one so that you can connect to it because they all use the same tcp port
- Follow test phases in order:

  1. **Discovery** - list_bevy_apps, list_bevy_examples
  2. **Launch** - Launch one app, test brp_status, test workspace disambiguation for duplicate apps
  3. **Introspection** - rpc_discover, registry_schema, list_components/resources
  4. **Data Operations** - Entity/component/resource CRUD operations
  5. **Advanced** - Watches, logging, raw execution

  **Workspace Disambiguation Tests** (during Launch phase):
  - Test `mcp__brp__brp_launch_bevy_app` with app_name "hana" (no workspace) - should error with multiple workspaces found
  - Verify error lists available workspaces (e.g., "hana" and "hana-brp-extras-2-1")
  - Test with workspace parameter: `{"app_name": "hana", "workspace": "hana-brp-extras-2-1"}` - should launch correct app
  - Test `mcp__brp__brp_launch_bevy_example` with duplicate example names - should show similar workspace error
  - Test with workspace parameter for examples - should launch from correct workspace
  - Verify launched app's response includes "workspace" field showing which workspace was used

  **Debug Mode Tests**:
  - Test `mcp__brp__brp_set_debug_mode` with enabled=true - should enable both brp_mcp and brp_extras debug
  - Perform any brp_extras operation (e.g., discover_format) - verify response contains:
    - `brp_mcp_debug_info`: Array of MCP-layer debug messages (parameter extraction, method resolution, BRP communication)
    - `brp_extras_debug_info`: Array of Bevy-layer debug messages (type discovery, field analysis)
  - Test `mcp__brp__brp_set_debug_mode` with enabled=false - should disable both debug modes
  - Test `mcp__brp__brp_extras_set_debug_mode` with enabled=true (while brp_mcp debug is off)
  - Perform same brp_extras operation - verify response contains:
    - Only `brp_extras_debug_info`: Array of Bevy-layer debug messages
    - No `brp_mcp_debug_info` field present
  - Verify debug mode separation works correctly with clean response structure (no nested duplication)

  **Enhanced Tier-Based Format Discovery Tests**:

  **Tier 2 Direct Discovery (with bevy_brp_extras extras_test_example.rs example)**:
  - Test spawning with intentionally wrong formats - should use Tier 2 direct discovery
  - ClearColor with LinearRgba in the wrong format: `[0.8, 0.2, 0.1, 1.0]` (should correct to object format)
  - Transform wrong format: `[1.0, 2.0, 3.0]` (should correct to full Transform structure)
  - Verify format_corrections field shows "Direct discovery from bevy_brp_extras" hints
  - Verify brp_mcp_debug_info shows "SUCCESS Tier 2: Direct Discovery" messages

  **Tier 3/4 Fallback Discovery (with nateroids - no bevy_brp_extras)**:
  - Test spawning with intentionally wrong formats - should fall back to pattern matching
  - LinearRgba: `{"red": 0.8, "green": 0.2, "blue": 0.1, "alpha": 1.0}`
  - Transform: `{"translation": {"x": 1.0, "y": 2.0, "z": 3.0}, "rotation": {"x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0}, "scale": {"x": 1.0, "y": 1.0, "z": 1.0}}`
  - Name: `{"name": "TestEntity"}`
  - Verify brp_mcp_debug_info shows "FAILED Tier 2: Direct Discovery" then "SUCCESS Tier 3/4"
  - **Path Error Discovery** - ClearColor mutation with intentionally wrong path `.0.red` (should suggest `.0.0.red`)

  **BRP Extras Method Availability Tests**:

  **With bevy_brp_extras (extras_test_example.rs example)**:
  - Test `mcp__brp__brp_extras_discover_format` with types array - should work normally and return format information
  - Test `mcp__brp__brp_extras_screenshot` with valid path - should work normally and save screenshot - make sure to read the saved screenshot and that it worked
  - Test `mcp__brp__brp_extras_shutdown` - should work normally with clean shutdown

  **BRP Extras Keyboard Tests (with extras_test_example.rs example)**:
  - Test `mcp__brp__brp_extras_send_keys` with default duration: `{"keys": ["KeyA", "Space"]}` - should hold for 100ms default
  - Test with custom duration: `{"keys": ["KeyH", "KeyI"], "duration_ms": 2000}` - should hold for 2 seconds
  - Test modifier combinations: `{"keys": ["ControlLeft", "KeyA"], "duration_ms": 500}`
  - Test short duration: `{"keys": ["KeyB"], "duration_ms": 50}` - quick tap
  - Test zero duration: `{"keys": ["KeyC"], "duration_ms": 0}` - instant press/release
  - Test maximum duration boundary: `{"keys": ["KeyD"], "duration_ms": 60000}` - 1 minute max
  - Test exceeding max duration: `{"keys": ["KeyE"], "duration_ms": 70000}` - should error
  - Test `mcp__brp__brp_extras_list_key_codes` - should return all available key codes with categories
  - Verify extras_test_example.rs displays pressed keys, modifiers, and duration on screen
  - Test invalid key code - should return error with invalid key name
  - Use `mcp__brp__brp_extras_screenshot` to capture the UI and verify keyboard state matches sent keys

  **Without bevy_brp_extras (nateroids)**:
  - Test `mcp__brp__brp_extras_discover_format` - should return helpful error about adding bevy_brp_extras with installation instructions
  - Test `mcp__brp__brp_extras_screenshot` - should return helpful error with bevy_brp_extras installation instructions
  - Test `mcp__brp__brp_extras_shutdown` - should fall back to process kill with warning message about adding bevy_brp_extras for clean shutdown
  - Verify all brp_extras tools give consistent error guidance when bevy_brp_extras is missing
  - Verify error messages include both the problem explanation and solution steps

  **Registry Discovery Tests** (spawn/insert components without Serialize/Deserialize):
  - Try spawning with Visibility component - should fail with registry diagnostic
  - Try inserting Aabb component - should fail with "lacks Serialize and Deserialize traits"
  - Verify error includes helpful message about BRP registration requirements
  - both of these should work for mutate however so please check that

  **Large Response Test**: registry schema returns too much data to fit in context limit imposed by claude. Validate that a file is written and the path returned so that you can look in the file for the results.

  **Watch Command Tests**:
  - Start `mcp__brp__bevy_get_watch` on an entity with components that change (e.g., Transform on a moving entity)
  - Verify watch returns watch_id and log_path
  - Use `mcp__brp__brp_read_log` to check log file contains COMPONENT_UPDATE entries
  - Start `mcp__brp__bevy_list_watch` on same entity to monitor component additions/removals
  - Verify different watch_id and log_path for list watch
  - Use `mcp__brp__brp_list_active_watches` to see both active watches
  - Add/remove components to trigger list watch logging
  - Read both log files to verify different watch types capture different events
  - Use `mcp__brp__brp_stop_watch` to stop both watches using their watch_ids
  - Verify `mcp__brp__brp_list_active_watches` shows no active watches after stopping

</TestScenarios>

**Issues requiring STOP**: Tool errors, parameter confusion, unexpected behavior
**Not issues**: Component or Resource format corrections (expected), documented limitations

<ExecuteTest>
- Execute test and explain results
- If ANY issue: STOP, report exact error, wait for guidance
- If no issues: continue to next test
</ExecuteTest>

<EndOfTest>
- Shutdown apps/examples
- Summarize results
</EndOfTest>
