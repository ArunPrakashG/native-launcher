# Native Launcher Themes

This directory contains example themes for Native Launcher. To use a custom theme:

1. Create your theme CSS file based on one of the examples
2. Copy it to `~/.config/native-launcher/theme.css`
3. Restart Native Launcher

## Available Themes

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
