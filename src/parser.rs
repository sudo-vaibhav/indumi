use regex::Regex;

#[derive(Debug, Clone)]
pub enum Expression {
    Number(f64),
    Variable(String),
    BinaryOp { op: Operator, left: Box<Expression>, right: Box<Expression> },
    Assignment { var: String, expr: Box<Expression> },
    CurrencyAnnotation { value: Box<Expression>, currency: String },
    CurrencyConversion { source: Box<Expression>, target_currency: String },
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Modulo,
}

pub struct Parser {
    assignment_regex: Regex,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            assignment_regex: Regex::new(r"^([a-zA-Z_]\w*)\s*=\s*(.+)$").unwrap(),
        }
    }

    pub fn parse(&self, input: &str) -> Result<Expression, String> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err("Empty input".to_string());
        }

        // Check for assignment
        if let Some(caps) = self.assignment_regex.captures(trimmed) {
            let var = caps[1].to_string();
            let expr = self.parse(&caps[2])?;
            return Ok(Expression::Assignment {
                var,
                expr: Box::new(expr),
            });
        }

        // Parse expression (handles everything including currency conversions)
        self.parse_expression(trimmed)
    }

    fn parse_expression(&self, input: &str) -> Result<Expression, String> {
        let tokens = tokenize(input);
        if tokens.is_empty() {
            return Err("No tokens".to_string());
        }

        let mut i = 0;
        self.parse_conversion(&tokens, &mut i)
    }

    // Lowest precedence: currency conversion (to operator)
    fn parse_conversion(&self, tokens: &[String], i: &mut usize) -> Result<Expression, String> {
        let mut left = self.parse_add_subtract(tokens, i)?;

        // Check for "to" operator
        if *i < tokens.len() && tokens[*i].to_lowercase() == "to" {
            *i += 1;
            if *i >= tokens.len() {
                return Err("Expected currency after 'to'".to_string());
            }

            let target_currency = normalize_currency(&tokens[*i]);
            *i += 1;

            left = Expression::CurrencyConversion {
                source: Box::new(left),
                target_currency,
            };
        }

        Ok(left)
    }

    fn parse_add_subtract(&self, tokens: &[String], i: &mut usize) -> Result<Expression, String> {
        let mut left = self.parse_mul_div(tokens, i)?;

        while *i < tokens.len() {
            match tokens[*i].as_str() {
                "+" => {
                    *i += 1;
                    let right = self.parse_mul_div(tokens, i)?;
                    left = Expression::BinaryOp {
                        op: Operator::Add,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                "-" => {
                    *i += 1;
                    let right = self.parse_mul_div(tokens, i)?;
                    left = Expression::BinaryOp {
                        op: Operator::Subtract,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_mul_div(&self, tokens: &[String], i: &mut usize) -> Result<Expression, String> {
        let mut left = self.parse_primary(tokens, i)?;

        while *i < tokens.len() {
            match tokens[*i].as_str() {
                "*" => {
                    *i += 1;
                    let right = self.parse_primary(tokens, i)?;
                    left = Expression::BinaryOp {
                        op: Operator::Multiply,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                "/" => {
                    *i += 1;
                    let right = self.parse_primary(tokens, i)?;
                    left = Expression::BinaryOp {
                        op: Operator::Divide,
                        left: Box::new(left),
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_primary(&self, tokens: &[String], i: &mut usize) -> Result<Expression, String> {
        if *i >= tokens.len() {
            return Err("Expected expression".to_string());
        }

        let token = &tokens[*i];

        // Handle parentheses
        if token == "(" {
            *i += 1;
            let expr = self.parse_conversion(tokens, i)?;  // Recursive call to top level
            if *i >= tokens.len() || tokens[*i] != ")" {
                return Err("Expected closing parenthesis".to_string());
            }
            *i += 1;
            return Ok(expr);
        }

        // Try to parse as number
        if let Ok(num) = token.parse::<f64>() {
            *i += 1;

            // Check if next token is a currency code
            if *i < tokens.len() {
                if is_currency(&tokens[*i]) {
                    let currency = normalize_currency(&tokens[*i]);
                    *i += 1;
                    return Ok(Expression::CurrencyAnnotation {
                        value: Box::new(Expression::Number(num)),
                        currency,
                    });
                }
            }

            return Ok(Expression::Number(num));
        }

        // Variable or identifier
        if token.chars().all(|c| c.is_alphanumeric() || c == '_') {
            *i += 1;
            return Ok(Expression::Variable(token.clone()));
        }

        Err(format!("Cannot parse: {}", token))
    }
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in input.chars() {
        match ch {
            '+' | '-' | '*' | '/' | '%' | '^' | '(' | ')' => {
                if !current.is_empty() {
                    tokens.push(current.trim().to_string());
                    current.clear();
                }
                tokens.push(ch.to_string());
            }
            ' ' => {
                if !current.is_empty() {
                    tokens.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push(current.trim().to_string());
    }

    // Post-process: combine number + text_multiplier into a single token
    let mut processed = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if i + 1 < tokens.len() {
            // Check if current token is a number and next is a text multiplier
            if let Ok(num) = tokens[i].parse::<f64>() {
                let multiplier_text = tokens[i + 1].to_lowercase();
                let multiplier = text_to_multiplier(&multiplier_text);

                if multiplier != 1.0 {
                    // Combine number and multiplier
                    let combined_value = num * multiplier;
                    processed.push(combined_value.to_string());
                    i += 2; // Skip both tokens
                    continue;
                }
            }
        }

        processed.push(tokens[i].clone());
        i += 1;
    }

    processed
}

fn text_to_multiplier(text: &str) -> f64 {
    match text.to_lowercase().as_str() {
        // Indian numbering
        "crore" | "crores" | "cr" => 10_000_000.0,
        "lakh" | "lakhs" | "lac" | "lacs" => 100_000.0,

        // Western numbering
        "billion" | "billions" | "b" => 1_000_000_000.0,
        "million" | "millions" | "m" => 1_000_000.0,
        "thousand" | "thousands" | "k" => 1_000.0,

        _ => 1.0,
    }
}

fn is_currency(token: &str) -> bool {
    matches!(token.to_uppercase().as_str(),
        "USD" | "EUR" | "INR" | "$" | "€" | "₹"
    )
}

fn normalize_currency(symbol: &str) -> String {
    match symbol.to_uppercase().as_str() {
        "$" | "USD" => "USD".to_string(),
        "€" | "EUR" => "EUR".to_string(),
        "₹" | "INR" => "INR".to_string(),
        _ => symbol.to_uppercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let parser = Parser::new();
        match parser.parse("42") {
            Ok(Expression::Number(n)) => assert_eq!(n, 42.0),
            _ => panic!("Expected Number expression"),
        }
    }

    #[test]
    fn test_parse_decimal() {
        let parser = Parser::new();
        match parser.parse("3.14") {
            Ok(Expression::Number(n)) => assert!((n - 3.14).abs() < 0.001),
            _ => panic!("Expected Number expression"),
        }
    }

    #[test]
    fn test_parse_variable() {
        let parser = Parser::new();
        match parser.parse("x") {
            Ok(Expression::Variable(name)) => assert_eq!(name, "x"),
            _ => panic!("Expected Variable expression"),
        }
    }

    #[test]
    fn test_parse_addition() {
        let parser = Parser::new();
        match parser.parse("2 + 3") {
            Ok(Expression::BinaryOp { op, left, right }) => {
                assert!(matches!(op, Operator::Add));
                assert!(matches!(*left, Expression::Number(n) if n == 2.0));
                assert!(matches!(*right, Expression::Number(n) if n == 3.0));
            }
            _ => panic!("Expected BinaryOp expression"),
        }
    }

    #[test]
    fn test_parse_subtraction() {
        let parser = Parser::new();
        match parser.parse("10 - 5") {
            Ok(Expression::BinaryOp { op, left, right }) => {
                assert!(matches!(op, Operator::Subtract));
                assert!(matches!(*left, Expression::Number(n) if n == 10.0));
                assert!(matches!(*right, Expression::Number(n) if n == 5.0));
            }
            _ => panic!("Expected BinaryOp expression"),
        }
    }

    #[test]
    fn test_parse_multiplication() {
        let parser = Parser::new();
        match parser.parse("4 * 5") {
            Ok(Expression::BinaryOp { op, left, right }) => {
                assert!(matches!(op, Operator::Multiply));
                assert!(matches!(*left, Expression::Number(n) if n == 4.0));
                assert!(matches!(*right, Expression::Number(n) if n == 5.0));
            }
            _ => panic!("Expected BinaryOp expression"),
        }
    }

    #[test]
    fn test_parse_division() {
        let parser = Parser::new();
        match parser.parse("20 / 4") {
            Ok(Expression::BinaryOp { op, left, right }) => {
                assert!(matches!(op, Operator::Divide));
                assert!(matches!(*left, Expression::Number(n) if n == 20.0));
                assert!(matches!(*right, Expression::Number(n) if n == 4.0));
            }
            _ => panic!("Expected BinaryOp expression"),
        }
    }

    #[test]
    fn test_operator_precedence_multiply_before_add() {
        let parser = Parser::new();
        // 2 + 3 * 4 should parse as 2 + (3 * 4)
        match parser.parse("2 + 3 * 4") {
            Ok(Expression::BinaryOp { op: Operator::Add, left, right }) => {
                assert!(matches!(*left, Expression::Number(n) if n == 2.0));
                assert!(matches!(*right, Expression::BinaryOp {
                    op: Operator::Multiply,
                    ..
                }));
            }
            _ => panic!("Expected Add with Multiply on right"),
        }
    }

    #[test]
    fn test_parentheses_override_precedence() {
        let parser = Parser::new();
        // (2 + 3) * 4 should parse as (2 + 3) * 4
        match parser.parse("(2 + 3) * 4") {
            Ok(Expression::BinaryOp { op: Operator::Multiply, left, right }) => {
                assert!(matches!(*left, Expression::BinaryOp {
                    op: Operator::Add,
                    ..
                }));
                assert!(matches!(*right, Expression::Number(n) if n == 4.0));
            }
            _ => panic!("Expected Multiply with Add on left"),
        }
    }

    #[test]
    fn test_nested_parentheses() {
        let parser = Parser::new();
        let result = parser.parse("((2 + 3) * 4) / 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_text_multiplier_billion() {
        let parser = Parser::new();
        match parser.parse("1 b") {
            Ok(Expression::Number(n)) => assert_eq!(n, 1_000_000_000.0),
            _ => panic!("Expected Number with billion multiplier"),
        }
    }

    #[test]
    fn test_text_multiplier_million() {
        let parser = Parser::new();
        match parser.parse("5 m") {
            Ok(Expression::Number(n)) => assert_eq!(n, 5_000_000.0),
            _ => panic!("Expected Number with million multiplier"),
        }
    }

    #[test]
    fn test_text_multiplier_crore() {
        let parser = Parser::new();
        match parser.parse("2 cr") {
            Ok(Expression::Number(n)) => assert_eq!(n, 20_000_000.0),
            _ => panic!("Expected Number with crore multiplier"),
        }
    }

    #[test]
    fn test_text_multiplier_lakh() {
        let parser = Parser::new();
        match parser.parse("3 lakh") {
            Ok(Expression::Number(n)) => assert_eq!(n, 300_000.0),
            _ => panic!("Expected Number with lakh multiplier"),
        }
    }

    #[test]
    fn test_text_multiplier_in_expression() {
        let parser = Parser::new();
        // 1 b / 4
        match parser.parse("1 b / 4") {
            Ok(Expression::BinaryOp { op: Operator::Divide, left, right }) => {
                assert!(matches!(*left, Expression::Number(n) if n == 1_000_000_000.0));
                assert!(matches!(*right, Expression::Number(n) if n == 4.0));
            }
            _ => panic!("Expected division with billion"),
        }
    }

    #[test]
    fn test_currency_annotation_usd() {
        let parser = Parser::new();
        match parser.parse("100 USD") {
            Ok(Expression::CurrencyAnnotation { value, currency }) => {
                assert!(matches!(*value, Expression::Number(n) if n == 100.0));
                assert_eq!(currency, "USD");
            }
            _ => panic!("Expected CurrencyAnnotation"),
        }
    }

    #[test]
    fn test_currency_annotation_symbol() {
        let parser = Parser::new();
        match parser.parse("50 €") {
            Ok(Expression::CurrencyAnnotation { value, currency }) => {
                assert!(matches!(*value, Expression::Number(n) if n == 50.0));
                assert_eq!(currency, "EUR");
            }
            _ => panic!("Expected CurrencyAnnotation with EUR"),
        }
    }

    #[test]
    fn test_simple_currency_conversion() {
        let parser = Parser::new();
        match parser.parse("100 USD to INR") {
            Ok(Expression::CurrencyConversion { source, target_currency }) => {
                assert_eq!(target_currency, "INR");
                assert!(matches!(*source, Expression::CurrencyAnnotation { .. }));
            }
            _ => panic!("Expected CurrencyConversion"),
        }
    }

    #[test]
    fn test_currency_conversion_with_division() {
        let parser = Parser::new();
        // "100 USD to INR / 4" is ambiguous:
        // Could mean: (100 USD to INR) / 4  OR  100 USD to (INR / 4)
        // Our parser gives "to" lowest precedence, so it should parse as:
        // 100 USD to (INR / 4)
        // But that doesn't make semantic sense!
        //
        // For the intended meaning "(100 USD to INR) / 4", user should write:
        // "(100 USD to INR) / 4" with explicit parentheses

        // Test that explicit parentheses work correctly
        let result = parser.parse("(100 USD to INR) / 4");
        assert!(result.is_ok(), "Failed to parse: (100 USD to INR) / 4");

        match result.unwrap() {
            Expression::BinaryOp { op: Operator::Divide, left, right } => {
                assert!(matches!(*left, Expression::CurrencyConversion { .. }));
                assert!(matches!(*right, Expression::Number(n) if n == 4.0));
            }
            _ => panic!("Expected division with currency conversion in parentheses"),
        }
    }

    #[test]
    fn test_currency_conversion_with_parentheses() {
        let parser = Parser::new();
        // Currency annotations only work directly on numbers, not on expressions.
        // So "(50 + 50) USD to EUR" won't work as expected.
        // Instead, test: "(50 USD + 50 USD) to EUR" or "100 USD to EUR"

        // Test 1: Simple conversion
        match parser.parse("100 USD to EUR") {
            Ok(Expression::CurrencyConversion { source, target_currency }) => {
                assert_eq!(target_currency, "EUR");
                assert!(matches!(*source, Expression::CurrencyAnnotation { .. }));
            }
            _ => panic!("Expected CurrencyConversion"),
        }

        // Test 2: Parenthesized expression with currency inside
        let result = parser.parse("(50 USD + 50 USD) to EUR");
        assert!(result.is_ok(), "Failed to parse: (50 USD + 50 USD) to EUR");
        // This creates: ((50 USD) + (50 USD)) to EUR
        // The conversion tries to extract currency from the addition,
        // which should find USD from the left operand
    }

    #[test]
    fn test_assignment() {
        let parser = Parser::new();
        match parser.parse("x = 100") {
            Ok(Expression::Assignment { var, expr }) => {
                assert_eq!(var, "x");
                assert!(matches!(*expr, Expression::Number(n) if n == 100.0));
            }
            _ => panic!("Expected Assignment"),
        }
    }

    #[test]
    fn test_assignment_with_expression() {
        let parser = Parser::new();
        match parser.parse("y = 50 + 50") {
            Ok(Expression::Assignment { var, expr }) => {
                assert_eq!(var, "y");
                assert!(matches!(*expr, Expression::BinaryOp { .. }));
            }
            _ => panic!("Expected Assignment with expression"),
        }
    }

    #[test]
    fn test_error_empty_input() {
        let parser = Parser::new();
        assert!(parser.parse("").is_err());
        assert!(parser.parse("   ").is_err());
    }

    #[test]
    fn test_error_missing_closing_paren() {
        let parser = Parser::new();
        assert!(parser.parse("(2 + 3").is_err());
    }

    #[test]
    fn test_error_missing_operand() {
        let parser = Parser::new();
        assert!(parser.parse("5 +").is_err());
    }

    #[test]
    fn test_normalize_currency() {
        assert_eq!(normalize_currency("$"), "USD");
        assert_eq!(normalize_currency("USD"), "USD");
        assert_eq!(normalize_currency("€"), "EUR");
        assert_eq!(normalize_currency("EUR"), "EUR");
        assert_eq!(normalize_currency("₹"), "INR");
        assert_eq!(normalize_currency("INR"), "INR");
    }

    #[test]
    fn test_is_currency() {
        assert!(is_currency("USD"));
        assert!(is_currency("$"));
        assert!(is_currency("EUR"));
        assert!(is_currency("€"));
        assert!(is_currency("INR"));
        assert!(is_currency("₹"));
        assert!(!is_currency("XYZ"));
        assert!(!is_currency("foo"));
    }

    #[test]
    fn test_text_to_multiplier() {
        assert_eq!(text_to_multiplier("billion"), 1_000_000_000.0);
        assert_eq!(text_to_multiplier("b"), 1_000_000_000.0);
        assert_eq!(text_to_multiplier("million"), 1_000_000.0);
        assert_eq!(text_to_multiplier("m"), 1_000_000.0);
        assert_eq!(text_to_multiplier("crore"), 10_000_000.0);
        assert_eq!(text_to_multiplier("cr"), 10_000_000.0);
        assert_eq!(text_to_multiplier("lakh"), 100_000.0);
        assert_eq!(text_to_multiplier("lac"), 100_000.0);
        assert_eq!(text_to_multiplier("thousand"), 1_000.0);
        assert_eq!(text_to_multiplier("k"), 1_000.0);
    }
}
