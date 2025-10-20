# Raycast Design Language Implementation

This document describes how we've implemented Raycast's design language in Native Launcher.

## Overview

Raycast is known for its clean, minimal, and professional design aesthetic. Their design language emphasizes:

- **Clarity**: Clean typography and clear visual hierarchy
- **Speed**: Fast, responsive interactions with minimal animation overhead
- **Professional**: Dark theme with subtle, sophisticated color palette
- **Consistency**: Predictable patterns that users can rely on

## Color Palette

We've adopted Raycast's characteristic color scheme:

### Primary Colors

- **Raycast Red/Coral**: `#FF6363` - Used for accents and selected states
- **Hover State**: `#FF7A7A` - Slightly lighter for hover feedback
- **Active State**: `#FF4D4D` - Slightly darker for active clicks

### Background Colors

- **Primary Background**: `#1C1C1E` - Main dark charcoal background
- **Secondary Background**: `#2C2C2E` - Input fields and hover states
- **Tertiary Background**: `#3A3A3C` - Borders and dividers

### Text Colors

- **Primary Text**: `#FFFFFF` - Main headings and labels
- **Secondary Text**: `#EBEBF5` - Regular text content
- **Tertiary Text**: `#EBEBF599` - 60% opacity - Descriptions and metadata
- **Quaternary Text**: `#EBEBF54D` - 30% opacity - Placeholders and disabled

## Design Principles

### 1. Minimal Borders

Raycast uses very subtle borders. We use:

- `1px solid` borders in dark gray (`#3A3A3C`)
- Transparent borders on non-selected items
- Borders serve to separate sections, not highlight them

### 2. Flat Design with Subtle Depth

Unlike our previous glassmorphism design, Raycast prefers:

- Flat color fills instead of gradients
- Minimal shadows (only on window itself)
- No transform effects on hover (no scale or translate)
- Subtle background color changes for states

### 3. Restrained Animation

- Fast transitions: `0.15s` (vs our previous 0.25-0.3s)
- Cubic bezier: `cubic-bezier(0.4, 0, 0.2, 1)` - smooth but quick
- No elaborate animations or pulse effects
- Color transitions only, no transforms

### 4. Typography

- **Font Size**: 20px for search input (large and comfortable)
- **Font Weight**:
  - 400 (regular) for input and body text
  - 500 (medium) for labels
  - 600 (semibold) for selected items
- **Line Height**: Comfortable spacing for readability
- **No Text Shadows**: Clean, flat text rendering

## Component Styles

### Window

- **Background**: Dark charcoal (`#1C1C1E`) with subtle gradient to `#18181A`
- **Border Radius**: 16px (less rounded than before)
- **Border**: 1px solid with subtle inner highlight
- **Shadow**: Single strong shadow (no multiple layers)

### Search Entry

- **Background**: Secondary background color (`#2C2C2E`)
- **Padding**: 14px vertical, 20px horizontal
- **Border Radius**: 10px
- **Focus State**: Red border with subtle outer glow (0 0 0 3px rgba(255, 99, 99, 0.15))
- **No transform effects**: Stays in place

### Result Items

- **Default**: Transparent background, minimal padding
- **Hover**: Secondary background color only
- **Selected**: Primary color background (`#FF6363`)
- **Spacing**: 2px margin between items
- **Border Radius**: 8px (subtle rounding)
- **No shadows or transforms**: Pure flat design

### Scrollbar

- **Width**: 6px (minimal, grows to 8px on hover)
- **Color**: Quaternary text color, becomes tertiary on hover
- **Active**: Primary color
- **Border Radius**: 10px (pill shape)

## Differences from Previous Design

| Aspect     | Previous (Glassmorphism)  | Current (Raycast)    |
| ---------- | ------------------------- | -------------------- |
| Background | Gradient with blur effect | Flat dark charcoal   |
| Shadows    | Multiple layered shadows  | Single window shadow |
| Animations | 0.25-0.3s with transforms | 0.15s color only     |
| Selection  | Blue gradient with glow   | Solid red background |
| Hover      | Scale + translate effects | Color change only    |
| Borders    | Colored glowing borders   | Subtle gray borders  |
| Text       | Text shadows              | Flat rendering       |

## Implementation Details

### CSS Variables

All colors are defined as CSS custom properties (`:root`) for easy theming:

```css
--raycast-primary: #ff6363;
--raycast-bg-primary: #1c1c1e;
--raycast-text-primary: #ffffff;
```

### State Management

- **Normal**: Transparent/primary background
- **Hover**: Secondary background
- **Selected**: Primary color
- **Selected + Hover**: Slightly lighter primary

### Accessibility

- High contrast text (white on dark)
- Clear selection states
- Visible focus indicators (red border glow)
- Comfortable font sizes

## Visual Comparison

### Before (Glassmorphism)

- Translucent backgrounds with blur
- Blue/purple color scheme
- Heavy use of shadows and glows
- Transform effects (scale, translate)
- Slower, more elaborate animations

### After (Raycast)

- Solid dark backgrounds
- Red/coral accent color
- Minimal shadows, flat design
- No transform effects
- Fast, snappy transitions

## Future Enhancements

To further match Raycast:

1. **Icons**: Display app icons on the left of each result
2. **Keyboard Shortcuts**: Show keyboard hints on the right
3. **Sections**: Group results by category with headers
4. **Metadata**: Show more app details (file path, version)
5. **Quick Actions**: Secondary actions per result (hover to reveal)

## Credits

Design inspiration from [Raycast](https://www.raycast.com/) - the blazingly fast, totally extendable launcher for macOS.
