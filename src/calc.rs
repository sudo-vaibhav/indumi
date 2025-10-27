use std::collections::HashMap;
use crate::parser::{Expression, Operator};
use crate::currency::CurrencyConverter;

#[derive(Debug)]
pub struct Calculator {
    variables: HashMap<String, f64>,
    converter: CurrencyConverter,
}

impl Calculator {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let converter = CurrencyConverter::new().await?;
        Ok(Self {
            variables: HashMap::new(),
            converter,
        })
    }

    pub fn evaluate(&mut self, expr: &Expression) -> Result<f64, String> {
        match expr {
            Expression::Number(n) => Ok(*n),

            Expression::Variable(name) => {
                self.variables
                    .get(name)
                    .copied()
                    .ok_or_else(|| format!("Undefined variable: {}", name))
            }

            Expression::CurrencyAnnotation { value, .. } => {
                // Currency annotation just evaluates the inner value
                // The currency info is metadata used by CurrencyConversion
                self.evaluate(value)
            }

            Expression::CurrencyConversion { source, target_currency } => {
                // First evaluate the source to get the amount
                let amount = self.evaluate(source)?;

                // Extract the source currency from the expression
                let source_currency = self.extract_currency(source)?;

                // Convert from source to target currency
                self.converter.convert(amount, &source_currency, target_currency)
            }

            Expression::BinaryOp { op, left, right } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;

                match op {
                    Operator::Add => Ok(left_val + right_val),
                    Operator::Subtract => Ok(left_val - right_val),
                    Operator::Multiply => Ok(left_val * right_val),
                    Operator::Divide => {
                        if right_val == 0.0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(left_val / right_val)
                        }
                    }
                    Operator::Power => Ok(left_val.powf(right_val)),
                    Operator::Modulo => Ok(left_val % right_val),
                }
            }

            Expression::Assignment { var, expr } => {
                let value = self.evaluate(expr)?;
                self.variables.insert(var.clone(), value);
                Ok(value)
            }
        }
    }

    fn extract_currency(&self, expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::CurrencyAnnotation { currency, .. } => Ok(currency.clone()),
            Expression::BinaryOp { left, .. } => {
                // Try left side first, then right side
                self.extract_currency(left)
            }
            _ => Err("Expression does not have a currency annotation".to_string())
        }
    }

    pub fn evaluate_line(&mut self, line: &str) -> Option<String> {
        if line.trim().is_empty() {
            return None;
        }

        let parser = crate::parser::Parser::new();
        match parser.parse(line) {
            Ok(expr) => {
                // Check if this is a currency conversion to format with currency unit
                let target_currency = match &expr {
                    Expression::CurrencyConversion { target_currency, .. } => Some(target_currency.as_str()),
                    _ => None,
                };

                match self.evaluate(&expr) {
                    Ok(result) => {
                        if let Some(currency) = target_currency {
                            Some(format_currency(result, currency))
                        } else {
                            Some(format_number(result))
                        }
                    }
                    Err(e) => Some(format!("Error: {}", e)),
                }
            }
            Err(e) => Some(format!("Parse error: {}", e)),
        }
    }
}

fn format_number(value: f64) -> String {
    let formatted = format_with_separator(value, false);
    let estimation = estimate_number(value, false);

    if let Some(est) = estimation {
        format!("{} ({})", formatted, est)
    } else {
        formatted
    }
}

fn format_currency(value: f64, currency: &str) -> String {
    let is_indian = currency == "INR";
    let formatted = format_with_separator(value, is_indian);
    let estimation = estimate_number(value, is_indian);

    let symbol = match currency {
        "USD" => "$",
        "EUR" => "€",
        "INR" => "₹",
        _ => currency,
    };

    if let Some(est) = estimation {
        format!("{} {} ({})", symbol, formatted, est)
    } else {
        format!("{} {}", symbol, formatted)
    }
}

fn estimate_number(value: f64, indian_style: bool) -> Option<String> {
    let abs_value = value.abs();

    // Don't show estimation for numbers less than 1000
    if abs_value < 1_000.0 {
        return None;
    }

    if indian_style {
        // Indian notation: Crore, Lakh, Thousand
        if abs_value >= 10_000_000.0 {
            let crores = abs_value / 10_000_000.0;
            Some(format!("{:.1} Cr", crores).replace(".0", ""))
        } else if abs_value >= 100_000.0 {
            let lakhs = abs_value / 100_000.0;
            Some(format!("{:.1} Lac", lakhs).replace(".0", ""))
        } else {
            let thousands = abs_value / 1_000.0;
            Some(format!("{:.1} K", thousands).replace(".0", ""))
        }
    } else {
        // Western notation: Billion, Million, Thousand
        if abs_value >= 1_000_000_000.0 {
            let billions = abs_value / 1_000_000_000.0;
            Some(format!("{:.1} B", billions).replace(".0", ""))
        } else if abs_value >= 1_000_000.0 {
            let millions = abs_value / 1_000_000.0;
            Some(format!("{:.1} M", millions).replace(".0", ""))
        } else {
            let thousands = abs_value / 1_000.0;
            Some(format!("{:.1} K", thousands).replace(".0", ""))
        }
    }
}

fn format_with_separator(value: f64, indian_style: bool) -> String {
    let is_negative = value < 0.0;
    let abs_value = value.abs();

    // Split into integer and decimal parts
    let integer_part = abs_value.floor() as i64;
    let decimal_part = ((abs_value - abs_value.floor()) * 100.0).round() as i64;

    let integer_str = if indian_style {
        format_indian_number(integer_part)
    } else {
        format_western_number(integer_part)
    };

    let sign = if is_negative { "-" } else { "" };

    if decimal_part > 0 {
        format!("{}{}.{:02}", sign, integer_str, decimal_part)
    } else {
        format!("{}{}", sign, integer_str)
    }
}

fn format_western_number(n: i64) -> String {
    let s = n.to_string();
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();

    for (i, ch) in chars.iter().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }

    result.chars().rev().collect()
}

fn format_indian_number(n: i64) -> String {
    let s = n.to_string();
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();

    for (i, ch) in chars.iter().rev().enumerate() {
        if i == 3 {
            result.push(',');
        } else if i > 3 && (i - 3) % 2 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }

    result.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Expression, Operator};

    async fn create_test_calculator() -> Calculator {
        Calculator::new().await.expect("Failed to create calculator")
    }

    #[tokio::test]
    async fn test_evaluate_number() {
        let mut calc = create_test_calculator().await;
        let expr = Expression::Number(42.0);
        assert_eq!(calc.evaluate(&expr).unwrap(), 42.0);
    }

    #[tokio::test]
    async fn test_evaluate_addition() {
        let mut calc = create_test_calculator().await;
        let expr = Expression::BinaryOp {
            op: Operator::Add,
            left: Box::new(Expression::Number(2.0)),
            right: Box::new(Expression::Number(3.0)),
        };
        assert_eq!(calc.evaluate(&expr).unwrap(), 5.0);
    }

    #[tokio::test]
    async fn test_evaluate_subtraction() {
        let mut calc = create_test_calculator().await;
        let expr = Expression::BinaryOp {
            op: Operator::Subtract,
            left: Box::new(Expression::Number(10.0)),
            right: Box::new(Expression::Number(3.0)),
        };
        assert_eq!(calc.evaluate(&expr).unwrap(), 7.0);
    }

    #[tokio::test]
    async fn test_evaluate_multiplication() {
        let mut calc = create_test_calculator().await;
        let expr = Expression::BinaryOp {
            op: Operator::Multiply,
            left: Box::new(Expression::Number(4.0)),
            right: Box::new(Expression::Number(5.0)),
        };
        assert_eq!(calc.evaluate(&expr).unwrap(), 20.0);
    }

    #[tokio::test]
    async fn test_evaluate_division() {
        let mut calc = create_test_calculator().await;
        let expr = Expression::BinaryOp {
            op: Operator::Divide,
            left: Box::new(Expression::Number(20.0)),
            right: Box::new(Expression::Number(4.0)),
        };
        assert_eq!(calc.evaluate(&expr).unwrap(), 5.0);
    }

    #[tokio::test]
    async fn test_evaluate_division_by_zero() {
        let mut calc = create_test_calculator().await;
        let expr = Expression::BinaryOp {
            op: Operator::Divide,
            left: Box::new(Expression::Number(10.0)),
            right: Box::new(Expression::Number(0.0)),
        };
        assert!(calc.evaluate(&expr).is_err());
    }

    #[tokio::test]
    async fn test_evaluate_variable_assignment() {
        let mut calc = create_test_calculator().await;
        let assign = Expression::Assignment {
            var: "x".to_string(),
            expr: Box::new(Expression::Number(100.0)),
        };
        assert_eq!(calc.evaluate(&assign).unwrap(), 100.0);

        // Variable should now be stored
        let var_expr = Expression::Variable("x".to_string());
        assert_eq!(calc.evaluate(&var_expr).unwrap(), 100.0);
    }

    #[tokio::test]
    async fn test_evaluate_undefined_variable() {
        let mut calc = create_test_calculator().await;
        let expr = Expression::Variable("undefined".to_string());
        assert!(calc.evaluate(&expr).is_err());
    }

    #[tokio::test]
    async fn test_evaluate_currency_annotation() {
        let mut calc = create_test_calculator().await;
        let expr = Expression::CurrencyAnnotation {
            value: Box::new(Expression::Number(100.0)),
            currency: "USD".to_string(),
        };
        // Currency annotation just returns the value
        assert_eq!(calc.evaluate(&expr).unwrap(), 100.0);
    }

    #[tokio::test]
    async fn test_evaluate_currency_conversion() {
        let mut calc = create_test_calculator().await;
        let expr = Expression::CurrencyConversion {
            source: Box::new(Expression::CurrencyAnnotation {
                value: Box::new(Expression::Number(100.0)),
                currency: "USD".to_string(),
            }),
            target_currency: "INR".to_string(),
        };
        // Exchange rates are fetched from API, so exact value varies
        // Just check that we get a reasonable positive number
        let result = calc.evaluate(&expr).unwrap();
        assert!(result > 7000.0 && result < 10000.0, "USD to INR rate out of expected range");
    }

    #[tokio::test]
    async fn test_evaluate_complex_expression() {
        let mut calc = create_test_calculator().await;
        // (2 + 3) * 4 = 20
        let expr = Expression::BinaryOp {
            op: Operator::Multiply,
            left: Box::new(Expression::BinaryOp {
                op: Operator::Add,
                left: Box::new(Expression::Number(2.0)),
                right: Box::new(Expression::Number(3.0)),
            }),
            right: Box::new(Expression::Number(4.0)),
        };
        assert_eq!(calc.evaluate(&expr).unwrap(), 20.0);
    }

    #[tokio::test]
    async fn test_extract_currency_from_annotation() {
        let calc = create_test_calculator().await;
        let expr = Expression::CurrencyAnnotation {
            value: Box::new(Expression::Number(100.0)),
            currency: "USD".to_string(),
        };
        assert_eq!(calc.extract_currency(&expr).unwrap(), "USD");
    }

    #[tokio::test]
    async fn test_extract_currency_from_binary_op() {
        let calc = create_test_calculator().await;
        // (50 + 50) USD
        let expr = Expression::BinaryOp {
            op: Operator::Add,
            left: Box::new(Expression::CurrencyAnnotation {
                value: Box::new(Expression::Number(50.0)),
                currency: "USD".to_string(),
            }),
            right: Box::new(Expression::Number(50.0)),
        };
        // Should extract USD from left side
        assert_eq!(calc.extract_currency(&expr).unwrap(), "USD");
    }

    #[tokio::test]
    async fn test_evaluate_line_basic() {
        let mut calc = create_test_calculator().await;
        let result = calc.evaluate_line("100 + 50");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "150");
    }

    #[tokio::test]
    async fn test_evaluate_line_with_formatting() {
        let mut calc = create_test_calculator().await;
        let result = calc.evaluate_line("1000000");
        assert!(result.is_some());
        // Should have formatting with comma separators
        assert!(result.unwrap().contains("1,000,000"));
    }

    #[tokio::test]
    async fn test_evaluate_line_currency_conversion() {
        let mut calc = create_test_calculator().await;
        let result = calc.evaluate_line("100 USD to INR");
        assert!(result.is_some());
        let output = result.unwrap();
        eprintln!("Currency conversion output: {}", output);
        // Should have currency symbol and formatting
        assert!(output.contains("₹"));
        // The actual output might vary based on exchange rates fetched
        // Just check that we get a proper number
        assert!(output.chars().any(|c| c.is_numeric()));
    }

    #[tokio::test]
    async fn test_evaluate_line_empty() {
        let mut calc = create_test_calculator().await;
        assert!(calc.evaluate_line("").is_none());
        assert!(calc.evaluate_line("   ").is_none());
    }

    #[test]
    fn test_format_western_number() {
        assert_eq!(format_western_number(1000), "1,000");
        assert_eq!(format_western_number(1000000), "1,000,000");
        assert_eq!(format_western_number(1234567), "1,234,567");
    }

    #[test]
    fn test_format_indian_number() {
        assert_eq!(format_indian_number(1000), "1,000");
        assert_eq!(format_indian_number(100000), "1,00,000");
        assert_eq!(format_indian_number(10000000), "1,00,00,000");
        assert_eq!(format_indian_number(12345678), "1,23,45,678");
    }

    #[test]
    fn test_format_with_separator_western() {
        assert_eq!(format_with_separator(1234.56, false), "1,234.56");
        assert_eq!(format_with_separator(1000000.0, false), "1,000,000");
    }

    #[test]
    fn test_format_with_separator_indian() {
        assert_eq!(format_with_separator(100000.0, true), "1,00,000");
        assert_eq!(format_with_separator(10000000.0, true), "1,00,00,000");
    }

    #[test]
    fn test_format_with_separator_negative() {
        assert_eq!(format_with_separator(-1234.0, false), "-1,234");
        assert_eq!(format_with_separator(-100000.0, true), "-1,00,000");
    }

    #[test]
    fn test_estimate_number_below_threshold() {
        assert_eq!(estimate_number(500.0, false), None);
        assert_eq!(estimate_number(999.0, false), None);
    }

    #[test]
    fn test_estimate_number_thousands() {
        assert_eq!(estimate_number(1000.0, false), Some("1 K".to_string()));
        assert_eq!(estimate_number(5500.0, false), Some("5.5 K".to_string()));
        assert_eq!(estimate_number(10000.0, false), Some("10 K".to_string()));
    }

    #[test]
    fn test_estimate_number_millions() {
        assert_eq!(estimate_number(1000000.0, false), Some("1 M".to_string()));
        assert_eq!(estimate_number(2500000.0, false), Some("2.5 M".to_string()));
    }

    #[test]
    fn test_estimate_number_billions() {
        assert_eq!(estimate_number(1000000000.0, false), Some("1 B".to_string()));
        assert_eq!(estimate_number(3500000000.0, false), Some("3.5 B".to_string()));
    }

    #[test]
    fn test_estimate_number_lakhs() {
        assert_eq!(estimate_number(100000.0, true), Some("1 Lac".to_string()));
        assert_eq!(estimate_number(500000.0, true), Some("5 Lac".to_string()));
    }

    #[test]
    fn test_estimate_number_crores() {
        assert_eq!(estimate_number(10000000.0, true), Some("1 Cr".to_string()));
        assert_eq!(estimate_number(25000000.0, true), Some("2.5 Cr".to_string()));
    }

    #[test]
    fn test_format_currency_usd() {
        let result = format_currency(1234.56, "USD");
        assert!(result.contains("$"));
        assert!(result.contains("1,234.56"));
    }

    #[test]
    fn test_format_currency_inr() {
        let result = format_currency(100000.0, "INR");
        assert!(result.contains("₹"));
        assert!(result.contains("1,00,000"));
    }

    #[test]
    fn test_format_currency_eur() {
        let result = format_currency(5000.0, "EUR");
        assert!(result.contains("€"));
        assert!(result.contains("5,000"));
    }

    #[test]
    fn test_format_number_with_estimate() {
        let result = format_number(1000000.0);
        assert!(result.contains("1,000,000"));
        assert!(result.contains("1 M"));
    }

    #[test]
    fn test_format_number_without_estimate() {
        let result = format_number(500.0);
        assert_eq!(result, "500");
    }
}
