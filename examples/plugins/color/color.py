#!/usr/bin/env python3
"""
Color picker plugin for Native Launcher
Converts between hex, RGB, HSL color formats
"""

import json
import re
import sys
from colorsys import hls_to_rgb, rgb_to_hls


def hex_to_rgb(hex_color):
    """Convert hex color to RGB tuple"""
    hex_color = hex_color.lstrip("#")
    if len(hex_color) == 3:
        hex_color = "".join([c * 2 for c in hex_color])
    return tuple(int(hex_color[i : i + 2], 16) for i in (0, 2, 4))


def rgb_to_hex(r, g, b):
    """Convert RGB to hex"""
    return f"#{r:02x}{g:02x}{b:02x}"


def rgb_to_hsl(r, g, b):
    """Convert RGB to HSL"""
    r, g, b = r / 255.0, g / 255.0, b / 255.0
    h, l, s = rgb_to_hls(r, g, b)
    return int(h * 360), int(s * 100), int(l * 100)


def hsl_to_rgb_tuple(h, s, l):
    """Convert HSL to RGB"""
    h, s, l = h / 360.0, s / 100.0, l / 100.0
    r, g, b = hls_to_rgb(h, l, s)
    return int(r * 255), int(g * 255), int(b * 255)


def parse_color(color_str):
    """Parse color from various formats"""
    color_str = color_str.strip().lower()

    # Hex format: #RRGGBB or #RGB
    hex_match = re.match(r"^#?([0-9a-f]{3}|[0-9a-f]{6})$", color_str)
    if hex_match:
        r, g, b = hex_to_rgb(hex_match.group(1))
        return ("hex", r, g, b)

    # RGB format: rgb(R, G, B) or r,g,b
    rgb_match = re.match(r"rgb\s*\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*\)", color_str)
    if rgb_match:
        r, g, b = map(int, rgb_match.groups())
        return ("rgb", r, g, b)

    # Simple comma format: R,G,B
    comma_match = re.match(r"(\d+)\s*,\s*(\d+)\s*,\s*(\d+)", color_str)
    if comma_match:
        r, g, b = map(int, comma_match.groups())
        return ("rgb", r, g, b)

    # HSL format: hsl(H, S%, L%)
    hsl_match = re.match(
        r"hsl\s*\(\s*(\d+)\s*,\s*(\d+)%?\s*,\s*(\d+)%?\s*\)", color_str
    )
    if hsl_match:
        h, s, l = map(int, hsl_match.groups())
        r, g, b = hsl_to_rgb_tuple(h, s, l)
        return ("hsl", r, g, b)

    return None


def generate_color_results(r, g, b, original_format):
    """Generate all color format results"""
    results = []

    # Validate RGB values
    r = max(0, min(255, r))
    g = max(0, min(255, g))
    b = max(0, min(255, b))

    hex_color = rgb_to_hex(r, g, b).upper()
    h, s, l = rgb_to_hsl(r, g, b)

    # Color preview using ANSI color (approximation)
    preview = "█████"

    # Hex format
    results.append(
        {
            "title": f"{preview}  {hex_color}",
            "subtitle": "Hex Color • Click to copy",
            "command": f"echo -n '{hex_color}' | wl-copy && notify-send 'Color Copied' '{hex_color}'",
        }
    )

    # RGB format
    rgb_str = f"rgb({r}, {g}, {b})"
    results.append(
        {
            "title": f"{preview}  {rgb_str}",
            "subtitle": "RGB Color • Click to copy",
            "command": f"echo -n '{rgb_str}' | wl-copy && notify-send 'Color Copied' '{rgb_str}'",
        }
    )

    # HSL format
    hsl_str = f"hsl({h}, {s}%, {l}%)"
    results.append(
        {
            "title": f"{preview}  {hsl_str}",
            "subtitle": "HSL Color • Click to copy",
            "command": f"echo -n '{hsl_str}' | wl-copy && notify-send 'Color Copied' '{hsl_str}'",
        }
    )

    # CSS variable format
    css_var = f"--color: {hex_color};"
    results.append(
        {
            "title": f"{preview}  {css_var}",
            "subtitle": "CSS Variable • Click to copy",
            "command": f"echo -n '{css_var}' | wl-copy && notify-send 'Color Copied' 'CSS Variable'",
        }
    )

    # Tailwind CSS class (approximate)
    tailwind_hint = f"Use hex {hex_color} in Tailwind"
    results.append(
        {
            "title": f"{preview}  {tailwind_hint}",
            "subtitle": "Add to tailwind.config.js colors",
            "command": f"echo -n '{hex_color}' | wl-copy && notify-send 'Color Copied' 'For Tailwind'",
        }
    )

    return results


def main():
    query = sys.argv[1] if len(sys.argv) > 1 else ""

    if not query:
        output = {
            "results": [
                {
                    "title": "Enter a color...",
                    "subtitle": "Examples: #FF5733, rgb(255,87,51), hsl(9,100%,60%)",
                    "command": "echo 'Enter a color' | wl-copy",
                }
            ]
        }
    else:
        parsed = parse_color(query)

        if parsed:
            format_type, r, g, b = parsed
            results = generate_color_results(r, g, b, format_type)
            output = {"results": results}
        else:
            output = {
                "results": [
                    {
                        "title": "Invalid color format",
                        "subtitle": f"Could not parse: {query}",
                        "command": "echo 'Invalid color' | wl-copy",
                    },
                    {
                        "title": "Supported formats:",
                        "subtitle": "#FF5733, rgb(255,87,51), hsl(9,100%,60%)",
                        "command": "echo 'See examples' | wl-copy",
                    },
                ]
            }

    print(json.dumps(output, indent=2))


if __name__ == "__main__":
    main()
