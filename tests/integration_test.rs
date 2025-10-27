// Integration tests for Indumi calculator
// These tests exercise the full system: parsing + evaluation + formatting

use indumi::calc::Calculator;

async fn test_expression(expr: &str, expected_contains: &[&str], expected_not_contains: &[&str]) {
    let mut calc = Calculator::new().await.expect("Failed to create calculator");
    let result = calc.evaluate_line(expr);

    assert!(result.is_some(), "Expression '{}' returned None", expr);

    let output = result.unwrap();
    println!("Expression: {} => {}", expr, output);

    for pattern in expected_contains {
        assert!(
            output.contains(pattern),
            "Output '{}' should contain '{}'",
            output,
            pattern
        );
    }

    for pattern in expected_not_contains {
        assert!(
            !output.contains(pattern),
            "Output '{}' should NOT contain '{}'",
            output,
            pattern
        );
    }
}

#[tokio::test]
async fn test_basic_arithmetic() {
    test_expression("2 + 3", &["5"], &["Error"]).await;
    test_expression("10 - 5", &["5"], &["Error"]).await;
    test_expression("4 * 5", &["20"], &["Error"]).await;
    test_expression("20 / 4", &["5"], &["Error"]).await;
}

#[tokio::test]
async fn test_operator_precedence() {
    test_expression("2 + 3 * 4", &["14"], &["20", "Error"]).await; // Should be 2 + 12 = 14
    test_expression("10 - 2 * 3", &["4"], &["24", "Error"]).await; // Should be 10 - 6 = 4
}

#[tokio::test]
async fn test_parentheses() {
    test_expression("(2 + 3) * 4", &["20"], &["14", "Error"]).await;
    test_expression("2 * (3 + 4)", &["14"], &["10", "Error"]).await;
    test_expression("((2 + 3) * 4) / 2", &["10"], &["Error"]).await;
}

#[tokio::test]
async fn test_text_multipliers() {
    test_expression("1 b", &["1,000,000,000", "1 B"], &["Error"]).await;
    test_expression("5 m", &["5,000,000", "5 M"], &["Error"]).await;
    test_expression("10 k", &["10,000", "10 K"], &["Error"]).await;
    // Note: "2 cr" = 20,000,000 displays as "20 M" (Western style) not "2 Cr"
    // because plain numbers use Western formatting. For Indian formatting, use with INR currency.
    test_expression("2 cr", &["20,000,000"], &["Error"]).await;
    test_expression("3 lakh", &["300,000"], &["Error"]).await;
}

#[tokio::test]
async fn test_text_multipliers_in_expressions() {
    test_expression("1 b / 4", &["250,000,000", "250 M"], &["Error"]).await;
    test_expression("10 k * 3", &["30,000", "30 K"], &["Error"]).await;
    test_expression("1 m + 500 k", &["1,500,000", "1.5 M"], &["Error"]).await;
}

#[tokio::test]
async fn test_number_formatting() {
    // Western formatting
    test_expression("1000", &["1,000"], &["Error"]).await;
    test_expression("1000000", &["1,000,000", "1 M"], &["Error"]).await;

    // Large numbers show estimates
    test_expression("1000000000", &["1,000,000,000", "1 B"], &["Error"]).await;
}

#[tokio::test]
async fn test_currency_conversions() {
    // Simple conversions
    test_expression("100 USD to INR", &["₹"], &["Error"]).await;
    test_expression("100 USD to EUR", &["€"], &["Error"]).await;
    test_expression("1000 INR to USD", &["$"], &["Error"]).await;
}

#[tokio::test]
async fn test_currency_with_parentheses() {
    // Note: After division, currency context is lost (it becomes a plain number)
    test_expression("(100 USD to INR) / 4", &[], &["Error"]).await;
    // But direct conversion works
    test_expression("(50 USD + 50 USD) to EUR", &["€"], &["Error"]).await;
}

#[tokio::test]
async fn test_variables() {
    let mut calc = Calculator::new().await.expect("Failed to create calculator");

    // Set a variable
    let result1 = calc.evaluate_line("x = 100");
    assert!(result1.is_some());
    assert_eq!(result1.unwrap(), "100");

    // Use the variable
    let result2 = calc.evaluate_line("x + 50");
    assert!(result2.is_some());
    assert_eq!(result2.unwrap(), "150");

    // Use in multiplication
    let result3 = calc.evaluate_line("x * 2");
    assert!(result3.is_some());
    assert_eq!(result3.unwrap(), "200");
}

#[tokio::test]
async fn test_variable_with_currency() {
    let mut calc = Calculator::new().await.expect("Failed to create calculator");

    // Convert and store - the variable stores the numeric value
    // Note: Assignment expressions show numeric result, not currency formatted
    let result1 = calc.evaluate_line("converted = 100 USD to INR");
    assert!(result1.is_some());
    let output1 = result1.unwrap();
    // Assignment returns the numeric value (formatted but without currency symbol)
    assert!(!output1.contains("Error"));
    assert!(output1.chars().any(|c| c.is_numeric()));

    // Use the stored value
    let result2 = calc.evaluate_line("converted / 4");
    assert!(result2.is_some());
    // Result should be a number (without currency symbol since variable stores plain number)
    let output2 = result2.unwrap();
    assert!(!output2.contains("Error"));
}

#[tokio::test]
async fn test_complex_real_world_scenarios() {
    // Budget calculation
    test_expression("(1500 + 800 + 300) * 12", &["31,200", "31.2 K"], &["Error"]).await;

    // Large number division
    test_expression("1 b / 1 m", &["1,000"], &["Error"]).await;

    // Mixed operations
    test_expression("(10 k + 5 k) * 2", &["30,000", "30 K"], &["Error"]).await;
}

#[tokio::test]
async fn test_error_cases() {
    let mut calc = Calculator::new().await.expect("Failed to create calculator");

    // Division by zero
    let result = calc.evaluate_line("10 / 0");
    assert!(result.is_some());
    assert!(result.unwrap().contains("Error"));

    // Undefined variable
    let result2 = calc.evaluate_line("undefined_var + 5");
    assert!(result2.is_some());
    assert!(result2.unwrap().contains("Error"));

    // Invalid syntax
    let result3 = calc.evaluate_line("5 +");
    assert!(result3.is_some());
    assert!(result3.unwrap().contains("error"));
}

#[tokio::test]
async fn test_edge_cases() {
    // Very small numbers - note: formatting rounds to 2 decimal places
    // so 0.001 becomes 0
    test_expression("0.001", &["0"], &["Error"]).await;

    // Negative numbers - parser doesn't support unary minus yet
    // Use subtraction instead
    test_expression("0 - 5 + 10", &["5"], &["Error"]).await;
    test_expression("5 - 10", &["-5"], &["Error"]).await;

    // Zero
    test_expression("0", &["0"], &["Error"]).await;
}

#[tokio::test]
async fn test_decimal_precision() {
    test_expression("10 / 3", &["3.33"], &["Error"]).await;
    test_expression("1.5 * 2.5", &["3.75"], &["Error"]).await;
}

#[tokio::test]
async fn test_mixed_currency_and_math() {
    // Currency annotation in arithmetic
    test_expression("50 USD + 50 USD", &["100"], &["Error"]).await;

    // Large currency conversion with division
    // Note: After division, currency context is lost
    test_expression("(1 b INR to USD) / 4", &[], &["Error"]).await;

    // Direct currency conversion maintains currency
    test_expression("1 b INR to USD", &["$"], &["Error"]).await;
}

#[tokio::test]
async fn test_empty_and_whitespace() {
    let mut calc = Calculator::new().await.expect("Failed to create calculator");

    assert!(calc.evaluate_line("").is_none());
    assert!(calc.evaluate_line("   ").is_none());
    assert!(calc.evaluate_line("\t\n").is_none());
}

#[tokio::test]
async fn test_sequential_calculations() {
    let mut calc = Calculator::new().await.expect("Failed to create calculator");

    // Multiple calculations in sequence
    calc.evaluate_line("a = 10");
    calc.evaluate_line("b = 20");
    calc.evaluate_line("c = a + b");

    let result = calc.evaluate_line("c * 2");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), "60");
}
