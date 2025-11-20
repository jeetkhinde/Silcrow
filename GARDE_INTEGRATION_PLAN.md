# Garde Integration Plan for RHTMX-Forms

## Executive Summary

**Goal**: Replace ~1,025 lines of custom validation logic with the mature `garde` crate while preserving RHTMX-Forms' unique "single source of truth" pattern.

**Status**: Planning Phase
**Estimated LOC Reduction**: ~2,500 → ~500 lines (80% reduction)
**Risk Level**: Medium (requires careful macro adaptation)

---

## Current System Analysis

### Strengths to Preserve
1. **FormField Macro**: Auto-generates HTML5 attributes from Rust structs
2. **Single Source of Truth**: Define validation rules once, use everywhere
3. **WASM Bridge**: Client-side validation with same logic as server
4. **Custom Validators**: Unique email domain logic (public/blocked domains)

### Components to Replace
- `/crates/RHTMX-Form/src/validation.rs` (1,025 lines) - Core validation engine
- `/crates/rhtmx-validation/core/src/` - Custom validation implementations

### Components to Adapt
- FormField macro attribute parsing
- WASM bridge integration
- Custom email domain validators

---

## Garde Capabilities Assessment

### Built-in Validators (18 total)
✅ **Can Replace Directly**:
- `email` → replaces custom email format validation
- `url` → replaces custom URL validation
- `length(min=N, max=N)` → replaces minlength/maxlength
- `range(min=N, max=N)` → replaces numeric range
- `pattern(regex)` → replaces regex validation
- `required` → replaces required validation
- `matches` → replaces field equality checks
- `alphanumeric`, `ascii` → replaces character set validation

✅ **Bonus Features** (not currently implemented):
- `credit_card` validation
- `phone_number` validation
- `ip`/`ipv4`/`ipv6` validation

⚠️ **Need Custom Implementation**:
- Public email domain blocking (unique to RHTMX)
- Blocked domain list checking (unique to RHTMX)
- Password strength tiers (unique implementation)

### Garde Strengths
- ✅ Active maintenance (2024 releases)
- ✅ Custom validator support via `#[garde(custom(fn_name))]`
- ✅ Context support for passing runtime data
- ✅ WASM compatible (`js-sys`, `wasm-bindgen-test`)
- ✅ Better error messages than validator crate
- ✅ Inspired by validator but modern rewrite

### Garde Limitations
- ❌ No public API to access validation metadata (challenge for FormField)
- ❓ `no_std` support unclear (need to verify for WASM)
- ⚠️ Breaking change for existing code using custom `#[validate(...)]` attributes

---

## Architecture Design

### Simplified API Design (Garde Hidden)

**User Experience** - Super clean, garde is invisible:
```rust
// ✅ CLEAN: User never sees garde complexity
#[derive(FormField)]
struct LoginForm {
    #[email]
    #[no_public_domains]
    email: String,

    #[password(strength = "strong")]
    #[min_length(8)]
    password: String,
}

// Simple validation call
form.validate()?;
```

**Under the Hood** - FormField macro handles everything:
- Uses garde internally for built-in validators (`email`, `min_length`, etc.)
- Uses custom functions for RHTMX-specific validators (`no_public_domains`, `password`)
- Auto-injects context (no user boilerplate)
- Merges all errors into single result

### Three-Layer Integration Strategy

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: Server-Side Validation (Hidden from User)         │
├─────────────────────────────────────────────────────────────┤
│ FormField macro generates:                                 │
│                                                             │
│ impl Validate for LoginForm {                              │
│   fn validate(&self) -> Result<(), ValidationErrors> {     │
│     // Auto-inject context                                 │
│     let ctx = EmailValidationContext { ... };              │
│                                                             │
│     // Use garde for built-in validators                   │
│     garde::validate_email(&self.email)?;                   │
│     garde::validate_length(&self.password, 8, None)?;      │
│                                                             │
│     // Use custom for RHTMX validators                     │
│     no_public_email(&self.email, &ctx)?;                   │
│     password_strength(&self.password)?;                    │
│   }                                                         │
│ }                                                           │
└─────────────────────────────────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: FormField Macro (Compile-Time)                    │
├─────────────────────────────────────────────────────────────┤
│ #[derive(FormField)]                                       │
│ - Parse simple attributes (#[email], #[no_public_domains]) │
│ - Map built-ins → garde validators internally             │
│ - Map RHTMX-specific → custom validators                  │
│ - Generate HTML5 attrs (same as before)                   │
│ - Generate data-validate JSON (same as before)            │
│ - Generate Validate trait impl (new)                      │
│                                                             │
│ Output: <input type="email" required data-validate="..."/> │
└─────────────────────────────────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ Layer 3: WASM Bridge (Client-Side)                         │
├─────────────────────────────────────────────────────────────┤
│ Option A: Port garde to WASM (if no_std compatible)       │
│ Option B: Keep minimal validation core + custom logic     │
│ Option C: Hybrid - garde for std rules, custom for email  │
│                                                             │
│ validateField(name, value, rules) → ValidationError[]     │
└─────────────────────────────────────────────────────────────┘
```

**Key Design Principle**: Garde is an implementation detail, not part of the user-facing API.

---

## Implementation Plan

### Phase 1: Foundation (Week 1)

#### 1.1 Add Garde Dependency
**File**: `/crates/RHTMX-Form/Cargo.toml`

```toml
[dependencies]
garde = { version = "0.20", features = ["email", "url", "pattern"] }
```

**Considerations**:
- Check if `garde` works with `no_std` for WASM (test compilation)
- Enable only needed feature flags to minimize binary size

#### 1.2 Create Custom Validators
**New File**: `/crates/RHTMX-Form/src/validators/email_domain.rs`

```rust
use garde::Validate;

/// Context for email domain validation
pub struct EmailValidationContext {
    pub public_domains: &'static [&'static str],
    pub blocked_domains: Vec<String>,
}

/// Custom validator: No public email domains
pub fn no_public_email(
    value: &str,
    ctx: &EmailValidationContext
) -> garde::Result {
    let domain = extract_domain(value);
    if ctx.public_domains.iter().any(|&d| d == domain) {
        return Err(garde::Error::new("public email domains not allowed"));
    }
    Ok(())
}

/// Custom validator: No blocked domains
pub fn no_blocked_email(
    value: &str,
    ctx: &EmailValidationContext
) -> garde::Result {
    let domain = extract_domain(value);
    if ctx.blocked_domains.iter().any(|d| d == domain) {
        return Err(garde::Error::new("this email domain is blocked"));
    }
    Ok(())
}

fn extract_domain(email: &str) -> &str {
    email.split('@').nth(1).unwrap_or("")
}
```

**New File**: `/crates/RHTMX-Form/src/validators/password.rs`

```rust
/// Custom validator: Password strength
pub fn password_strength(value: &str, _ctx: &()) -> garde::Result {
    let has_uppercase = value.chars().any(|c| c.is_uppercase());
    let has_lowercase = value.chars().any(|c| c.is_lowercase());
    let has_digit = value.chars().any(|c| c.is_numeric());
    let has_special = value.chars().any(|c| "!@#$%^&*".contains(c));

    if value.len() < 8 {
        return Err(garde::Error::new("password must be at least 8 characters"));
    }

    if !(has_uppercase && has_lowercase && has_digit) {
        return Err(garde::Error::new(
            "password must contain uppercase, lowercase, and digit"
        ));
    }

    Ok(())
}
```

**Files to Create**:
- `/crates/RHTMX-Form/src/validators/mod.rs` - Module organization
- `/crates/RHTMX-Form/src/validators/email_domain.rs` - Email validators
- `/crates/RHTMX-Form/src/validators/password.rs` - Password validators

#### 1.3 Update Validation Trait
**File**: `/crates/rhtmx/src/validation/mod.rs`

```rust
// BEFORE (current):
pub trait Validate {
    fn validate(&self) -> Result<(), HashMap<String, String>>;
}

// AFTER (garde integration):
pub trait Validate {
    /// Validate using garde with optional context
    fn validate(&self) -> Result<(), ValidationErrors>;

    /// Convert garde errors to HashMap for template rendering
    fn validation_errors(&self) -> HashMap<String, String>;
}
```

**Changes**:
- Wrap `garde::Validate` trait
- Provide conversion from `garde::Report` to `HashMap<String, String>`
- Maintain backward compatibility for templates

---

### Phase 2: FormField Macro Adaptation (Week 2)

#### 2.1 Attribute Parser Update
**File**: `/crates/RHTMX-Form/src/validation.rs`

**Current**: Parses `#[validate(...)]` attributes
**Target**: Parse `#[garde(...)]` attributes

```rust
// BEFORE:
#[validate(email, minlength = 5, maxlength = 100)]
email: String,

// AFTER:
#[garde(email, length(min = 5, max = 100))]
email: String,
```

**Implementation Strategy**:
1. Update `parse_validation_attrs()` to recognize garde syntax
2. Map garde attribute names to internal representation
3. Keep same `ValidationAttr` enum structure

```rust
// File: /crates/RHTMX-Form/src/validation.rs (lines ~50-200)
enum ValidationAttr {
    Email,
    Length { min: Option<usize>, max: Option<usize> },
    Range { min: Option<i64>, max: Option<i64> },
    Pattern { regex: String },
    Custom { func: String },
    // ... existing variants
}

fn parse_garde_attrs(field: &syn::Field) -> Vec<ValidationAttr> {
    // Parse #[garde(...)] attributes
    // Map to ValidationAttr enum
    // Same structure as before, different input syntax
}
```

#### 2.2 HTML5 Attribute Mapping
**File**: `/crates/RHTMX-Form/src/validation.rs` (lines 802-837)

**No changes needed** - Already maps `ValidationAttr` to HTML5:

```rust
fn validation_to_html5_attrs(attrs: &[ValidationAttr]) -> HashMap<String, String> {
    // Email → type="email"
    // Length → minlength/maxlength
    // Range → min/max
    // Pattern → pattern
    // Required → required
}
```

Just ensure `parse_garde_attrs()` returns same `ValidationAttr` structure.

#### 2.3 JSON Validation Rules
**File**: `/crates/RHTMX-Form/src/validation.rs` (lines 839-938)

**No changes needed** - Already generates `data-validate` JSON:

```rust
fn validation_to_json(attrs: &[ValidationAttr]) -> String {
    // Same JSON format as before
    // WASM bridge expects this structure
}
```

#### 2.4 Macro Entry Point
**File**: `/crates/RHTMX-Form/src/lib.rs` (lines 244-247)

```rust
#[proc_macro_derive(FormField, attributes(garde, form_field))]
pub fn derive_form_field(input: TokenStream) -> TokenStream {
    // Update to look for #[garde(...)] instead of #[validate(...)]
    validation::impl_form_field(input)
}
```

**Key Change**: Add `garde` to attributes list so macro can read them.

---

### Phase 3: WASM Bridge Integration (Week 3)

#### 3.1 Evaluate Garde WASM Compatibility

**Action**: Test if garde compiles to WASM

```bash
cd /crates/rhtmx-validation/wasm
cargo add garde --features email,url,pattern
wasm-pack build --target web
```

**Decision Tree**:
- ✅ If successful → Use garde directly in WASM (Option A)
- ❌ If fails → Keep minimal validation core (Option B)

#### 3.2 Option A: Direct Garde Integration (Preferred)

**File**: `/crates/rhtmx-validation/wasm/src/lib.rs`

```rust
use garde::Validate;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn validateField(
    field_name: &str,
    value: &str,
    rules: JsValue,
) -> Result<JsValue, JsValue> {
    // Parse rules JSON into garde validation struct
    // Run garde validation
    // Return errors as JSON
}
```

**Benefits**:
- Same validation logic server + client (true single source)
- Less code to maintain
- Garde's better error messages on client too

**Risks**:
- Garde may not support `no_std` (need to verify)
- Larger WASM binary size (need to test)

#### 3.3 Option B: Hybrid Approach (Fallback)

Keep minimal validation core for WASM, use garde on server only.

**File**: `/crates/rhtmx-validation/core/src/lib.rs`

```rust
// Keep lightweight implementations for WASM
pub fn validate_email(value: &str) -> bool { /* simple regex */ }
pub fn validate_length(value: &str, min: usize, max: usize) -> bool { /* ... */ }
```

**File**: `/crates/rhtmx-validation/wasm/src/lib.rs`

```rust
// Use core validation functions (current approach)
// Keep as-is if garde WASM doesn't work
```

**Trade-off**: More code to maintain, but guaranteed WASM compatibility.

---

### Phase 4: Migration & Testing (Week 4)

#### 4.1 Update Example Forms

**File**: `/examples/form/src/main.rs` (or similar)

```rust
// BEFORE (current):
#[derive(Validate, FormField)]
struct LoginForm {
    #[validate(email)]
    email: String,
    #[validate(minlength = 8)]
    password: String,
}

form.validate()?;

// AFTER (with garde hidden):
#[derive(FormField)]  // Just one derive!
struct LoginForm {
    #[email]
    #[no_public_domains]
    email: String,

    #[password(strength = "strong")]
    #[min_length(8)]
    password: String,
}

form.validate()?;  // Same simple call, context auto-injected
```

**Key Improvements**:
- ✅ No `garde::Validate` derive needed
- ✅ No context creation/passing
- ✅ Clean stacked attributes
- ✅ Same validation call

#### 4.2 Test Coverage

**Server-Side Tests**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_garde_email_validation() {
        // Test garde email validator
    }

    #[test]
    fn test_custom_public_domain_blocking() {
        // Test custom validator
    }

    #[test]
    fn test_form_field_html5_generation() {
        // Ensure HTML5 attrs still generated correctly
    }
}
```

**WASM Tests**:
```rust
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen_test]
fn test_wasm_validation() {
    // Test WASM bridge with garde
}
```

#### 4.3 Documentation Updates

**Files to Update**:
- `README.md` - Update examples to use garde
- `/docs/validation.md` - Document garde integration
- `/examples/*/README.md` - Update all examples

**Key Points**:
- Migration guide from `#[validate]` to `#[garde]`
- Custom validator usage examples
- Context usage for email domain blocking

---

## Detailed Attribute Mapping

### Validation Rules Conversion

| Current `#[validate(...)]` | Garde `#[garde(...)]` | HTML5 Output |
|----------------------------|----------------------|--------------|
| `email` | `email` | `type="email"` |
| `url` | `url` | `type="url"` |
| `minlength = N` | `length(min = N)` | `minlength="N"` |
| `maxlength = N` | `length(max = N)` | `maxlength="N"` |
| `min = N` (number) | `range(min = N)` | `min="N"` |
| `max = N` (number) | `range(max = N)` | `max="N"` |
| `regex = "..."` | `pattern("...")` | `pattern="..."` |
| `required` | `required` | `required` |
| `matches = "field"` | `matches(field)` | None (JS only) |
| `custom(no_public_email)` | `custom(no_public_email)` | `data-validate` JSON |

### Custom Validators Mapping

| Current | Garde Equivalent | Implementation |
|---------|-----------------|----------------|
| `#[validate(email, no_public_domains)]` | `#[garde(email, custom(no_public_email))]` | Custom function |
| `#[validate(email, blocked_domains)]` | `#[garde(email, custom(no_blocked_email))]` | Custom function |
| `#[validate(password_strength = "medium")]` | `#[garde(custom(password_strength))]` | Custom function |

---

## Risk Assessment & Mitigation

### High Risk

**Risk**: FormField macro breaks due to attribute parsing changes
**Mitigation**:
- Create comprehensive tests before migration
- Implement parser incrementally with fallback
- Test with all 30 current validators

**Risk**: WASM bridge incompatible with garde
**Mitigation**:
- Test garde WASM compilation early (Phase 3.1)
- Keep Option B (hybrid) as backup plan
- Minimal code changes if fallback needed

### Medium Risk

**Risk**: Breaking changes for existing users
**Mitigation**:
- Provide migration guide
- Consider supporting both `#[validate]` and `#[garde]` temporarily
- Version bump (0.x → 0.y) to signal breaking change

**Risk**: Performance regression
**Mitigation**:
- Benchmark garde vs current implementation
- Profile WASM binary size (target: <50KB gzipped)
- Optimize feature flags

### Low Risk

**Risk**: Custom validators don't integrate well
**Mitigation**:
- Garde explicitly supports custom validators
- Context support covers email domain blocking use case
- Prototype in Phase 1 to verify

---

## Success Metrics

### Code Quality
- ✅ LOC reduction: ~1,025 → ~300 lines (-70%)
- ✅ Reduced maintenance burden (leverage garde updates)
- ✅ Better error messages (garde's improved reporting)

### Functionality Preservation
- ✅ FormField macro still generates HTML5 attributes
- ✅ WASM bridge maintains client-side validation
- ✅ Custom email domain validators work
- ✅ Single source of truth pattern intact

### Developer Experience
- ✅ Modern, actively maintained dependency
- ✅ Better documentation (garde's ecosystem)
- ✅ Easier to add new validators (garde's built-ins)
- ✅ Clear migration path for users

### Performance
- ✅ WASM binary ≤ current size (~35KB gzipped)
- ✅ Server validation performance equivalent or better
- ✅ No regression in HTML generation time

---

## Timeline

| Week | Phase | Deliverables |
|------|-------|-------------|
| 1 | Foundation | Custom validators, garde dependency added |
| 2 | FormField Macro | Attribute parser updated, HTML5 generation working |
| 3 | WASM Bridge | WASM compatibility tested, integration complete |
| 4 | Migration | Examples updated, docs updated, tests passing |

**Total Estimated Time**: 4 weeks
**Recommended Approach**: One phase per week, with buffer for testing

---

## Open Questions

### To Investigate

1. **Garde no_std support**: Does garde compile without std for WASM?
   - Action: Test `cargo build --target wasm32-unknown-unknown --no-default-features`
   - Fallback: Use Option B (hybrid approach)

2. **Binary size impact**: How much does garde add to WASM bundle?
   - Action: Benchmark current (35KB) vs garde version
   - Target: Keep under 50KB gzipped

3. **Error message customization**: Can we customize garde error messages per field?
   - Action: Review garde's `Error::new()` API
   - Requirement: Must support localization-ready messages

4. **Attribute compatibility**: Can FormField macro support both `#[validate]` and `#[garde]`?
   - Action: Test dual-attribute parsing
   - Benefit: Easier migration for users

### To Decide

1. **Migration strategy**: Big bang vs gradual?
   - Option A: Full migration in one PR (breaking change)
   - Option B: Support both syntaxes during transition period

2. **WASM approach**: Direct garde or hybrid?
   - Depends on Question #1 answer (no_std support)
   - Preference: Direct garde for true single source

3. **Version bump**: 0.x → 0.y or 0.x → 1.0?
   - Consider: Is this the right time for 1.0 release?
   - Signal: Major architectural improvement

---

## Recommendation

### Proceed with Integration ✅

**Why**:
1. **Proven library**: Garde is mature, actively maintained, modern rewrite of validator
2. **Clear benefits**: -70% LOC, better errors, growing ecosystem
3. **Manageable risks**: WASM compatibility testable early, fallback plan exists
4. **Preserves uniqueness**: Custom validators + FormField macro + WASM bridge all maintained
5. **Improves DX**: Easier to add validators, better documentation

### Suggested Path

1. **Start with Phase 1** (Foundation)
   - Test garde WASM compatibility immediately
   - Implement custom validators
   - Verify this week's work before proceeding

2. **Prototype FormField macro changes** (Phase 2)
   - Prove attribute parsing works
   - Validate HTML5 generation unchanged
   - This is the highest-risk component

3. **Evaluate WASM options** (Phase 3)
   - If garde WASM works → use directly
   - If not → keep minimal core (low overhead)

4. **Complete migration** (Phase 4)
   - Update all examples
   - Write comprehensive tests
   - Document for users

### Next Steps

1. Get approval on this plan
2. Create feature branch `feature/garde-integration`
3. Start Phase 1 implementation
4. Review after each phase before proceeding

---

## Appendix: Code Structure Changes

### Files to Modify

**Core Changes**:
- `/crates/RHTMX-Form/src/validation.rs` - Attribute parsing (major)
- `/crates/RHTMX-Form/src/lib.rs` - Macro attributes (minor)
- `/crates/RHTMX-Form/Cargo.toml` - Add garde dependency

**New Files**:
- `/crates/RHTMX-Form/src/validators/mod.rs`
- `/crates/RHTMX-Form/src/validators/email_domain.rs`
- `/crates/RHTMX-Form/src/validators/password.rs`

**WASM Changes** (if Option A):
- `/crates/rhtmx-validation/wasm/src/lib.rs` - Garde integration
- `/crates/rhtmx-validation/wasm/Cargo.toml` - Add garde dependency

**Documentation**:
- `README.md`
- `/docs/validation.md`
- All example READMEs

### Files to Potentially Remove

After successful migration:
- `/crates/rhtmx-validation/core/src/email.rs` (156 lines) - If garde WASM works
- `/crates/rhtmx-validation/core/src/password.rs` (146 lines) - If garde WASM works
- Most of `/crates/RHTMX-Form/src/validation.rs` (keep only attribute parser)

**Total Removal**: ~1,000+ lines if garde WASM works fully

---

## Conclusion

This integration will modernize RHTMX-Forms' validation while preserving its unique "single source of truth" architecture. The phased approach minimizes risk, and the fallback options ensure success even if WASM compatibility issues arise.

**The key insight**: We're not replacing the FormField macro or WASM bridge - we're just replacing the validation engine underneath. The unique developer experience remains intact.

Ready to proceed with Phase 1? 🚀
