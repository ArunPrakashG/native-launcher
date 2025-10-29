use super::traits::{Plugin, PluginContext, PluginResult};
use anyhow::Result;
use chrono::{Duration, Local, Utc};
use std::collections::HashMap;

/// Advanced calculator plugin with time, unit, and currency conversions
#[derive(Debug)]
pub struct AdvancedCalculatorPlugin {
    enabled: bool,
    currency_rates: HashMap<String, f64>, // Base: USD
}

impl AdvancedCalculatorPlugin {
    pub fn new() -> Self {
        // Initialize with some basic currency rates (should be fetched from API in production)
        let mut currency_rates = HashMap::new();

        // Base rates (as of 2024 - static fallback)
        currency_rates.insert("USD".to_string(), 1.0);
        currency_rates.insert("EUR".to_string(), 0.92);
        currency_rates.insert("GBP".to_string(), 0.79);
        currency_rates.insert("JPY".to_string(), 149.50);
        currency_rates.insert("CNY".to_string(), 7.24);
        currency_rates.insert("INR".to_string(), 83.12);
        currency_rates.insert("CAD".to_string(), 1.36);
        currency_rates.insert("AUD".to_string(), 1.53);
        currency_rates.insert("CHF".to_string(), 0.88);
        currency_rates.insert("KRW".to_string(), 1329.0);

        Self {
            enabled: true,
            currency_rates,
        }
    }

    /// Parse time-based queries like "1 hour ago", "350 days ago", "in 5 hours"
    fn parse_time_query(&self, query: &str) -> Option<Vec<PluginResult>> {
        let query_lower = query.to_lowercase();
        let now = Local::now();

        // Pattern: "X <unit> ago" or "X <unit> from now" or "in X <unit>"
        let re_ago =
            regex::Regex::new(r"^\s*(\d+)\s*(second|minute|hour|day|week|month|year)s?\s+ago\s*$")
                .ok()?;
        let re_from_now = regex::Regex::new(
            r"^\s*(in\s+)?(\d+)\s*(second|minute|hour|day|week|month|year)s?(\s+from\s+now)?\s*$",
        )
        .ok()?;

        if let Some(caps) = re_ago.captures(&query_lower) {
            let amount: i64 = caps.get(1)?.as_str().parse().ok()?;
            let unit = caps.get(2)?.as_str();

            let past_time = match unit {
                "second" => now - Duration::seconds(amount),
                "minute" => now - Duration::minutes(amount),
                "hour" => now - Duration::hours(amount),
                "day" => now - Duration::days(amount),
                "week" => now - Duration::weeks(amount),
                "month" => now - Duration::days(amount * 30),
                "year" => now - Duration::days(amount * 365),
                _ => return None,
            };

            let local_time = past_time.format("%Y-%m-%d %H:%M:%S").to_string();
            let utc_time = past_time
                .with_timezone(&Utc)
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string();
            let timestamp = past_time.timestamp().to_string();

            return Some(vec![
                PluginResult::new(
                    local_time.clone(),
                    format!(
                        "echo '{}' | wl-copy && notify-send 'Copied to Clipboard' '{}'",
                        local_time, local_time
                    ),
                    self.name().to_string(),
                )
                .with_subtitle(format!("{} ago (Local time) • Press Enter to copy", query))
                .with_icon("appointment-new".to_string())
                .with_score(9500),
                PluginResult::new(
                    utc_time.clone(),
                    format!(
                        "echo '{}' | wl-copy && notify-send 'Copied to Clipboard' '{}'",
                        utc_time, utc_time
                    ),
                    self.name().to_string(),
                )
                .with_subtitle("UTC time • Press Enter to copy".to_string())
                .with_icon("appointment-new".to_string())
                .with_score(9400),
                PluginResult::new(
                    format!("Unix timestamp: {}", timestamp),
                    format!(
                        "echo '{}' | wl-copy && notify-send 'Copied to Clipboard' 'Timestamp: {}'",
                        timestamp, timestamp
                    ),
                    self.name().to_string(),
                )
                .with_subtitle("Seconds since epoch • Press Enter to copy".to_string())
                .with_icon("appointment-new".to_string())
                .with_score(9300),
            ]);
        }

        if let Some(caps) = re_from_now.captures(&query_lower) {
            let amount: i64 = caps.get(2)?.as_str().parse().ok()?;
            let unit = caps.get(3)?.as_str();

            let future_time = match unit {
                "second" => now + Duration::seconds(amount),
                "minute" => now + Duration::minutes(amount),
                "hour" => now + Duration::hours(amount),
                "day" => now + Duration::days(amount),
                "week" => now + Duration::weeks(amount),
                "month" => now + Duration::days(amount * 30),
                "year" => now + Duration::days(amount * 365),
                _ => return None,
            };

            let local_time = future_time.format("%Y-%m-%d %H:%M:%S").to_string();
            let utc_time = future_time
                .with_timezone(&Utc)
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string();

            return Some(vec![
                PluginResult::new(
                    local_time.clone(),
                    format!(
                        "echo '{}' | wl-copy && notify-send 'Copied to Clipboard' '{}'",
                        local_time, local_time
                    ),
                    self.name().to_string(),
                )
                .with_subtitle(format!("In {} (Local time) • Press Enter to copy", query))
                .with_icon("appointment-new".to_string())
                .with_score(9500),
                PluginResult::new(
                    utc_time.clone(),
                    format!(
                        "echo '{}' | wl-copy && notify-send 'Copied to Clipboard' '{}'",
                        utc_time, utc_time
                    ),
                    self.name().to_string(),
                )
                .with_subtitle("UTC time • Press Enter to copy".to_string())
                .with_icon("appointment-new".to_string())
                .with_score(9400),
            ]);
        }

        None
    }

    /// Parse unit conversions like "150 days to years", "5 km to miles"
    fn parse_unit_conversion(&self, query: &str) -> Option<Vec<PluginResult>> {
        let query_lower = query.to_lowercase();

        // Pattern: "X <from_unit> to <to_unit>"
        let re = regex::Regex::new(r"(\d+\.?\d*)\s*(\w+)\s+to\s+(\w+)").ok()?;

        if let Some(caps) = re.captures(&query_lower) {
            let value: f64 = caps.get(1)?.as_str().parse().ok()?;
            let from_unit = caps.get(2)?.as_str();
            let to_unit = caps.get(3)?.as_str();

            // Time conversions
            let time_result = self.convert_time_units(value, from_unit, to_unit);
            if let Some(result) = time_result {
                return Some(vec![result]);
            }

            // Distance conversions
            let distance_result = self.convert_distance_units(value, from_unit, to_unit);
            if let Some(result) = distance_result {
                return Some(vec![result]);
            }

            // Weight conversions
            let weight_result = self.convert_weight_units(value, from_unit, to_unit);
            if let Some(result) = weight_result {
                return Some(vec![result]);
            }

            // Temperature conversions
            let temp_result = self.convert_temperature(value, from_unit, to_unit);
            if let Some(result) = temp_result {
                return Some(vec![result]);
            }
        }

        None
    }

    /// Convert time units
    fn convert_time_units(&self, value: f64, from: &str, to: &str) -> Option<PluginResult> {
        let to_seconds = match from {
            "second" | "seconds" | "s" | "sec" => value,
            "minute" | "minutes" | "m" | "min" => value * 60.0,
            "hour" | "hours" | "h" | "hr" => value * 3600.0,
            "day" | "days" | "d" => value * 86400.0,
            "week" | "weeks" | "w" => value * 604800.0,
            "month" | "months" => value * 2592000.0, // 30 days
            "year" | "years" | "y" => value * 31536000.0, // 365 days
            _ => return None,
        };

        let result = match to {
            "second" | "seconds" | "s" | "sec" => to_seconds,
            "minute" | "minutes" | "m" | "min" => to_seconds / 60.0,
            "hour" | "hours" | "h" | "hr" => to_seconds / 3600.0,
            "day" | "days" | "d" => to_seconds / 86400.0,
            "week" | "weeks" | "w" => to_seconds / 604800.0,
            "month" | "months" => to_seconds / 2592000.0,
            "year" | "years" | "y" => to_seconds / 31536000.0,
            _ => return None,
        };

        Some(
            PluginResult::new(
                format!("{:.2} {}", result, to),
                format!("echo '{:.2} {}'", result, to),
                self.name().to_string(),
            )
            .with_subtitle(format!("{} {} = {:.2} {}", value, from, result, to))
            .with_icon("appointment-new".to_string())
            .with_score(9500),
        )
    }

    /// Convert distance units
    fn convert_distance_units(&self, value: f64, from: &str, to: &str) -> Option<PluginResult> {
        let to_meters = match from {
            "mm" | "millimeter" | "millimeters" => value / 1000.0,
            "cm" | "centimeter" | "centimeters" => value / 100.0,
            "m" | "meter" | "meters" => value,
            "km" | "kilometer" | "kilometers" => value * 1000.0,
            "inch" | "inches" | "in" => value * 0.0254,
            "foot" | "feet" | "ft" => value * 0.3048,
            "yard" | "yards" | "yd" => value * 0.9144,
            "mile" | "miles" | "mi" => value * 1609.34,
            _ => return None,
        };

        let result = match to {
            "mm" | "millimeter" | "millimeters" => to_meters * 1000.0,
            "cm" | "centimeter" | "centimeters" => to_meters * 100.0,
            "m" | "meter" | "meters" => to_meters,
            "km" | "kilometer" | "kilometers" => to_meters / 1000.0,
            "inch" | "inches" | "in" => to_meters / 0.0254,
            "foot" | "feet" | "ft" => to_meters / 0.3048,
            "yard" | "yards" | "yd" => to_meters / 0.9144,
            "mile" | "miles" | "mi" => to_meters / 1609.34,
            _ => return None,
        };

        Some(
            PluginResult::new(
                format!("{:.4} {}", result, to),
                format!("echo '{:.4} {}'", result, to),
                self.name().to_string(),
            )
            .with_subtitle(format!("{} {} = {:.4} {}", value, from, result, to))
            .with_icon("emblem-system".to_string())
            .with_score(9500),
        )
    }

    /// Convert weight units
    fn convert_weight_units(&self, value: f64, from: &str, to: &str) -> Option<PluginResult> {
        let to_grams = match from {
            "mg" | "milligram" | "milligrams" => value / 1000.0,
            "g" | "gram" | "grams" => value,
            "kg" | "kilogram" | "kilograms" => value * 1000.0,
            "oz" | "ounce" | "ounces" => value * 28.3495,
            "lb" | "pound" | "pounds" => value * 453.592,
            "ton" | "tons" => value * 907185.0,
            _ => return None,
        };

        let result = match to {
            "mg" | "milligram" | "milligrams" => to_grams * 1000.0,
            "g" | "gram" | "grams" => to_grams,
            "kg" | "kilogram" | "kilograms" => to_grams / 1000.0,
            "oz" | "ounce" | "ounces" => to_grams / 28.3495,
            "lb" | "pound" | "pounds" => to_grams / 453.592,
            "ton" | "tons" => to_grams / 907185.0,
            _ => return None,
        };

        Some(
            PluginResult::new(
                format!("{:.4} {}", result, to),
                format!("echo '{:.4} {}'", result, to),
                self.name().to_string(),
            )
            .with_subtitle(format!("{} {} = {:.4} {}", value, from, result, to))
            .with_icon("emblem-system".to_string())
            .with_score(9500),
        )
    }

    /// Convert temperature
    fn convert_temperature(&self, value: f64, from: &str, to: &str) -> Option<PluginResult> {
        let to_celsius = match from {
            "c" | "celsius" => value,
            "f" | "fahrenheit" => (value - 32.0) * 5.0 / 9.0,
            "k" | "kelvin" => value - 273.15,
            _ => return None,
        };

        let result = match to {
            "c" | "celsius" => to_celsius,
            "f" | "fahrenheit" => to_celsius * 9.0 / 5.0 + 32.0,
            "k" | "kelvin" => to_celsius + 273.15,
            _ => return None,
        };

        Some(
            PluginResult::new(
                format!("{:.2}°{}", result, to.to_uppercase().chars().next()?),
                format!("echo '{:.2}°{}'", result, to.to_uppercase().chars().next()?),
                self.name().to_string(),
            )
            .with_subtitle(format!(
                "{:.2}°{} = {:.2}°{}",
                value,
                from.to_uppercase().chars().next()?,
                result,
                to.to_uppercase().chars().next()?
            ))
            .with_icon("weather-clear".to_string())
            .with_score(9500),
        )
    }

    /// Parse currency conversions like "100 USD to EUR"
    fn parse_currency_conversion(&self, query: &str) -> Option<Vec<PluginResult>> {
        let query_upper = query.to_uppercase();

        // Pattern: "X <currency_from> to <currency_to>"
        let re = regex::Regex::new(r"(\d+\.?\d*)\s*([A-Z]{3})\s+TO\s+([A-Z]{3})").ok()?;

        if let Some(caps) = re.captures(&query_upper) {
            let amount: f64 = caps.get(1)?.as_str().parse().ok()?;
            let from_currency = caps.get(2)?.as_str();
            let to_currency = caps.get(3)?.as_str();

            let from_rate = self.currency_rates.get(from_currency)?;
            let to_rate = self.currency_rates.get(to_currency)?;

            // Convert: amount * (to_rate / from_rate)
            let result = amount * (to_rate / from_rate);

            return Some(vec![PluginResult::new(
                format!("{:.2} {}", result, to_currency),
                format!("echo '{:.2} {}'", result, to_currency),
                self.name().to_string(),
            )
            .with_subtitle(format!(
                "{} {} ≈ {:.2} {}",
                amount, from_currency, result, to_currency
            ))
            .with_icon("emblem-money".to_string())
            .with_score(9500)]);
        }

        None
    }

    /// Parse timezone conversions like "now in UTC", "5pm EST to PST"
    fn parse_timezone_query(&self, query: &str) -> Option<Vec<PluginResult>> {
        let query_lower = query.to_lowercase();

        if query_lower.contains("now")
            && (query_lower.contains("utc") || query_lower.contains("timezone"))
        {
            let now_local = Local::now();
            let now_utc = Utc::now();

            let local_str = now_local.format("%Y-%m-%d %H:%M:%S %Z").to_string();
            let utc_str = now_utc.format("%Y-%m-%d %H:%M:%S %Z").to_string();

            return Some(vec![
                PluginResult::new(
                    format!("Local: {}", local_str),
                    format!(
                        "echo '{}' | wl-copy && notify-send 'Copied to Clipboard' '{}'",
                        local_str, local_str
                    ),
                    self.name().to_string(),
                )
                .with_subtitle("Current local time • Press Enter to copy".to_string())
                .with_icon("appointment-new".to_string())
                .with_score(9500),
                PluginResult::new(
                    format!("UTC: {}", utc_str),
                    format!(
                        "echo '{}' | wl-copy && notify-send 'Copied to Clipboard' '{}'",
                        utc_str, utc_str
                    ),
                    self.name().to_string(),
                )
                .with_subtitle("Current UTC time • Press Enter to copy".to_string())
                .with_icon("appointment-new".to_string())
                .with_score(9400),
            ]);
        }

        None
    }
}

impl Default for AdvancedCalculatorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for AdvancedCalculatorPlugin {
    fn name(&self) -> &str {
        "advanced_calculator"
    }

    fn description(&self) -> &str {
        "Advanced calculations: time (1 hour ago), unit conversions (150 days to years), currency (100 USD to EUR), timezone conversions"
    }

    fn command_prefixes(&self) -> Vec<&str> {
        vec!["@calc", "@convert", "@time", "@currency"]
    }

    fn priority(&self) -> i32 {
        850 // Higher than regular calculator
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn should_handle(&self, query: &str) -> bool {
        if !self.enabled || query.is_empty() {
            return false;
        }

        let query_lower = query.to_lowercase();

        // Check for time-based queries
        if query_lower.contains("ago")
            || query_lower.contains("from now")
            || query_lower.starts_with("in ")
        {
            return true;
        }

        // Check for conversion queries
        if query_lower.contains(" to ") {
            return true;
        }

        // Check for timezone queries
        if query_lower.contains("utc") || query_lower.contains("timezone") {
            return true;
        }

        // Check for explicit commands
        query_lower.starts_with("@calc")
            || query_lower.starts_with("@convert")
            || query_lower.starts_with("@time")
            || query_lower.starts_with("@currency")
    }

    fn search(&self, query: &str, _context: &PluginContext) -> Result<Vec<PluginResult>> {
        if !self.enabled {
            return Ok(vec![]);
        }

        // Try time-based queries first
        if let Some(results) = self.parse_time_query(query) {
            return Ok(results);
        }

        // Try unit conversions
        if let Some(results) = self.parse_unit_conversion(query) {
            return Ok(results);
        }

        // Try currency conversions
        if let Some(results) = self.parse_currency_conversion(query) {
            return Ok(results);
        }

        // Try timezone queries
        if let Some(results) = self.parse_timezone_query(query) {
            return Ok(results);
        }

        Ok(vec![])
    }
}
