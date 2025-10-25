#!/usr/bin/env bash
# Weather plugin script for Native Launcher
# Uses wttr.in API to fetch weather data

set -euo pipefail

# Get location from argument (default to current location)
LOCATION="${1:-}"

if [ -z "$LOCATION" ]; then
    # No query - show helpful message
    cat <<EOF
{
  "results": [
    {
      "title": "Type a city name...",
      "subtitle": "Example: weather Tokyo or w London",
      "command": "echo 'Enter a location' | wl-copy"
    }
  ]
}
EOF
    exit 0
fi

# Fetch weather data from wttr.in
# Format: Location: Condition Temperature Humidity Wind
WEATHER_DATA=$(curl -s "wttr.in/${LOCATION}?format=%l:+%C+%t+%h+%w" 2>/dev/null || echo "Error fetching weather")

if [[ "$WEATHER_DATA" == "Error"* ]] || [[ "$WEATHER_DATA" == "Unknown location"* ]]; then
    cat <<EOF
{
  "results": [
    {
      "title": "Location not found",
      "subtitle": "Could not find weather for '$LOCATION'",
      "command": "echo 'Location not found' | wl-copy"
    }
  ]
}
EOF
    exit 0
fi

# Parse the weather data
# Format: City: Condition Temperature Humidity Wind
IFS=':+' read -r CITY CONDITION TEMP HUMIDITY WIND <<< "$WEATHER_DATA"

# Get full forecast URL
FORECAST_URL="https://wttr.in/${LOCATION}"

# Build JSON output
cat <<EOF
{
  "results": [
    {
      "title": "${CITY// /} - ${CONDITION// /}",
      "subtitle": "🌡️ ${TEMP// /} • 💧 ${HUMIDITY// /} • 💨 ${WIND// /} • Press Enter to copy",
      "command": "echo '${CITY}: ${CONDITION} ${TEMP} ${HUMIDITY} ${WIND}' | wl-copy && notify-send 'Weather Copied' '${CITY}'"
    },
    {
      "title": "View Detailed Forecast",
      "subtitle": "Open wttr.in in browser",
      "command": "xdg-open '${FORECAST_URL}'"
    },
    {
      "title": "ASCII Forecast",
      "subtitle": "Show detailed ASCII weather in terminal",
      "command": "alacritty -e bash -c 'curl wttr.in/${LOCATION}; read -p \"Press Enter to close\"'"
    }
  ]
}
EOF
