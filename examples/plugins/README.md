# Example Plugins for Native Launcher

This directory contains example script-based plugins that demonstrate how to extend Native Launcher functionality without writing Rust code.

## Available Example Plugins

### 🌤️ Weather Plugin

**Command:** `weather <location>` or `w <location>`

Fetches current weather information using the wttr.in API.

**Features:**

- Current conditions (temperature, humidity, wind)
- Copy weather data to clipboard
- Open detailed forecast in browser
- ASCII art weather display in terminal

**Location:** `weather/`

**Usage:**

```
weather Tokyo
w London
weather "New York"
```

**Installation:**

```bash
cp -r weather ~/.config/native-launcher/plugins/
```

---

### 😀 Emoji Search

**Command:** `emoji <keyword>` or `em <keyword>` or `:<keyword>`

Search through 200+ emojis by keyword and copy to clipboard.

**Features:**

- Comprehensive emoji database with keywords
- Fuzzy keyword matching
- One-click copy to clipboard
- Desktop notification on copy
- Popular emojis on empty query

**Location:** `emoji/`

**Usage:**

```
emoji smile
em heart
:fire
:rocket
```

**Installation:**

```bash
cp -r emoji ~/.config/native-launcher/plugins/
```

---

### 🎨 Color Picker

**Command:** `color <value>` or `col <value>` or `#<hex>`

Convert colors between hex, RGB, and HSL formats.

**Features:**

- Hex to RGB/HSL conversion
- RGB to Hex/HSL conversion
- HSL to Hex/RGB conversion
- CSS variable format
- Tailwind CSS hints
- Color preview with block characters

**Location:** `color/`

**Usage:**

```
color #FF5733
col rgb(255, 87, 51)
#FF5733
color hsl(9, 100%, 60%)
```

**Installation:**

```bash
cp -r color ~/.config/native-launcher/plugins/
```

---

## Plugin Development

Want to create your own plugin? See the comprehensive guide:

**[Plugin Development Guide](../docs/PLUGIN_DEVELOPMENT.md)**

### Quick Start

1. **Create plugin directory:**

```bash
mkdir -p ~/.config/native-launcher/plugins/my-plugin
cd ~/.config/native-launcher/plugins/my-plugin
```

2. **Create manifest (plugin.toml):**

```toml
[metadata]
name = "My Plugin"
description = "Does something cool"
author = "Your Name"
version = "1.0.0"
priority = 600

triggers = ["myplugin ", "mp "]

[execution]
script = "main.sh"
interpreter = "bash"
output_format = "json"
timeout_ms = 3000
```

3. **Create script (main.sh):**

```bash
#!/usr/bin/env bash

cat <<EOF
{
  "results": [
    {
      "title": "Result: $1",
      "subtitle": "Press Enter to copy",
      "command": "echo '$1' | wl-copy"
    }
  ]
}
EOF
```

4. **Make executable:**

```bash
chmod +x main.sh
```

5. **Test:**

```bash
./main.sh "test query"
```

---

## Plugin Structure

Each plugin directory should contain:

```
my-plugin/
├── plugin.toml      # Manifest (required)
├── main.sh          # Main script (or main.py, index.js, etc.)
├── README.md        # Documentation
├── icon.svg         # Optional icon
└── lib/             # Optional dependencies
    └── helper.sh
```

---

## Output Formats

### JSON Format (Recommended)

```json
{
  "results": [
    {
      "title": "Result Title",
      "subtitle": "Optional description",
      "command": "shell command to execute",
      "icon": "optional-icon-name"
    }
  ]
}
```

### Text Format

```
Title|Subtitle|Command
Simple Title
Another Title|With Subtitle|echo 'command'
```

---

## Dependencies

Most example plugins require:

### System Packages

- **wl-clipboard** - Clipboard support (wl-copy command)

  ```bash
  # Arch Linux
  sudo pacman -S wl-clipboard

  # Ubuntu/Debian
  sudo apt install wl-clipboard

  # Fedora
  sudo dnf install wl-clipboard
  ```

- **libnotify** - Desktop notifications (notify-send command)

  ```bash
  # Arch Linux
  sudo pacman -S libnotify

  # Ubuntu/Debian
  sudo apt install libnotify-bin

  # Fedora
  sudo dnf install libnotify
  ```

### Plugin-Specific Dependencies

**Weather Plugin:**

- `curl` - For API requests
- `alacritty` (optional) - For ASCII weather display

**Emoji Plugin:**

- `python3` - Python interpreter

**Color Plugin:**

- `python3` - Python interpreter
- No additional packages needed (uses standard library)

---

## Testing Plugins

### Test Plugin Script Directly

```bash
cd ~/.config/native-launcher/plugins/weather
./weather.sh "Tokyo"
```

### Test with Launcher (Debug Mode)

```bash
RUST_LOG=debug native-launcher
```

Then type your trigger command and watch the logs.

### Validate JSON Output

```bash
./weather.sh "Tokyo" | jq
```

---

## Troubleshooting

### Plugin Not Loading

1. **Check manifest syntax:**

```bash
# Validate TOML syntax
cat plugin.toml
```

2. **Check script permissions:**

```bash
chmod +x *.sh *.py
```

3. **Check logs:**

```bash
RUST_LOG=debug native-launcher 2>&1 | grep -i plugin
```

### Script Not Executing

1. **Test directly:**

```bash
./main.sh "test query"
```

2. **Check interpreter:**

```bash
which bash
which python3
```

3. **Check shebang:**

```bash
head -1 main.sh  # Should be #!/usr/bin/env bash or similar
```

### No Results Showing

1. **Verify trigger:**
   Make sure your query starts with the trigger string.

2. **Test output:**

```bash
./main.sh "query" | jq  # For JSON format
```

3. **Check priority:**
   Higher priority plugins are searched first.

---

## Contributing

Have a cool plugin idea? Share it!

1. Create your plugin in `examples/plugins/your-plugin/`
2. Add documentation (README.md)
3. Test thoroughly
4. Submit a Pull Request

### Plugin Ideas

- 📖 Dictionary (define words)
- 🌐 Translation (translate text)
- 💱 Cryptocurrency prices
- 📈 Stock ticker
- 🔑 Password generator
- 🔐 Base64 encoder/decoder
- 🔢 Hash calculator (MD5, SHA256)
- 🌍 IP lookup and geolocation
- 🐳 Docker container manager
- 📊 System info (CPU, RAM, disk)
- 🔍 Process killer
- 📋 Clipboard history
- 📝 Snippet manager
- 🔖 Bookmark manager
- 📅 Calendar quick add

---

## License

Example plugins are released under MIT License.
You're free to use, modify, and distribute them.

See [LICENSE](../LICENSE) for details.

---

## Support

- **Documentation:** [Plugin Development Guide](../docs/PLUGIN_DEVELOPMENT.md)
- **Wiki:** [Script Plugins](../wiki/Script-Plugins.md)
- **Issues:** [GitHub Issues](https://github.com/ArunPrakashG/native-launcher/issues)
- **Discussions:** [GitHub Discussions](https://github.com/ArunPrakashG/native-launcher/discussions)
