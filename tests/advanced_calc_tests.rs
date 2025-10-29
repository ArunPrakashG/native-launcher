use native_launcher::config::Config;
use native_launcher::plugins::traits::{Plugin, PluginContext};
use native_launcher::plugins::AdvancedCalculatorPlugin;

#[test]
fn test_time_ago_query() {
    let plugin = AdvancedCalculatorPlugin::new();
    let config = Config::default();
    let context = PluginContext::new(10, &config);

    // Test "1 hour ago"
    let results = plugin
        .search("1 hour ago", &context)
        .expect("search failed");
    assert!(
        !results.is_empty(),
        "Should return results for '1 hour ago'"
    );
    assert!(results[0].title.contains(":"), "Should contain time format");

    // Test "5 days ago"
    let results = plugin
        .search("5 days ago", &context)
        .expect("search failed");
    assert!(
        !results.is_empty(),
        "Should return results for '5 days ago'"
    );
}

#[test]
fn test_time_from_now() {
    let plugin = AdvancedCalculatorPlugin::new();
    let config = Config::default();
    let context = PluginContext::new(10, &config);

    // Test "in 2 hours"
    let results = plugin
        .search("in 2 hours", &context)
        .expect("search failed");
    assert!(
        !results.is_empty(),
        "Should return results for 'in 2 hours'"
    );

    // Test "5 days from now"
    let results = plugin
        .search("5 days from now", &context)
        .expect("search failed");
    assert!(
        !results.is_empty(),
        "Should return results for '5 days from now'"
    );
}

#[test]
fn test_unit_conversions() {
    let plugin = AdvancedCalculatorPlugin::new();
    let config = Config::default();
    let context = PluginContext::new(10, &config);

    // Test time unit conversion
    let results = plugin
        .search("150 days to years", &context)
        .expect("search failed");
    assert!(
        !results.is_empty(),
        "Should return results for '150 days to years'"
    );
    let title = results[0].title.to_lowercase();
    assert!(
        title.contains("year"),
        "Should contain 'year', got: {}",
        results[0].title
    );

    // Test distance conversion
    let results = plugin
        .search("5 km to miles", &context)
        .expect("search failed");
    assert!(
        !results.is_empty(),
        "Should return results for '5 km to miles'"
    );
    assert!(results[0].title.contains("mile"), "Should contain 'mile'");

    // Test weight conversion
    let results = plugin
        .search("100 pounds to kg", &context)
        .expect("search failed");
    assert!(
        !results.is_empty(),
        "Should return results for '100 pounds to kg'"
    );
    assert!(results[0].title.contains("kg"), "Should contain 'kg'");

    // Test temperature conversion
    let results = plugin
        .search("32 fahrenheit to celsius", &context)
        .expect("search failed");
    assert!(
        !results.is_empty(),
        "Should return results for '32 fahrenheit to celsius'"
    );
    assert!(
        results[0].title.contains("°"),
        "Should contain degree symbol"
    );
}

#[test]
fn test_currency_conversion() {
    let plugin = AdvancedCalculatorPlugin::new();
    let config = Config::default();
    let context = PluginContext::new(10, &config);

    // Test currency conversion
    let results = plugin
        .search("100 USD to EUR", &context)
        .expect("search failed");
    assert!(
        !results.is_empty(),
        "Should return results for '100 USD to EUR'"
    );
    assert!(results[0].title.contains("EUR"), "Should contain 'EUR'");

    // Test another currency pair
    let results = plugin
        .search("50 GBP to JPY", &context)
        .expect("search failed");
    assert!(
        !results.is_empty(),
        "Should return results for '50 GBP to JPY'"
    );
    assert!(results[0].title.contains("JPY"), "Should contain 'JPY'");
}

#[test]
fn test_timezone_query() {
    let plugin = AdvancedCalculatorPlugin::new();
    let config = Config::default();
    let context = PluginContext::new(10, &config);

    // Test current time in UTC
    let results = plugin
        .search("now in UTC", &context)
        .expect("search failed");
    assert!(
        !results.is_empty(),
        "Should return results for 'now in UTC'"
    );
    assert!(
        results.iter().any(|r| r.title.contains("UTC")),
        "Should contain UTC time"
    );
}

#[test]
fn test_plugin_metadata() {
    let plugin = AdvancedCalculatorPlugin::new();

    assert_eq!(plugin.name(), "advanced_calculator");
    assert!(plugin.enabled());
    assert_eq!(plugin.priority(), 850);

    let prefixes = plugin.command_prefixes();
    assert!(prefixes.contains(&"@calc"));
    assert!(prefixes.contains(&"@convert"));
    assert!(prefixes.contains(&"@time"));
    assert!(prefixes.contains(&"@currency"));
}

#[test]
fn test_should_handle() {
    let plugin = AdvancedCalculatorPlugin::new();

    // Should handle time queries
    assert!(plugin.should_handle("1 hour ago"));
    assert!(plugin.should_handle("in 5 days"));

    // Should handle conversions
    assert!(plugin.should_handle("100 km to miles"));
    assert!(plugin.should_handle("150 days to years"));

    // Should handle timezone queries
    assert!(plugin.should_handle("now in UTC"));

    // Should handle command prefixes
    assert!(plugin.should_handle("@calc 5+5"));
    assert!(plugin.should_handle("@convert 5 km to miles"));
    assert!(plugin.should_handle("@time 1 hour ago"));
    assert!(plugin.should_handle("@currency 100 USD to EUR"));

    // Should not handle unrelated queries
    assert!(!plugin.should_handle("firefox"));
    assert!(!plugin.should_handle("hello world"));
}
