use super::traits::{Plugin, PluginContext, PluginResult};
use anyhow::Result;

/// Plugin for evaluating mathematical expressions
#[derive(Debug)]
pub struct CalculatorPlugin {
    enabled: bool,
}

impl CalculatorPlugin {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// Check if query looks like a math expression
    fn is_math_expression(query: &str) -> bool {
        // Check for common math operators and numbers
        let has_numbers = query.chars().any(|c| c.is_ascii_digit());
        let has_operators = query
            .chars()
            .any(|c| matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '^' | '%'));

        has_numbers && (has_operators || query.parse::<f64>().is_ok())
    }

    /// Evaluate a math expression
    fn evaluate(&self, expr: &str) -> Result<f64> {
        // Simple evaluation using meval-rs or similar
        // For now, use a basic implementation
        let mut expr = expr.trim().to_string();

        // Lightweight handling for sqrt(...) by recursively evaluating the inner expression
        // and replacing occurrences with their computed values before final evaluation.
        // This avoids pulling in heavy parser dependencies while covering common cases.
        expr = Self::preprocess_sqrt(&expr, |inner| self.evaluate(inner))?;

        // Try parsing as number first
        if let Ok(num) = expr.parse::<f64>() {
            return Ok(num);
        }

        // Use evalexpr crate for safe expression evaluation
        match evalexpr::eval(&expr) {
            Ok(value) => {
                if let evalexpr::Value::Float(f) = value {
                    Ok(f)
                } else if let evalexpr::Value::Int(i) = value {
                    Ok(i as f64)
                } else {
                    Err(anyhow::anyhow!("Invalid result type"))
                }
            }
            Err(e) => Err(anyhow::anyhow!("Evaluation error: {}", e)),
        }
    }

    /// Preprocess sqrt(...) patterns by evaluating inner expressions and replacing them.
    /// The callback `eval_inner` is used to compute the value of the inner expression.
    fn preprocess_sqrt<F>(input: &str, mut eval_inner: F) -> Result<String>
    where
        F: FnMut(&str) -> Result<f64>,
    {
        let mut s = input.to_string();
        loop {
            if let Some(start) = s.find("sqrt(") {
                // Find matching closing parenthesis
                let mut depth = 0usize;
                let mut end_opt = None;
                for (i, ch) in s[start + 5..].char_indices() {
                    let idx = start + 5 + i;
                    if ch == '(' {
                        depth += 1;
                    } else if ch == ')' {
                        if depth == 0 {
                            end_opt = Some(idx);
                            break;
                        } else {
                            depth -= 1;
                        }
                    }
                }

                let end = match end_opt {
                    Some(e) => e,
                    None => break, // Unbalanced, give up preprocessing
                };

                let inner = &s[start + 5..end]; // contents inside sqrt(...)
                let value = eval_inner(inner)?;
                if value < 0.0 {
                    return Err(anyhow::anyhow!("sqrt of negative value"));
                }
                let replacement = format!("{}", value.sqrt());
                s.replace_range(start..=end, &replacement);
            } else {
                break;
            }
        }
        Ok(s)
    }
}

impl Default for CalculatorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for CalculatorPlugin {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Evaluate mathematical expressions"
    }

    fn command_prefixes(&self) -> Vec<&str> {
        vec!["@cal"]
    }

    fn should_handle(&self, query: &str) -> bool {
        self.enabled && Self::is_math_expression(query)
    }

    fn search(&self, query: &str, _context: &PluginContext) -> Result<Vec<PluginResult>> {
        if !self.enabled || !Self::is_math_expression(query) {
            return Ok(vec![]);
        }

        match self.evaluate(query) {
            Ok(result) => {
                let formatted = if result.fract() == 0.0 {
                    format!("{:.0}", result)
                } else {
                    format!("{:.6}", result).trim_end_matches('0').to_string()
                };

                Ok(vec![PluginResult::new(
                    formatted.clone(),
                    format!("echo '{}'", formatted), // Copy to clipboard would be better
                    self.name().to_string(),
                )
                .with_subtitle(format!("= {}", query))
                .with_icon("accessories-calculator".to_string())
                .with_score(10000)]) // High score to show above app results
            }
            Err(_) => Ok(vec![]), // Invalid expression, no results
        }
    }

    fn priority(&self) -> i32 {
        500 // High priority for calculator
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_math_expression() {
        assert!(CalculatorPlugin::is_math_expression("2+2"));
        assert!(CalculatorPlugin::is_math_expression("15*3"));
        assert!(CalculatorPlugin::is_math_expression("(10+5)/3"));
        assert!(!CalculatorPlugin::is_math_expression("firefox"));
        assert!(!CalculatorPlugin::is_math_expression("test"));
    }

    #[test]
    fn test_evaluate() {
        let calc = CalculatorPlugin::new();
        assert_eq!(calc.evaluate("2+2").unwrap(), 4.0);
        assert_eq!(calc.evaluate("10*5").unwrap(), 50.0);
        assert_eq!(calc.evaluate("100/4").unwrap(), 25.0);
    }

    #[test]
    fn test_search() {
        use crate::config::Config;

        let calc = CalculatorPlugin::new();
        let config = Config::default();
        let ctx = PluginContext::new(10, &config);

        let results = calc.search("2+2", &ctx).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "4");
    }
}
