// Standalone test for the parser and calculator
// Run with: rustc test_parser.rs && ./test_parser

use std::collections::HashMap;

mod parser {
    include!("src/parser.rs");
}

mod currency {
    include!("src/currency.rs");
}

mod calc {
    include!("src/calc.rs");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing new parser with complex expressions:\n");

    let mut calculator = calc::Calculator::new().await?;

    let test_cases = vec![
        // Basic math
        ("100 + 50", "Basic addition"),
        ("100 * 5", "Multiplication"),
        ("1000 / 4", "Division"),

        // Parentheses
        ("(50 + 50) * 2", "Parentheses precedence"),
        ("100 * (2 + 3)", "Parentheses multiplication"),

        // Text-based numbers
        ("1 b / 4", "Text-based billion division"),
        ("10 k * 3", "Text-based thousand multiplication"),

        // Currency conversions
        ("100 USD to INR", "Simple currency conversion"),
        ("1000 INR to EUR", "INR to EUR conversion"),

        // Complex expressions
        ("100 USD to INR / 4", "Currency conversion with division"),
        ("1 b INR to USD / 4", "Large number conversion with division"),
    ];

    for (expr, description) in test_cases {
        println!("Testing: {} ({})", expr, description);

        match calculator.evaluate_line(expr) {
            Some(result) => println!("  Result: {}", result),
            None => println!("  No result"),
        }
        println!();
    }

    Ok(())
}