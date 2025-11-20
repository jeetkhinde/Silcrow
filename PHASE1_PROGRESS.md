# Phase 1: Garde Migration - Progress Report

## ✅ Week 1 COMPLETED (Foundation & Testing)

### Critical Decision: WASM Compatibility ✅ PASSED

**Test Performed**:
```bash
cd /crates/rhtmx-validation/wasm
rustup target add wasm32-unknown-unknown
cargo add garde --no-default-features --features email,url
cargo build --target wasm32-unknown-unknown
```

**Result**: ✅ **SUCCESS** - Garde compiles to WASM!

**Decision Made**: Proceed with **Option A** - Use garde in both server AND client (WASM)

**Benefits**:
- True single source of truth (same validation logic everywhere)
- -70% LOC reduction (both server and client)
- Better error messages on client too
- Less code to maintain

---

## Completed Tasks

### ✅ 1. Dependencies Added

**RHTMX-Form** (`/crates/RHTMX-Form/Cargo.toml`):
```toml
garde = { version = "0.22", features = ["email", "url", "pattern"] }
```

**rhtmx-validation-core** (`/crates/rhtmx-validation/core/Cargo.toml`):
```toml
[dependencies]
garde = { version = "0.22", optional = true, default-features = false, features = ["email", "url", "pattern"] }

[features]
default = ["std", "garde"]
std = []
garde = ["dep:garde"]
```

**rhtmx-validation-wasm** (`/crates/rhtmx-validation/wasm/Cargo.toml`):
```toml
garde = { version = "0.22", features = ["email", "url"] }
```

### ✅ 2. Custom Validators Module

**File**: `/crates/rhtmx-validation/core/src/garde_validators.rs`

**Implemented Validators**:
1. **`no_public_email(value, ctx)`** - Blocks gmail, yahoo, etc.
2. **`blocked_domain_validator(value, blocked)`** - Custom domain blocking
3. **`password_strength(value, tier)`** - 3-tier password validation

**Public Domains Blocked** (hardcoded list):
- gmail.com, yahoo.com, hotmail.com, outlook.com
- aol.com, icloud.com, mail.com, protonmail.com
- zoho.com, yandex.com

**Password Tiers**:
- `basic`: 6+ characters
- `medium`: 8+ chars + uppercase + lowercase + digit
- `strong`: 8+ chars + uppercase + lowercase + digit + special

**Test Coverage**: ✅ All validators have comprehensive unit tests

**Integration**: ✅ Properly exported from `rhtmx-validation-core` with feature flag

---

## Remaining Tasks

### 🔄 Week 2: Macro Adaptation (IN PROGRESS)

#### Task 4: Update Validation Macro Codegen

**File to Modify**: `/crates/RHTMX-Form/src/validation.rs`

**Current State**: Generates calls to `rhtmx::validation::validators::*`

**Target State**: Generate calls to garde validators where applicable

**Mapping Strategy**:

| ValidationAttr | Current Code | Target Code (Garde) |
|----------------|--------------|---------------------|
| `Email` | `rhtmx::validation::validators::is_valid_email()` | `garde::rules::email()` or keep custom |
| `NoPublicDomains` | `rhtmx::validation::validators::is_public_domain()` | `rhtmx_validation_core::no_public_email()` |
| `Password` | `rhtmx::validation::validators::validate_password()` | `rhtmx_validation_core::password_strength()` |
| `Min/Max/Range` | Custom numeric checks | Keep as-is (simple enough) |
| `MinLength/MaxLength` | Custom string checks | `garde::rules::length()` |
| `Regex` | `rhtmx::validation::validators::matches_regex()` | `garde::rules::pattern()` |
| `Url` | `rhtmx::validation::validators::is_valid_url()` | `garde::rules::url()` |
| `Contains` | Custom string check | `garde::rules::contains()` |
| `StartsWith` | Custom string check | `garde::rules::prefix()` |
| `EndsWith` | Custom string check | `garde::rules::suffix()` |
| `EqualsField` | Custom field comparison | `garde::rules::matches()` or keep custom |

**Code Generation Changes Needed**:

```rust
// BEFORE (line ~512-517):
ValidationAttr::Email => {
    quote! {
        if !rhtmx::validation::validators::is_valid_email(&self.#field_name) {
            errors.insert(#field_name_str.to_string(), "Invalid email address".to_string());
        }
    }
}

// AFTER (Option A - Use garde):
ValidationAttr::Email => {
    quote! {
        // Use garde's email validator
        if let Err(_) = garde::rules::email(&self.#field_name) {
            errors.insert(#field_name_str.to_string(), "Invalid email address".to_string());
        }
    }
}

// AFTER (Option B - Keep custom but simpler):
ValidationAttr::Email => {
    quote! {
        if !garde::rules::email(&self.#field_name).is_ok() {
            errors.insert(#field_name_str.to_string(), "Invalid email address".to_string());
        }
    }
}
```

**Challenge**: Garde validators return `garde::Result`, not `bool`. Need to adapt the error handling pattern.

**Suggested Approach**:
1. Create a helper function in the generated code to convert `garde::Result` → error insertion
2. Keep same error message format for backward compatibility
3. Map garde errors to user-friendly messages

---

### Week 3: WASM Bridge Integration

Since garde WASM works, we can now use garde directly in the WASM bridge!

**File to Modify**: `/crates/rhtmx-validation/wasm/src/lib.rs`

**Current Implementation**: Calls custom validation functions from `rhtmx-validation-core`

**Target Implementation**: Use garde validators + custom RHTMX validators

**Example**:
```rust
// BEFORE:
if rules.email {
    if !rhtmx_validation_core::email::is_valid_email(value) {
        errors.push(...);
    }
}

// AFTER:
if rules.email {
    if let Err(e) = garde::rules::email(value) {
        errors.push(ValidationError {
            field: field_name.to_string(),
            message: "Invalid email format".to_string(),
        });
    }
}

// Custom validators still work:
if rules.no_public_domains {
    if let Err(e) = rhtmx_validation_core::no_public_email(value, &()) {
        errors.push(...);
    }
}
```

---

### Week 4: Testing & Documentation

**Test Files to Create**:
1. `/crates/RHTMX-Form/tests/garde_integration.rs` - Integration tests
2. Update existing tests to ensure no regression
3. Add WASM tests for garde validators

**Documentation Updates**:
1. Update internal code comments
2. Add CHANGELOG entry
3. Document the garde integration (internal)

---

## Architecture Summary

```
User Code (Unchanged):
┌─────────────────────────────────┐
│ #[derive(Validate, FormField)]  │
│ struct LoginForm {              │
│     #[validate(email)]          │
│     #[validate(no_public_domains)]│
│     email: String,              │
│ }                               │
└─────────────────────────────────┘
           ▼
FormField Macro (Modified):
┌─────────────────────────────────┐
│ Parse: #[validate(email)]       │
│ Map to: ValidationAttr::Email   │
│ Generate:                       │
│   - Server: garde::rules::email()│
│   - WASM: garde::rules::email() │
│   - HTML5: type="email"         │
└─────────────────────────────────┘
           ▼
Runtime Validation:
┌─────────────────────────────────┐
│ Server: garde + custom          │
│ Client (WASM): garde + custom   │
│ HTML5: Native browser           │
└─────────────────────────────────┘
```

---

## Validators Distribution

### Garde Handles (14 validators):
- `email`, `url`, `regex`
- `min_length`, `max_length`, `length`
- `contains`, `starts_with`, `ends_with`
- `equals_field` (as `matches`)
- Potentially: `min`, `max`, `range` (though simple checks work fine as-is)

### Custom/RHTMX Handles (16 validators):
- Email: `no_public_domains`, `blocked_domains`
- Password: `password(tier)`
- Logic: `not_contains`, `equals`, `not_equals`, `depends_on`
- Collections: `min_items`, `max_items`, `unique`
- Enum: `enum_variant`
- Meta: `custom`, `message`, `label`, `message_key`
- General: `required`, `allow_whitespace`

---

## Success Metrics (Current Status)

| Metric | Target | Status |
|--------|--------|--------|
| WASM Compatibility | Test passes | ✅ PASSED |
| Dependencies Added | All 3 crates | ✅ DONE |
| Custom Validators | 3 implemented | ✅ DONE |
| Macro Updated | Generate garde calls | 🔄 IN PROGRESS |
| WASM Bridge | Use garde | ⏳ PENDING |
| Tests | All passing | ⏳ PENDING |
| LOC Reduction | -50% server | ⏳ PENDING |

---

## Next Immediate Steps

1. **Update validation.rs code generation**:
   - Lines ~510-780: Match on ValidationAttr
   - Change from `rhtmx::validation::validators::*` to `garde::rules::*` or `rhtmx_validation_core::*`
   - Handle `garde::Result` → error insertion properly

2. **Test macro changes**:
   - Create a test form with all validator types
   - Ensure validation still works
   - Verify error messages unchanged

3. **Update WASM bridge**:
   - Change `/crates/rhtmx-validation/wasm/src/lib.rs`
   - Use garde validators where applicable
   - Keep custom validators for RHTMX-specific

4. **Add tests**:
   - Integration tests
   - Regression tests
   - WASM tests

5. **Documentation & merge**:
   - Update CHANGELOG
   - Create migration notes (internal)
   - Merge to main

---

## Estimated Completion

- **Week 1**: ✅ COMPLETE (100%)
- **Week 2**: 🔄 25% complete (dependencies done, macro changes in progress)
- **Week 3**: ⏳ Not started
- **Week 4**: ⏳ Not started

**Overall Progress**: **~30% complete**

**Next Session**: Continue with updating validation macro code generation (Week 2 task)

---

## Key Decisions Made

1. ✅ **Use garde in WASM** - Compatibility test passed
2. ✅ **Optional garde feature** - Clean feature flag architecture
3. ✅ **Custom validators in core** - Proper separation of concerns
4. ✅ **Same user API** - No breaking changes (`#[validate(...)]` syntax preserved)

**No blockers identified. Proceeding with Week 2 tasks.**
