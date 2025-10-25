# Plugin Development Guide

## Overview

Native Launcher supports **script-based plugins** that allow you to extend functionality without writing Rust code. Plugins can be written in any language (Bash, Python, Node.js, etc.) and integrate seamlessly with the launcher.

## Table of Contents

- [Quick Start](#quick-start)
- [Plugin Structure](#plugin-structure)
- [Manifest Format](#manifest-format)
- [Output Formats](#output-formats)
- [Best Practices](#best-practices)
- [Testing & Debugging](#testing--debugging)
- [Example Plugins](#example-plugins)
- [Distribution](#distribution)

## Quick Start

### 1. Create Plugin Directory

```bash
mkdir -p ~/.config/native-launcher/plugins/my-plugin
cd ~/.config/native-launcher/plugins/my-plugin
```

### 2. Create Manifest (plugin.toml)

```toml
[metadata]
name = "My Plugin"
description = "A sample plugin"
author = "Your Name"
version = "1.0.0"
priority = 500  # Higher = searched first (0-1000)

# What the user types to trigger this plugin
triggers = ["myplugin ", "mp "]

[execution]
script = "main.sh"          # Script filename (relative to plugin dir)
interpreter = "bash"        # Optional: bash, python3, node, etc.
output_format = "json"      # "json" or "text"
timeout_ms = 3000          # Max execution time
show_on_empty = false      # Show results when query is empty?
```

### 3. Create Script (main.sh)

```bash
#!/usr/bin/env bash

QUERY="$1"

# Generate JSON output
cat <<EOF
{
  "results": [
    {
      "title": "Result for: $QUERY",
      "subtitle": "Press Enter to execute",
      "command": "echo 'Hello from my plugin!' | wl-copy",
      "icon": "dialog-information"
    }
  ]
}
EOF
```

### 4. Make Script Executable

```bash
chmod +x main.sh
```

### 5. Test Plugin

Restart Native Launcher and type your trigger (e.g., `myplugin test`).

---

## Plugin Structure

```
~/.config/native-launcher/plugins/
└── my-plugin/
    ├── plugin.toml        # Manifest (required)
    ├── main.sh            # Main script
    ├── icon.svg           # Optional icon
    └── lib/               # Optional dependencies
        └── helper.sh
```

**Plugin Directories Scanned:**

1. `~/.config/native-launcher/plugins/` (user plugins)
2. `/usr/share/native-launcher/plugins/` (system plugins)
3. `./plugins/` (development - current directory)

---

## Manifest Format

### [metadata] Section

| Field         | Type    | Required | Description                              |
| ------------- | ------- | -------- | ---------------------------------------- |
| `name`        | String  | ✅       | Display name of plugin                   |
| `description` | String  | ✅       | Short description                        |
| `author`      | String  | ✅       | Author name/email                        |
| `version`     | String  | ✅       | Semantic version (e.g., "1.0.0")         |
| `priority`    | Integer | ✅       | Search priority (0-1000, higher = first) |
| `icon`        | String  | ❌       | Icon name or path                        |

**Priority Guidelines:**

- **900-1000**: Critical/system plugins
- **800-899**: Built-in advanced plugins (Advanced Calculator: 850)
- **700-799**: Built-in core plugins (SSH: 750, Files: 700)
- **600-699**: User plugins (high priority)
- **500-599**: User plugins (normal priority)
- **400-499**: User plugins (low priority)
- **0-399**: Experimental/debug plugins

### triggers Array

List of command prefixes that activate your plugin. Must include trailing space for word-based triggers.

```toml
# Examples:
triggers = ["weather ", "w "]      # Triggered by "weather Tokyo" or "w Tokyo"
triggers = ["calc ", "="]          # Triggered by "calc 2+2" or "=2+2"
triggers = ["#"]                   # Triggered by "#FF5733"
triggers = [":"]                   # Triggered by ":smile"
```

### [execution] Section

| Field           | Type    | Required | Default  | Description                                 |
| --------------- | ------- | -------- | -------- | ------------------------------------------- |
| `script`        | String  | ✅       | -        | Script filename (relative or absolute)      |
| `interpreter`   | String  | ❌       | None     | Command to run script (bash, python3, node) |
| `output_format` | String  | ❌       | `"json"` | Output format: "json" or "text"             |
| `timeout_ms`    | Integer | ❌       | `3000`   | Max execution time (milliseconds)           |
| `show_on_empty` | Boolean | ❌       | `false`  | Show results when query is empty            |

**Interpreter Examples:**

```toml
interpreter = "bash"        # Bash script
interpreter = "python3"     # Python script
interpreter = "node"        # Node.js script
interpreter = "ruby"        # Ruby script
# Or omit for executable scripts with shebang
```

### [environment] Section (Optional)

Set environment variables for your script:

```toml
[environment]
API_KEY = "your-api-key-here"
BASE_URL = "https://api.example.com"
DEBUG = "true"
```

Access in Bash:

```bash
echo "API key: $API_KEY"
```

Access in Python:

```python
import os
api_key = os.environ.get('API_KEY')
```

---

## Output Formats

### JSON Format (Recommended)

**Structure:**

```json
{
  "results": [
    {
      "title": "Result Title",
      "subtitle": "Optional subtitle or description",
      "command": "shell command to execute",
      "icon": "optional-icon-name"
    }
  ]
}
```

**Example (Python):**

```python
#!/usr/bin/env python3
import json
import sys

query = sys.argv[1] if len(sys.argv) > 1 else ""

output = {
    "results": [
        {
            "title": f"Search for: {query}",
            "subtitle": "Press Enter to copy",
            "command": f"echo '{query}' | wl-copy && notify-send 'Copied' '{query}'"
        }
    ]
}

print(json.dumps(output, indent=2))
```

**Example (Bash):**

```bash
#!/usr/bin/env bash
QUERY="$1"

cat <<EOF
{
  "results": [
    {
      "title": "Result: $QUERY",
      "subtitle": "Click to execute",
      "command": "notify-send 'Hello' 'World'"
    }
  ]
}
EOF
```

### Text Format (Simple)

For simple plugins, use text format with pipe-separated values:

**Format:** `title|subtitle|command`

```bash
#!/usr/bin/env bash
echo "First Result|Description|echo 'command1'"
echo "Second Result|Description|echo 'command2'"
echo "Simple Result"  # Title only (command = title)
```

**In manifest:**

```toml
[execution]
output_format = "text"
```

---

## Best Practices

### Performance

1. **Keep scripts fast** - Target <100ms execution time
2. **Cache data** - Don't fetch remote data on every keystroke
3. **Limit results** - Return max 10-20 results
4. **Use timeouts** - Set reasonable `timeout_ms` in manifest
5. **Async where possible** - Use background jobs for slow operations

### User Experience

1. **Clear titles** - Make result titles descriptive
2. **Helpful subtitles** - Explain what will happen on Enter
3. **Clipboard integration** - Use `wl-copy` for copy actions
4. **Desktop notifications** - Use `notify-send` for feedback
5. **Empty query handling** - Show helpful hints when query is empty

### Error Handling

1. **Validate input** - Check query format before processing
2. **Handle failures gracefully** - Return error message as result
3. **Log errors to stderr** - Will appear in launcher logs
4. **Don't exit with error** - Return error as a result instead

**Example Error Handling:**

```bash
#!/usr/bin/env bash
QUERY="$1"

if [ -z "$QUERY" ]; then
    cat <<EOF
{
  "results": [
    {
      "title": "Enter a query...",
      "subtitle": "Example: mycommand something",
      "command": "echo 'No query'"
    }
  ]
}
EOF
    exit 0
fi

# Try to fetch data
DATA=$(curl -s "https://api.example.com/$QUERY" 2>/dev/null)

if [ $? -ne 0 ] || [ -z "$DATA" ]; then
    cat <<EOF
{
  "results": [
    {
      "title": "API Error",
      "subtitle": "Could not fetch data for '$QUERY'",
      "command": "echo 'Error'"
    }
  ]
}
EOF
    exit 0
fi

# Process data...
```

### Security

1. **Sanitize input** - Never directly execute user input
2. **Use quotes** - Always quote variables in shell commands
3. **Validate commands** - Check command paths exist
4. **Avoid eval** - Don't use `eval` on user input
5. **Check permissions** - Verify file/command permissions

**Bad (Command Injection):**

```bash
command = "$1"  # User types: "; rm -rf /"
```

**Good (Quoted & Validated):**

```bash
QUERY="$1"
command="echo 'Safe: $QUERY' | wl-copy"
```

### Clipboard Integration

Always use `wl-copy` for Wayland clipboard:

```bash
# Copy text
echo "Hello World" | wl-copy

# Copy with notification
echo "Result" | wl-copy && notify-send 'Copied' 'Result'

# Copy without newline
echo -n "No newline" | wl-copy
```

**In JSON:**

```json
{
  "command": "echo -n 'Copy me' | wl-copy && notify-send 'Copied' 'Success'"
}
```

---

## Testing & Debugging

### Manual Testing

```bash
# Navigate to plugin directory
cd ~/.config/native-launcher/plugins/my-plugin

# Run script directly
./main.sh "test query"

# Test with interpreter
bash main.sh "test query"
python3 main.py "test query"
```

### Enable Debug Logging

```bash
# Run launcher with debug logs
RUST_LOG=debug native-launcher
```

Check logs in `~/.local/share/native-launcher/launcher.log` or stderr.

### Common Issues

**Issue: Plugin not loading**

- Check manifest syntax: `toml-cli ~/.config/native-launcher/plugins/my-plugin/plugin.toml`
- Verify script path exists
- Check file permissions: `chmod +x script.sh`
- Check logs: `RUST_LOG=debug native-launcher 2>&1 | grep -i plugin`

**Issue: Script not executing**

- Test script directly: `./script.sh "query"`
- Check shebang line: `#!/usr/bin/env bash`
- Verify interpreter path: `which python3`
- Check timeout: Increase `timeout_ms`

**Issue: No results appearing**

- Verify JSON syntax: `./script.sh "test" | jq`
- Check output format matches manifest
- Test trigger: Make sure query starts with trigger
- Check priority: Higher priority plugins searched first

**Issue: Command not executing**

- Test command in terminal first
- Check for typos in `command` field
- Ensure dependencies installed (wl-copy, notify-send)
- Check command permissions

---

## Example Plugins

### 1. Weather Plugin

**Use case:** `weather Tokyo` or `w London`

See `examples/plugins/weather/` for complete implementation.

**Features:**

- Fetches weather from wttr.in API
- Shows temperature, humidity, wind
- Copy weather data to clipboard
- Open detailed forecast in browser
- ASCII art weather display in terminal

### 2. Emoji Search

**Use case:** `emoji smile` or `em heart` or `:fire`

See `examples/plugins/emoji/` for complete implementation.

**Features:**

- 200+ emoji database with keywords
- Fuzzy keyword matching
- One-click copy to clipboard
- Desktop notification on copy
- Popular emojis shown on empty query

### 3. Color Picker

**Use case:** `color #FF5733` or `col rgb(255,87,51)` or `#FF5733`

See `examples/plugins/color/` for complete implementation.

**Features:**

- Converts between hex, RGB, HSL
- CSS variable format
- Tailwind CSS hints
- Color preview (block characters)
- Instant clipboard copy

---

## Distribution

### Sharing Your Plugin

1. **GitHub Repository:**

```
my-plugin/
├── README.md
├── plugin.toml
├── script.py
└── LICENSE
```

2. **Installation Instructions:**

```bash
# Clone to plugins directory
git clone https://github.com/user/my-plugin \
  ~/.config/native-launcher/plugins/my-plugin

# Or download release
wget https://github.com/user/my-plugin/releases/latest/download/my-plugin.tar.gz
tar -xzf my-plugin.tar.gz -C ~/.config/native-launcher/plugins/
```

3. **Dependencies:**
   Document required system packages:

```markdown
## Dependencies

- `curl` - For API requests
- `jq` - For JSON parsing
- `wl-clipboard` - For clipboard support
```

### Plugin Marketplace (Coming Soon)

We're planning a community plugin repository where users can:

- Browse available plugins
- One-command installation
- Auto-update plugins
- Rate and review plugins

---

## API Reference

### Command Line Arguments

Your script receives the user's query as the first argument:

```bash
# User types: "myplugin hello world"
# Script receives: "hello world"

QUERY="$1"
```

### Environment Variables

Available to all plugins:

- `HOME` - User's home directory
- `USER` - Current username
- `XDG_CONFIG_HOME` - Config directory (usually ~/.config)
- Plus any variables defined in `[environment]` section

### Result Commands

Commands are executed using `/bin/sh -c "command"`, so you can use:

- **Pipes:** `echo 'text' | wl-copy`
- **Chaining:** `command1 && command2`
- **Redirection:** `echo 'log' >> /tmp/log.txt`
- **Background:** `long-command &`

**Examples:**

```json
{
  "command": "echo 'Multi' | wl-copy && notify-send 'Done'"
}
{
  "command": "xdg-open 'https://example.com'"
}
{
  "command": "alacritty -e bash -c 'echo Hello; read'"
}
```

### Icons

Use icon names from your system icon theme:

```toml
[metadata]
icon = "weather-clear"       # System icon
icon = "/path/to/icon.svg"   # Custom icon (absolute path)
```

Common icon names:

- `dialog-information`
- `dialog-warning`
- `dialog-error`
- `weather-clear`
- `folder`
- `text-x-generic`
- `emblem-favorite`

---

## Advanced Topics

### Multi-Step Workflows

Return results that trigger other actions:

```json
{
  "results": [
    {
      "title": "Option 1: Copy",
      "command": "echo 'data' | wl-copy"
    },
    {
      "title": "Option 2: Open File",
      "command": "xdg-open /path/to/file"
    },
    {
      "title": "Option 3: Run Command",
      "command": "alacritty -e vim /path/to/file"
    }
  ]
}
```

### Persistent State

Use files to store plugin state:

```bash
#!/usr/bin/env bash
STATE_FILE="$HOME/.cache/my-plugin/state.json"
mkdir -p "$(dirname "$STATE_FILE")"

# Read state
if [ -f "$STATE_FILE" ]; then
    STATE=$(cat "$STATE_FILE")
else
    STATE='{}'
fi

# Update state
echo '{"last_query": "'"$1"'"}' > "$STATE_FILE"
```

### Background Updates

Use systemd user timer for periodic updates:

```ini
# ~/.config/systemd/user/my-plugin-update.timer
[Unit]
Description=My Plugin Update Timer

[Timer]
OnBootSec=5min
OnUnitActiveSec=1h

[Install]
WantedBy=timers.target
```

```ini
# ~/.config/systemd/user/my-plugin-update.service
[Unit]
Description=My Plugin Update

[Service]
Type=oneshot
ExecStart=/home/user/.config/native-launcher/plugins/my-plugin/update.sh
```

Enable:

```bash
systemctl --user enable --now my-plugin-update.timer
```

---

## Plugin Ideas

### Suggested Plugins to Build

1. **Dictionary** - Define words using dict.org
2. **Unit Converter** - Advanced conversions (cooking, data, etc.)
3. **Cryptocurrency** - Live crypto prices
4. **Stock Ticker** - Stock quotes
5. **Translation** - Translate text (Google Translate API)
6. **YouTube Search** - Search and open videos
7. **GitHub** - Search repos, issues, PRs
8. **StackOverflow** - Search questions
9. **Password Generator** - Generate secure passwords
10. **QR Code** - Generate QR codes
11. **Base64** - Encode/decode base64
12. **Hash Calculator** - MD5, SHA256, etc.
13. **IP Lookup** - Get IP info, geolocation
14. **Docker** - Manage containers
15. **System Info** - CPU, RAM, disk usage
16. **Process Killer** - Search and kill processes
17. **Clipboard History** - Browse clipboard history
18. **Snippet Manager** - Store and paste snippets
19. **Bookmark Manager** - Browser bookmarks
20. **Calendar Events** - Quick event creation

---

## Support

- **Documentation:** [Wiki](https://github.com/ArunPrakashG/native-launcher/wiki)
- **Issues:** [GitHub Issues](https://github.com/ArunPrakashG/native-launcher/issues)
- **Discussions:** [GitHub Discussions](https://github.com/ArunPrakashG/native-launcher/discussions)

---

## License

Plugin Development Guide is part of Native Launcher.
Licensed under MIT License.

**Your plugins can use any license** - they are independent projects.
