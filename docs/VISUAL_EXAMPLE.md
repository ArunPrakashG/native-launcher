# Visual Example: Inline Actions Display

## Before & After Comparison

### BEFORE (Separate Views Mode)

```
┌─────────────────────────────────────────────┐
│ Search Applications...                      │
├─────────────────────────────────────────────┤
│ ▸ Firefox                    → 3 actions    │  ← Selected, indicator on right
│   Visual Studio Code         → 2 actions    │
│   Chromium                   → 4 actions    │
│   Files                                     │
│   Terminal                                  │
└─────────────────────────────────────────────┘

Press Right Arrow →

┌─────────────────────────────────────────────┐
│ Search Applications...                      │
├─────────────────────────────────────────────┤
│   Firefox                                   │  ← Header (can launch)
│ ▸ New Window                                │  ← Indented action (selected)
│   New Private Window                        │  ← Indented action
│   Profile Manager                           │  ← Indented action
│                                             │
└─────────────────────────────────────────────┘
```

### AFTER (Inline Display)

```
┌─────────────────────────────────────────────┐
│ Search Applications...                      │
├─────────────────────────────────────────────┤
│ ▸ Firefox                                   │  ← Selected (white background)
│   ┃ New Window                             │  ← Coral text, indented
│   ┃ New Private Window                     │  ← Coral text, indented
│   ┃ Profile Manager                        │  ← Coral text, indented
│   Visual Studio Code                        │
│   ┃ New Window                             │  ← Coral text, indented
│   ┃ New Empty Window                       │  ← Coral text, indented
│   Chromium                                  │
│   ┃ New Window                             │  ← Coral text, indented
│   ┃ New Incognito Window                   │  ← Coral text, indented
│   ┃ New Guest Session                      │  ← Coral text, indented
│   ┃ Open in Safe Mode                      │  ← Coral text, indented
│   Files                                     │
│   Terminal                                  │
└─────────────────────────────────────────────┘

All actions visible immediately! Just use ↑↓ to navigate
```

## Color Coding

### Normal State (Not Selected)

```
Firefox                      ← White text (#ebebf5)
  ┃ New Window               ← Coral text (#ff6363) - stands out!
  ┃ New Private Window       ← Coral text (#ff6363) - stands out!
```

### Hover State

```
Firefox                      ← Light background (#2c2c2e)
  ┃ New Window               ← Coral hover (#ff7a7a), border coral
  ┃ New Private Window       ← Coral hover (#ff7a7a), border coral
```

### Selected State

```
▸ Firefox                    ← Bright coral background (#ff6363)
  ┃ New Window               ← White text, bright white border
  ┃ New Private Window       ← White text, bright white border
```

## Navigation Flow

### Example 1: Launch Main App

```
1. Type "fire"
2. See Firefox with actions below
3. Press Enter (Firefox is selected)
4. ✓ Firefox launches
```

### Example 2: Launch Action

```
1. Type "fire"
2. See Firefox with actions below
3. Press ↓ to select "New Private Window"
4. Press Enter
5. ✓ Firefox opens in private mode
```

### Example 3: Quick Navigation

```
1. Type "vs"
2. See VS Code with 2 actions below
3. Press ↓ once → "New Window" selected (1st action)
4. Press ↓ again → "New Empty Window" selected (2nd action)
5. Press ↓ again → Next app in list
6. No need to press Left/Right to enter/exit action mode!
```

## Visual Hierarchy

The design uses multiple visual cues to show parent-child relationships:

1. **Indentation**: Actions are indented 24px from the left
2. **Border**: Subtle left border (2px) indicates child items
3. **Color**: Coral (#ff6363) makes actions stand out from main app names
4. **Spacing**: Consistent padding maintains rhythm

```
Main Item (Level 0)
  ┃ Child Action (Level 1, +24px indent)
  ┃ Child Action (Level 1, +24px indent)
Main Item (Level 0)
Main Item (Level 0)
  ┃ Child Action (Level 1, +24px indent)
```

## Real-World Example

Searching for "chrom" might show:

```
┌─────────────────────────────────────────────┐
│ chrom                                       │
├─────────────────────────────────────────────┤
│ ▸ Chromium Web Browser                      │  ← Selected
│   Web Browser                               │  ← Generic name (dim)
│   ┃ New Window                             │  ← Action (coral)
│   ┃ New Incognito Window                   │  ← Action (coral)
│   ┃ New Guest Session                      │  ← Action (coral)
│   ┃ Open in Safe Mode                      │  ← Action (coral)
│                                             │
│   Google Chrome                             │
│   Web Browser                               │  ← Generic name (dim)
│   ┃ New Window                             │  ← Action (coral)
│   ┃ New Incognito Window                   │  ← Action (coral)
│                                             │
│   ChromeDriver                              │
│   WebDriver for Chrome                      │  ← Generic name (dim)
│                                             │  ← No actions
└─────────────────────────────────────────────┘
```

## Accessibility

- **Keyboard Only**: Full navigation with Up/Down arrows and Enter
- **Color Contrast**: Coral (#ff6363) on dark background meets WCAG AA
- **Visual Hierarchy**: Clear indentation and borders for screen readers
- **No Hidden Actions**: Everything visible without mode switching

## Performance

- **Efficient**: Single pass to build flat list
- **Scalable**: Handles apps with many actions (10+) gracefully
- **Fast**: No async operations, immediate display
- **Memory**: Minimal overhead with RefCell for items list
