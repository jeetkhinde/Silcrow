# RHTMX-Forms Garde Migration Roadmap

## Two-Phase Approach: Internal First, API Later

### ✅ Phase 1: Internal Migration (DO NOW)
**Goal**: Use garde internally, keep current `#[validate(...)]` syntax
**Timeline**: 2-3 weeks
**Risk**: Low (no API changes for users)

### 🔮 Phase 2: API Simplification (DO LATER)
**Goal**: Improve syntax to `#[email]` style
**Timeline**: 1 week after Phase 1 stable
**Risk**: Medium (breaking changes, but optional)

---

## Phase 1: Internal Garde Migration (Current Focus)

### User Experience: NO CHANGES

```rust
// ✅ SAME API - Users don't see any difference
#[derive(Validate, FormField)]
struct LoginForm {
    #[validate(email)]
    email: String,

    #[validate(minlength = 8)]
    password: String,
}

form.validate()?;
```

**What users see**: Nothing changes!
**What changes internally**: garde powers the validation instead of custom code

---

## Week 1: Foundation & Testing

### 1.1 Add Garde Dependency

**File**: `/crates/RHTMX-Form/Cargo.toml`

```toml
[dependencies]
garde = { version = "0.20", features = ["email", "url", "pattern"] }
```

### 1.2 Test Garde WASM Compatibility ⚠️ CRITICAL

**Action**: Verify garde compiles to WASM

```bash
cd /crates/rhtmx-validation/wasm

# Test 1: Add garde dependency
cargo add garde --features email,url,pattern --no-default-features

# Test 2: Try to build WASM
wasm-pack build --target web

# Test 3: Check binary size
ls -lh pkg/*.wasm
```

**Decision Point**:
- ✅ **If succeeds**: Use garde in WASM (Option A) - Best outcome
- ❌ **If fails**: Keep current WASM core (Option B) - Still 50% LOC reduction on server

### 1.3 Create Custom Validators Module

**New File**: `/crates/RHTMX-Form/src/garde_validators.rs`

```rust
//! Custom garde validators for RHTMX-specific features

use garde::Validate;

/// Static list of public email domains to block
pub static PUBLIC_DOMAINS: &[&str] = &[
    "gmail.com", "yahoo.com", "hotmail.com", "outlook.com",
    "aol.com", "icloud.com", "mail.com", "protonmail.com",
    "zoho.com", "yandex.com"
];

/// Validator: Block public email domains
pub fn no_public_email(value: &str, _ctx: &()) -> garde::Result {
    let domain = value.split('@').nth(1).unwrap_or("");
    let domain_lower = domain.to_lowercase();

    if PUBLIC_DOMAINS.iter().any(|&d| d == domain_lower) {
        return Err(garde::Error::new("public email domains not allowed"));
    }

    Ok(())
}

/// Validator: Block specific domains (from config)
pub fn blocked_domain_validator(value: &str, blocked: &[String]) -> garde::Result {
    let domain = value.split('@').nth(1).unwrap_or("");
    let domain_lower = domain.to_lowercase();

    if blocked.iter().any(|d| d.to_lowercase() == domain_lower) {
        return Err(garde::Error::new("this email domain is blocked"));
    }

    Ok(())
}

/// Validator: Password strength tiers
pub fn password_strength(value: &str, tier: &str) -> garde::Result {
    let has_upper = value.chars().any(|c| c.is_uppercase());
    let has_lower = value.chars().any(|c| c.is_lowercase());
    let has_digit = value.chars().any(|c| c.is_numeric());
    let has_special = value.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

    match tier {
        "basic" => {
            if value.len() < 6 {
                return Err(garde::Error::new("password must be at least 6 characters"));
            }
        }
        "medium" => {
            if value.len() < 8 {
                return Err(garde::Error::new("password must be at least 8 characters"));
            }
            if !(has_upper && has_lower && has_digit) {
                return Err(garde::Error::new(
                    "password must contain uppercase, lowercase, and digit"
                ));
            }
        }
        "strong" => {
            if value.len() < 8 {
                return Err(garde::Error::new("password must be at least 8 characters"));
            }
            if !(has_upper && has_lower && has_digit && has_special) {
                return Err(garde::Error::new(
                    "password must contain uppercase, lowercase, digit, and special character"
                ));
            }
        }
        _ => {
            return Err(garde::Error::new("invalid password strength tier"));
        }
    }

    Ok(())
}
```

**Files to Create**:
- `/crates/RHTMX-Form/src/garde_validators.rs` - Custom validators
- Update `/crates/RHTMX-Form/src/lib.rs` to include module

---

## Week 2: Macro Adaptation

### 2.1 Update Validation Macro to Use Garde

**File**: `/crates/RHTMX-Form/src/validation.rs`

**Strategy**: Keep parsing `#[validate(...)]`, but generate garde calls internally

```rust
// Current parsing stays the same
fn parse_validation_attrs(field: &syn::Field) -> Vec<ValidationAttr> {
    // Parse #[validate(email, minlength = 5)]
    // Same as before!
}

// NEW: Map to garde validators
fn generate_validation_impl(attrs: &[ValidationAttr]) -> TokenStream {
    let mut validations = vec![];

    for attr in attrs {
        match attr {
            // Use garde for built-ins
            ValidationAttr::Email => {
                validations.push(quote! {
                    if !garde::rules::email::validate_email(&#field_name) {
                        errors.insert(#field_str, "invalid email format");
                    }
                });
            }

            ValidationAttr::MinLength { value } => {
                validations.push(quote! {
                    if #field_name.len() < #value {
                        errors.insert(#field_str, format!("must be at least {} characters", #value));
                    }
                });
            }

            // Use custom for RHTMX-specific
            ValidationAttr::NoPublicDomains => {
                validations.push(quote! {
                    if let Err(e) = crate::garde_validators::no_public_email(&#field_name, &()) {
                        errors.insert(#field_str, e.to_string());
                    }
                });
            }

            // ... map all 30 validators
        }
    }

    quote! {
        impl Validate for #struct_name {
            fn validate(&self) -> Result<(), HashMap<String, String>> {
                let mut errors = HashMap::new();
                #(#validations)*
                if errors.is_empty() { Ok(()) } else { Err(errors) }
            }
        }
    }
}
```

**Key Point**: Attribute parsing unchanged, just different code generation!

### 2.2 Validator Mapping Implementation

| Current Attribute | Generated Code |
|------------------|----------------|
| `#[validate(email)]` | `garde::rules::email::validate_email(value)` |
| `#[validate(minlength = 8)]` | `garde::rules::length::validate_length(value, Some(8), None)` |
| `#[validate(url)]` | `garde::rules::url::validate_url(value)` |
| `#[validate(no_public_domains)]` | `garde_validators::no_public_email(value, &())` |
| `#[validate(password = "strong")]` | `garde_validators::password_strength(value, "strong")` |

**Benefit**: Same attribute syntax, better validators underneath!

---

## Week 3: WASM Bridge (Conditional)

### Option A: Garde WASM Works (Preferred)

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
    let rules: FieldRules = serde_wasm_bindgen::from_value(rules)?;

    let mut errors = vec![];

    // Use garde validators
    if rules.email {
        if !garde::rules::email::validate_email(value) {
            errors.push(ValidationError {
                field: field_name.to_string(),
                message: "Invalid email format".to_string(),
            });
        }
    }

    // Use custom validators
    if rules.no_public_domains {
        if let Err(e) = rhtmx_form::garde_validators::no_public_email(value, &()) {
            errors.push(ValidationError {
                field: field_name.to_string(),
                message: e.to_string(),
            });
        }
    }

    Ok(serde_wasm_bindgen::to_value(&errors)?)
}
```

### Option B: Garde WASM Fails (Fallback)

**Keep current WASM implementation**: `/crates/rhtmx-validation/wasm/src/lib.rs` unchanged

**Trade-off**: Server uses garde, WASM uses custom - still works, just separate implementations

---

## Week 4: Testing & Validation

### 4.1 Test Suite

**File**: `/crates/RHTMX-Form/tests/garde_integration.rs`

```rust
#[cfg(test)]
mod tests {
    use rhtmx_form::{Validate, FormField};

    #[test]
    fn test_email_validation_with_garde() {
        #[derive(Validate, FormField)]
        struct TestForm {
            #[validate(email)]
            email: String,
        }

        let valid = TestForm { email: "test@example.com".to_string() };
        assert!(valid.validate().is_ok());

        let invalid = TestForm { email: "not-an-email".to_string() };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_no_public_domains() {
        #[derive(Validate, FormField)]
        struct TestForm {
            #[validate(email, no_public_domains)]
            email: String,
        }

        let invalid = TestForm { email: "test@gmail.com".to_string() };
        let result = invalid.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains_key("email"));
    }

    #[test]
    fn test_password_strength() {
        #[derive(Validate, FormField)]
        struct TestForm {
            #[validate(password = "strong")]
            password: String,
        }

        let weak = TestForm { password: "abc123".to_string() };
        assert!(weak.validate().is_err());

        let strong = TestForm { password: "Abcd123!".to_string() };
        assert!(strong.validate().is_ok());
    }

    // Test all 30 validators...
}
```

### 4.2 Regression Testing

**Ensure no behavior changes**:
1. All existing tests pass
2. Form validation works same as before
3. HTML5 attributes generated correctly
4. WASM bridge functions properly
5. Error messages match (or improve)

---

## Phase 1 Success Criteria

### ✅ Must Have
- [ ] Garde dependency added
- [ ] 14 validators use garde internally
- [ ] 16 custom validators implemented
- [ ] All existing tests pass
- [ ] FormField macro generates same code
- [ ] No API changes for users

### ✅ WASM Decision Made
- [ ] Garde WASM tested (success or failure)
- [ ] WASM bridge working (either with garde or without)

### ✅ Documentation
- [ ] CHANGELOG entry
- [ ] Internal docs updated
- [ ] Migration tested on real forms

---

## Phase 2: API Simplification (Future)

### After Phase 1 is Stable

**Goal**: Simplify to clean attribute syntax

```rust
// BEFORE (Phase 1):
#[derive(Validate, FormField)]
struct Form {
    #[validate(email, no_public_domains)]
    email: String,
}

// AFTER (Phase 2):
#[derive(FormField)]  // Single derive
struct Form {
    #[email]
    #[no_public_domains]
    email: String,
}
```

**Changes Required**:
1. Support both syntaxes (backward compatibility)
2. Parse multiple attributes per field
3. Auto-generate Validate trait
4. Remove need for context passing

**Timeline**: After Phase 1 proven stable (4+ weeks)

---

## Risk Mitigation

### Phase 1 Risks

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Garde WASM fails | Medium | Keep current WASM (Option B) |
| Macro complexity | Low | Same parsing, just different codegen |
| Performance regression | Very Low | Garde likely faster than custom |
| Breaking changes | None | API unchanged |

### Phase 2 Risks

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| User migration burden | Medium | Support both syntaxes |
| Complex macro | Medium | Phase 1 proves feasibility |

---

## Implementation Checklist

### Week 1
- [ ] Create feature branch `feature/garde-internal-migration`
- [ ] Add garde to `/crates/RHTMX-Form/Cargo.toml`
- [ ] Test garde WASM compilation
- [ ] Create `/crates/RHTMX-Form/src/garde_validators.rs`
- [ ] Implement 3 custom validators (email domains, password)

### Week 2
- [ ] Update `/crates/RHTMX-Form/src/validation.rs` codegen
- [ ] Map 14 validators to garde
- [ ] Keep 16 validators as custom
- [ ] Test macro generates correct code

### Week 3
- [ ] Update WASM bridge (if garde compatible)
- [ ] OR keep WASM as-is (if garde incompatible)
- [ ] Test client-side validation

### Week 4
- [ ] Write comprehensive tests
- [ ] Run regression tests
- [ ] Update internal documentation
- [ ] Prepare for merge

---

## Benefits (Phase 1)

### Immediate
- ✅ -50% LOC on server validation
- ✅ Better email validation (garde's implementation)
- ✅ Better error messages
- ✅ Easier to maintain (leverage garde updates)

### No Downsides
- ✅ Same user API
- ✅ Same functionality
- ✅ Same performance (likely better)
- ✅ No breaking changes

---

## Next Steps

1. **Get approval** on this phased approach
2. **Create branch** `feature/garde-internal-migration`
3. **Start Week 1** - Test garde WASM immediately
4. **Proceed iteratively** - One week at a time with reviews

**Phase 2 (API simplification)** can be discussed after Phase 1 proves successful!

---

## Summary

**Phase 1 Strategy**:
- Internal implementation change only
- Users see zero difference
- Lower risk, immediate benefits
- Proves garde integration works

**Phase 2 Strategy**:
- API improvement (optional)
- Cleaner syntax for users
- Built on proven Phase 1 foundation

**This approach de-risks the migration while delivering benefits immediately!**
