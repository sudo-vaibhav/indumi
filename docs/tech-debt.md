# Technical Debt & Future Improvements

This document tracks known limitations, technical debt, and planned improvements for Indumi.

## Current Limitations

### 1. Parser Limitations

#### Unary Minus Not Supported
**Issue**: Negative number literals like `-5` don't parse correctly
```
-5 + 10         L Parse error
0 - 5 + 10       Works (use subtraction instead)
```
**Impact**: Minor UX issue, workaround is simple
**Fix**: Add unary operator support in `parse_primary()`
**Priority**: Low


#### Currency Context Lost After Math Operations
**Issue**: When you do math on a currency conversion, the result loses its currency context
```
(100 USD to INR) / 4    � Returns plain number, not INR
```
**Behavior**: This is by design - division returns a numeric value
**Workaround**: Use variables if you need to preserve context
**Priority**: Design decision - may not fix

### 2. Number Formatting Limitations

#### Small Decimal Precision
**Issue**: Numbers < 0.01 round to 0 in display
```
0.001           � Displays as "0"
0.005           � Displays as "0"
```

#### Assignment Shows Numeric Value Without Currency
**Issue**: When assigning a currency conversion, result doesn't show currency symbol
```
x = 100 USD to INR   � Shows "8791" not "� 8,791"
```
**Root Cause**: Assignment returns the numeric value from evaluation
**Impact**: Minor UX inconsistency
**Fix**: Check if assignment RHS is CurrencyConversion and format accordingly
**Priority**: Low


#### Exchange Rate Fetching
**Issue**: Relies on external API (exchangerate-api.com)
- No offline mode
- No rate caching across sessions
- API could be down or rate-limited
**Impact**: Requires internet connection, potential for failures
**Fix**:
- Add persistent rate cache (JSON file)
- Add manual rate override option
- Improve fallback behavior
**Priority**: Medium

#### No UI Tests
**Issue**: TUI rendering and interaction not tested
**Impact**: UI regressions possible
**Fix**: Add integration tests that mock terminal or use snapshot testing
**Priority**: Low (manual testing sufficient for now)

#### Limited Async Testing
**Issue**: Currency API failures not fully tested
**Impact**: Network error paths may be untested
**Fix**: Add mock CurrencyConverter for deterministic testing
**Priority**: Medium

## Future Improvements

### Near-term (Good First Issues)

3. **Improve error messages** (~2 hours)
   - Show error position in input
   - Suggest corrections
   - Better parse error descriptions

## Architecture Debt

### 1. Currency Converter in Calculator
**Issue**: Calculator depends on CurrencyConverter via async
**Impact**: Makes Calculator initialization async, complicates testing
**Fix**: Dependency injection or lazy initialization
**Priority**: Low

### 2. Parser State
**Issue**: Parser uses index mutation (`i: &mut usize`) extensively
**Impact**: Hard to reason about, error-prone
**Fix**: Use iterator/stream-based approach
**Priority**: Low (works well, not worth refactor)

### 3. Mixed Concerns in evaluate_line
**Issue**: evaluate_line handles parsing, evaluation, AND formatting
**Impact**: Testing individual pieces is harder
**Fix**: Split into separate functions
**Priority**: Low

## Documentation Debt

### Well Documented 
- README with examples
- CLAUDE.md with architecture
- Inline comments for complex logic
- Test coverage ~78 tests

### Could Improve
- [ ] Add doc comments to public functions
- [ ] Create user guide / tutorial
- [ ] Video demo
- [ ] Contribution guidelines

## Technical Decisions to Revisit

### 1. Async Currency Fetching on Startup
**Current**: Fetches rates on Calculator creation
**Problem**: Blocks startup, requires tokio runtime
**Alternative**: Lazy fetch on first conversion
**Revisit**: If startup time becomes an issue

### 2. RefCell for Calculator in Editor
**Current**: `RefCell<Calculator>` for interior mutability
**Problem**: Runtime borrow checking overhead
**Alternative**: Make editor methods take `&mut self`
**Revisit**: If profiling shows RefCell is a bottleneck

### 3. String-based Error Types
**Current**: `Result<T, String>` everywhere
**Problem**: No structure, hard to programmatically handle
**Alternative**: Define custom Error enum
**Revisit**: If error handling becomes more sophisticated

## Resolved Issues

###  Fixed During Development

1. **Currency regex complexity** � Removed, replaced with proper parser
2. **Division operator not working** � Implemented full precedence parsing
3. **Text multipliers only in currency** � Extended to all expressions
4. **No parentheses support** � Added full recursive descent parsing
5. **Test failures with external API** � Made tests more resilient to varying rates

---

**Last Updated**: 2025-01-27
**Next Review**: When adding major features or after 6 months
