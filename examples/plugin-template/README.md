# Example Plugin Template

This is a complete working example of a dynamic plugin for Native Launcher.

## What It Does

This example plugin:

- Responds to queries starting with `example:`
- Returns sample results with customizable commands
- Demonstrates all plugin API features
- Shows how to handle keyboard events (Ctrl+E)

## Building

```bash
cargo build --release
```

This creates `target/release/libexample_plugin.so`

## Installing

```bash
# Create plugin directory
mkdir -p ~/.config/native-launcher/plugins

# Copy plugin
cp target/release/libexample_plugin.so ~/.config/native-launcher/plugins/

# Restart launcher
```

## Testing

1. Start Native Launcher
2. Type `example:` to see help
3. Type `example: test` to see search results
4. Select a result and press Enter to execute
5. Try Ctrl+E while focused on an `example:` query

## How It Works

The plugin implements the C FFI interface defined by Native Launcher:

```rust
// Required functions
plugin_get_abi_version()     // Returns ABI version (must be 1)
plugin_get_name()            // Plugin name
plugin_get_description()     // Plugin description
plugin_get_priority()        // Search priority (higher = first)
plugin_should_handle()       // Check if plugin handles query
plugin_search()              // Return search results
plugin_handle_keyboard_event()  // Handle keyboard shortcuts
plugin_free_results()        // Free result memory
plugin_free_string()         // Free string memory
```

## Customizing

### Change Trigger Prefix

```rust
pub extern "C" fn plugin_should_handle(query: CStringSlice) -> bool {
    unsafe {
        let query_str = query.to_string();
        query_str.starts_with("myprefix:")  // Change this
    }
}
```

### Modify Results

```rust
pub extern "C" fn plugin_search(...) -> CResultArray {
    // Create custom results
    results.push(CPluginResult {
        title: CStringSlice::from_string("My Custom Result"),
        subtitle: CStringSlice::from_string("My description"),
        icon: CStringSlice::from_string("my-icon-name"),
        command: CStringSlice::from_string("my-command"),
        terminal: false,
        score: 1000,
    });
}
```

### Add Keyboard Shortcuts

```rust
pub extern "C" fn plugin_handle_keyboard_event(event: CKeyboardEvent) -> CKeyboardActionData {
    // Ctrl = 0x04, Shift = 0x01, Alt = 0x08, Super = 0x40
    if event.modifiers & 0x04 != 0 {  // Ctrl pressed
        if event.key_val == 'x' as u32 {
            // Handle Ctrl+X
            return CKeyboardActionData {
                action: CKeyboardAction::Execute,
                data: CStringSlice::from_string("my-command"),
                terminal: false,
            };
        }
    }
    // Don't handle
    CKeyboardActionData {
        action: CKeyboardAction::None,
        data: CStringSlice::empty(),
        terminal: false,
    }
}
```

## Debugging

Enable debug logging:

```bash
RUST_LOG=debug native-launcher
```

Look for plugin loading messages:

```
INFO native_launcher::plugins::dynamic: Loading plugin from: ~/.config/native-launcher/plugins/libexample_plugin.so
INFO native_launcher::plugins::dynamic: Loaded plugin 'Example Plugin' (priority: 200)
```

## Next Steps

1. Copy this template for your own plugin
2. Customize `src/lib.rs` with your functionality
3. Add dependencies to `Cargo.toml` as needed
4. Build and test
5. Share with the community!

## Resources

- **Full Guide**: See [DYNAMIC_PLUGINS.md](../../DYNAMIC_PLUGINS.md)
- **Plugin API**: See [src/plugins/dynamic.rs](../../src/plugins/dynamic.rs)
- **Built-in Plugins**: See [src/plugins/](../../src/plugins/) for reference

## License

MIT - same as Native Launcher
