# Native Launcher Themes

This directory contains built-in themes for Native Launcher. You can use these themes or create your own custom theme.

## Using Built-in Themes

**Method 1: Configuration File (Recommended)**

Edit `~/.config/native-launcher/config.toml`:

```toml
[ui]
# Use a built-in theme by name
theme = "dracula"  # or "dark", "light", "nord", "high-contrast"
```

**Method 2: Custom CSS File**

You can also specify a path to a custom CSS file:

```toml
[ui]
# Use custom theme file (absolute path)
theme = "/home/user/my-custom-theme.css"

# Or use the default custom theme location
# Copy your theme to: ~/.config/native-launcher/theme.css
# Then set: theme = "dark" (or any name, it will use theme.css if found)
```

## Theme Loading Priority

Native Launcher loads themes in this order:

1. **Absolute path** - If `theme` starts with `/`, load that file
2. **Built-in theme by name** - If it matches a built-in theme name
3. **User custom theme** - `~/.config/native-launcher/theme.css` (if exists)
4. **Default fallback** - Built-in dark theme

## Theme Customization

All themes support customization through CSS variables. You can override any of these in your custom theme:

### Color Variables

```css
:root {
  /* Primary accent colors */
  --nl-primary: #ff6363;
  --nl-primary-hover: #ff7a7a;
  --nl-primary-active: #ff4d4d;

  /* Background colors */
  --nl-bg-primary: #1c1c1e; /* Main window background */
  --nl-bg-secondary: #2c2c2e; /* Search input, results container */
  --nl-bg-tertiary: #3a3a3c; /* Hover states */

  /* Text colors */
  --nl-text-primary: #ffffff; /* Main text */
  --nl-text-secondary: #ebebf5; /* Secondary text */
  --nl-text-tertiary: #ebebf599; /* Dim labels (60% opacity) */
  --nl-text-quaternary: #ebebf54d; /* Very dim (30% opacity) */

  /* Borders and dividers */
  --nl-border: #3a3a3c;
  --nl-divider: #48484a;

  /* Shadows */
  --nl-shadow: rgba(0, 0, 0, 0.3);
  --nl-shadow-strong: rgba(0, 0, 0, 0.5);

  /* Border Radius - NEW: Unified design language */
  --nl-radius-window: 20px; /* Main window corners */
  --nl-radius-large: 16px; /* Search input, results container */
  --nl-radius-medium: 14px; /* List items */
  --nl-radius-small: 10px; /* Icons, buttons */
  --nl-radius-tiny: 8px; /* Small UI elements */

  /* Animation timings */
  --nl-animation-fast: 0.08s;
  --nl-animation-normal: 0.12s;
  --nl-animation-slow: 0.15s;
  --nl-easing: cubic-bezier(0.4, 0, 0.2, 1);
}
```

### Creating a Custom Theme

1. **Copy a base theme** as a starting point
2. **Modify the variables** to match your preferences
3. **Save to config directory** or specify absolute path

**Example: Sharp corners theme**

```css
/* ~/.config/native-launcher/theme.css */
@import url("path/to/dark.css"); /* Optional: base theme */

:root {
  /* Override for sharp corners */
  --nl-radius-window: 0px;
  --nl-radius-large: 0px;
  --nl-radius-medium: 0px;
  --nl-radius-small: 0px;
  --nl-radius-tiny: 0px;

  /* Custom accent color */
  --nl-primary: #00ff00;
}
```

**Example: Extra rounded theme**

```css
:root {
  --nl-radius-window: 30px;
  --nl-radius-large: 24px;
  --nl-radius-medium: 20px;
  --nl-radius-small: 16px;
  --nl-radius-tiny: 12px;
}
```

## Available Built-in Themes

### Default (Dark)

The built-in theme with coral accents on dark charcoal background.

- **File**: `dark.css` (same as built-in `src/ui/style.css`)
- **Primary Color**: Coral (#ff6363)
- **Background**: Dark Charcoal (#1c1c1e)

### Light Theme

A clean light theme inspired by macOS Spotlight.

- **File**: `light.css`
- **Primary Color**: Blue (#007aff)
- **Background**: Light Gray (#f5f5f7)

### High Contrast

Accessibility-focused theme with high contrast colors.

- **File**: `high-contrast.css`
- **Primary Color**: Bright Yellow (#ffff00)
- **Background**: Pure Black (#000000)

### Dracula

Popular dark theme with purple accents.

- **File**: `dracula.css`
- **Primary Color**: Purple (#bd93f9)
- **Background**: Dark Purple (#282a36)

### Nord

Cool blue-tinted theme.

- **File**: `nord.css`
- **Primary Color**: Frost Blue (#88c0d0)
- **Background**: Polar Night (#2e3440)

## CSS Class Reference

### Main Components

- `window` - Main launcher window
- `entry` - Search input field
- `scrolledwindow` - Results container
- `listbox` - Results list
- `row` - Individual result row

### Result Items

- `.app-name` - Application name label
- `.app-generic` - Generic name/subtitle label
- `.app-icon` - Application icon
- `.action-name` - Desktop action name

### States

- `:focus` - Focused element
- `:hover` - Hovered element
- `:selected` - Selected list item
- `:active` - Active/pressed element

### Keyboard Hints

- `.keyboard-hints` - Bottom hints container
- `.hint-key` - Keyboard key indicator
- `.hint-text` - Hint description text

## Creating Custom Themes

### Basic Template

```css
/* My Custom Theme */
:root {
  /* Primary accent color */
  --primary-color: #your-color;
  --primary-hover: #your-hover-color;

  /* Background colors */
  --bg-primary: #your-bg;
  --bg-secondary: #your-secondary-bg;

  /* Text colors */
  --text-primary: #your-text;
  --text-secondary: #your-secondary-text;

  /* Border colors */
  --border-color: #your-border;
}

/* Apply your colors to components */
window {
  background-color: var(--bg-primary);
  border: 1px solid var(--border-color);
}

entry {
  background-color: var(--bg-secondary);
  color: var(--text-primary);
}

/* ... customize other elements ... */
```

### Tips

1. **Test with different apps**: Make sure your colors work with various icon colors
2. **Check contrast**: Ensure text is readable (WCAG AA: 4.5:1 ratio)
3. **Consider transparency**: Some compositors support transparent backgrounds
4. **Animation smoothness**: Keep transitions under 200ms for responsiveness
5. **Icon visibility**: Test with both light and dark icons

## Installation

```bash
# Copy a theme to your config directory
mkdir -p ~/.config/native-launcher
cp themes/light.css ~/.config/native-launcher/theme.css

# Or create your own
nano ~/.config/native-launcher/theme.css
```

## Sharing Themes

If you create a theme you'd like to share, please:

1. Fork the repository
2. Add your theme to `themes/`
3. Update this README with a description
4. Submit a pull request

Happy theming! 🎨
