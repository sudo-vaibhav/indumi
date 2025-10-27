# Indumi - AI Coding Instructions

## Project Overview
Indumi is a TUI calculator inspired by Numi and Soulver 3. It provides a text-editor-like interface where users type calculations and see live results on the right panel.

**Core Philosophy**: Simplicity, immediacy, clarity. Every line shows its result instantly.

## Architecture

### Module Structure
```
src/
├── main.rs      - Terminal setup, event loop, quit handling
├── editor.rs    - Text buffer, cursor management, keyboard input
├── parser.rs    - Tokenize and parse expressions into AST
├── calc.rs      - Evaluate expressions, manage variables, format results
├── currency.rs  - Currency conversion with static rates
└── ui.rs        - Ratatui rendering, split-screen layout
```

### Number Formatting

Results are formatted with locale-specific separators and human-readable estimates:

- **Indian numbering (INR)**: `1,00,00,000` - Comma after 3 digits, then every 2 digits
- **Western numbering (USD/EUR)**: `1,000,000` - Comma every 3 digits

Currency conversions show the currency symbol (₹, $, €) followed by the formatted amount.

#### Human-Readable Estimates

For numbers ≥ 1,000, an approximate value is shown in brackets:

- **Indian style (INR)**: Uses Cr (crore), Lac (lakh), K (thousand)
  - Example: `₹ 1,00,00,000 (1 Cr)`
- **Western style (USD/EUR)**: Uses B (billion), M (million), K (thousand)
  - Example: `$ 1,000,000 (1 M)`

Implementation:
- `estimate_number()` in `calc.rs` converts large numbers to human-readable form
- Shows 1 decimal place, removes ".0" for whole numbers
- Returns `None` for values < 1,000

### Text-based Number Multipliers

The parser supports natural language number inputs in currency conversions:

- **Indian units**: crore/crores/cr (10^7), lakh/lakhs/lac/lacs (10^5), thousand/thousands (10^3)
- **Western units**: billion/billions (10^9), million/millions (10^6), thousand/thousands (10^3)
- **Abbreviations**: k (10^3), m (10^6), b (10^9)

Examples: `1 crore INR to USD`, `5 cr INR to EUR`, `2.5 lakh INR to USD`, `10 lac INR to $`

Implementation in `parser.rs`:
- `text_to_multiplier()` converts text to numeric multiplier
- Currency regex captures optional text multiplier between amount and currency
- Final amount = base_amount × multiplier
- **In Regular Expressions**: The `tokenize()` function post-processes tokens to combine number + text_multiplier into a single numeric token
  - Example: "1 b / 4" → tokens ["1", "b", "/", "4"] → processed to ["1000000000", "/", "4"]
  - This allows text-based numbers in any math expression, not just currency conversions

### Data Flow
```
User Input → Editor → Parser → Calculator → UI Renderer
                                    ↓
                              Variable Store
```

## Code Style

- **Direct, no fluff** - Avoid over-engineering
- **Flat structures** - Keep nesting minimal
- **Explicit errors** - Return `Result<T, String>` with clear messages
- **No magic** - Code should be obvious on first read
- **Comments for "why"** - Not "what"

### Naming Conventions
- Functions: `snake_case`, verbs (e.g., `parse_expression`, `handle_key`)
- Structs: `PascalCase`, nouns (e.g., `Editor`, `Calculator`)
- Enums: `PascalCase` for type and variants (e.g., `Operator::Add`)

## Adding Features

### New Mathematical Operators
1. Add variant to `Operator` enum in `parser.rs`
2. Update `tokenize()` to recognize the symbol
3. Implement evaluation in `calc.rs` `BinaryOp` match
4. Update `parse_add_subtract()` or `parse_mul_div()` for precedence

### New Currency
1. Add currency code to `CurrencyConverter::new()` in `currency.rs`
2. Set exchange rate relative to USD
3. Update `normalize_currency()` in `parser.rs` to recognize symbols
4. Add currency symbol mapping in `format_currency()` in `calc.rs`
5. If the currency uses non-Western numbering, update `format_currency()` to set `is_indian = true` for that currency

### New Text Multiplier
1. Add the text pattern to the currency_regex in `Parser::new()` in `parser.rs`
2. Add the case to `text_to_multiplier()` function with its numeric value

### New Expression Types
1. Add variant to `Expression` enum in `parser.rs`
2. Add parsing logic to `Parser::parse()`
3. Add evaluation logic to `Calculator::evaluate()`

## Current Limitations (Future Work)

**Brief overview** - See `docs/tech-debt.md` for detailed analysis, priorities, and roadmap.

- **Parser**: Full expression parsing works. Still need `^`, `%` operators and unary minus (`-5`)
- **Units**: No length, weight, temperature conversions yet
- **File I/O**: No save/load functionality
- **Scrolling**: Large documents don't scroll yet
- **Currency**: Only USD, EUR, INR supported. API requires internet connection.
- **Editor**: No undo/redo, no line references (`line 3 * 2`)

**Working well:**
- ✅ Parentheses and operator precedence
- ✅ Currency conversions with complex expressions
- ✅ Variables and assignments
- ✅ Text multipliers (k, m, b, cr, lakh)

## Development Workflow

### Build & Run
```bash
cargo build          # Compiles to build/debug/indumi
cargo run            # Build + run
cargo build --release # Optimized binary to build/release/indumi
```

### VS Code Settings
- `rust-analyzer.checkOnSave: false` - Manual builds avoid conflicts
- `rust-analyzer.cargo.targetDir: "build"` - Custom build dir

### Testing Approach

**Comprehensive test suite with 78+ automated tests:**

#### Unit Tests (`cargo test`)
- **Parser tests** (src/parser.rs): 35 tests covering:
  - Basic parsing (numbers, variables, operators)
  - Operator precedence and parentheses
  - Text multipliers (billion, million, crore, lakh)
  - Currency annotations and conversions
  - Assignment expressions
  - Error handling

- **Calculator tests** (src/calc.rs): 26 tests covering:
  - Expression evaluation (all operators)
  - Variable storage and retrieval
  - Currency conversion logic
  - Number formatting (Indian and Western styles)
  - Estimation display (K, M, B, Lac, Cr)
  - Edge cases (division by zero, undefined variables)

#### Integration Tests (`cargo test --test integration_test`)
- **17 end-to-end tests** covering:
  - Complex expressions with parentheses
  - Currency conversions with math operations
  - Variable usage across calculations
  - Real-world scenarios (budget calculations, large numbers)
  - Error handling and edge cases

#### Running Tests
```bash
cargo test                       # All tests (unit + integration)
cargo test parser                # Just parser tests
cargo test --test integration    # Just integration tests
cargo test -- --nocapture        # Show println output
cargo test <test_name>           # Run specific test
```

#### Test Coverage
- ✅ Basic arithmetic (+, -, *, /)
- ✅ Operator precedence
- ✅ Parentheses
- ✅ Text-based numbers (k, m, b, cr, lakh)
- ✅ Currency conversions (USD, EUR, INR)
- ✅ Complex expressions
- ✅ Variables and assignments
- ✅ Number formatting (both styles)
- ✅ Error cases
- ✅ Edge cases

#### Manual Testing Checklist
For UI/UX testing (run `cargo run`):
- [ ] Cursor movement feels smooth
- [ ] Results update in real-time
- [ ] Colors are readable
- [ ] Unicode symbols display correctly (₹, €, $)
- [ ] Large numbers format correctly
- [ ] Parentheses matching works
- [ ] Multiline editing works
- [ ] Ctrl+C quits cleanly

## Extension Ideas

### Near-term (Good First Tasks)
- ✅ ~~Implement `*`, `/` operators~~ (DONE)
- Implement `^`, `%` operators
- Add parentheses support for precedence
- Scrolling for long documents
- Save/load files (`.indumi` extension)
- Error highlighting in UI (partial - errors show in red)

### Medium-term
- More currencies (GBP, JPY, CNY, etc.)
- Unit conversions (km to miles, kg to lbs)
- Percentage calculations (e.g., `100 + 15%`)
- Date/time calculations
- Line references (e.g., `line 3 * 2`)

### Advanced
- Functions (sin, cos, sqrt, log)
- Multi-currency results (show converted amount in all currencies)
- Live exchange rates via API with caching
- Syntax highlighting and themes
- Vim keybindings mode

## Debugging Tips

- **Parser issues**: Add `println!("{:?}", tokens)` in `tokenize()`
- **Calculator issues**: Print `expr` before `evaluate()`
- **UI glitches**: Check cursor bounds in `editor.rs`
- **Terminal not restoring**: Ensure `disable_raw_mode()` runs on panic

## Design Principles

1. **Immediate feedback** - Results update as you type
2. **Forgiveness** - Invalid lines show errors, don't crash
3. **Discoverability** - Natural syntax (e.g., "100 USD to INR")
4. **Minimal UI** - Two panels, no clutter
5. **Speed** - Sub-10ms rendering for smooth typing

## Terminal Font & Appearance

### Recommended Fonts
For the best experience, use a monospace font with good Unicode support:

- **JetBrains Mono** - Clean, modern, excellent for code
- **Fira Code** - Popular with ligatures support
- **Cascadia Code** - Microsoft's modern monospace font
- **SF Mono** - macOS system monospace font
- **Hack** - Clear and readable

All fonts should support Unicode symbols: ₹, €, $, and other special characters.

### Color Scheme

The UI uses a carefully chosen color palette for clarity and visual appeal:

**Input Panel (Left, 60%)**
- Border: Bright Cyan (#00FFFF) - Clear, professional
- Active line text: Bright White + Bold - High contrast for current line
- Inactive line text: Gray - Subdued but readable
- Cursor: Black text on White background + Bold - Clear position indicator
- Title: "Indumi Calculator (Ctrl+C to quit)"

**Results Panel (Right, 40%)**
- Border: Bright Magenta (#FF00FF) - Distinct from input panel
- Result text: Bright Green (#00FF00) - Success/output color
- Error text: Bright Red - Clear error indication
- Title: "Results"

**Color Psychology**
- Cyan: Technical, clear, professional - ideal for input
- Magenta: Creative, distinct - separates output visually
- Green: Success, calculation complete
- Red: Error, attention needed

To modify colors, edit `src/ui.rs` and adjust `Style::default().fg(Color::...)` calls.

## When Adding Code

- Keep functions under 30 lines when possible
- Prefer small enums over booleans for clarity
- Use `?` for error propagation, not `.unwrap()`
- Test edge cases manually after changes
- Update README.md if user-facing behavior changes

## Questions to Ask

Before implementing a feature:
- Does this align with "calculator that feels like a text editor"?
- Can a user discover this without reading docs?
- Does it work with live results (no "run" button)?
- Will it still feel fast with 1000 lines?

---

**Remember**: Indumi is about making quick calculations feel effortless. Prioritize user experience over feature count.
