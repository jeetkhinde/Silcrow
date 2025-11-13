// Example: r-field Directive - Zero Boilerplate Form Generation
//
// This example demonstrates the r-field directive which automatically
// generates validation attributes from your Rust struct, with ZERO manual work!

use rhtmx::{html, Html, Validate, FormField};
use serde::Deserialize;

// ============================================================================
// BEFORE r-field: Manual Attribute Management
// ============================================================================
//
// You had to manually call field_attrs() and render attributes:
//
// ```rust
// let email_attrs = form.field_attrs("email");
// html! {
//     <input name="email" {email_attrs.render_all()} />
// }
// ```
//
// ============================================================================
// AFTER r-field: Zero Boilerplate!
// ============================================================================
//
// Just use r-field directive:
//
// ```rust
// html! {
//     <input r-field={form.email} />
// }
// ```
//
// That's it! Validation attributes are automatically generated!

#[derive(Validate, FormField, Deserialize, Clone, Debug)]
struct RegisterForm {
    #[email]
    #[no_public_domains]
    #[required]
    #[label("Email Address")]
    email: String,

    #[min_length(8)]
    #[max_length(100)]
    #[password("strong")]
    #[required]
    #[label("Password")]
    password: String,

    #[min_length(3)]
    #[max_length(50)]
    #[required]
    #[label("Full Name")]
    name: String,

    #[min(18)]
    #[max(120)]
    #[label("Age")]
    age: i32,

    #[url]
    #[label("Website")]
    website: Option<String>,

    #[custom("validate_username")]
    #[min_length(3)]
    #[max_length(20)]
    #[regex(r"^[a-zA-Z0-9_]+$")]
    #[label("Username")]
    username: String,
}

// Custom validator function (server-side only)
fn validate_username(username: &str) -> Result<(), String> {
    if username.contains("admin") {
        Err("Username cannot contain 'admin'".to_string())
    } else {
        Ok(())
    }
}

// ============================================================================
// Using r-field Directive
// ============================================================================

fn registration_form_with_r_field(form: &RegisterForm) -> Html {
    html! {
        <div class="container">
            <h1>Registration Form</h1>
            <p class="subtitle">Using r-field directive - Zero boilerplate!</p>

            <form action="/register" method="post" class="registration-form">
                <div class="form-group">
                    <label for="email">Email Address</label>
                    // ✨ Magic! Just use r-field={form.email}
                    // Automatically generates: name, type="email", required, data-validate
                    <input id="email" r-field={form.email} class="form-control" />
                </div>

                <div class="form-group">
                    <label for="password">Password</label>
                    // ✨ Automatically generates: name, minlength, maxlength, required, data-validate
                    <input
                        id="password"
                        type="password"
                        r-field={form.password}
                        class="form-control"
                    />
                    <small>Must be 8-100 characters, strong password required</small>
                </div>

                <div class="form-group">
                    <label for="name">Full Name</label>
                    // ✨ Automatically generates: name, minlength, maxlength, required, data-validate
                    <input id="name" r-field={form.name} class="form-control" />
                </div>

                <div class="form-group">
                    <label for="age">Age</label>
                    // ✨ Automatically generates: name, min, max, data-validate
                    <input id="age" type="number" r-field={form.age} class="form-control" />
                    <small>Must be between 18 and 120</small>
                </div>

                <div class="form-group">
                    <label for="website">Website (optional)</label>
                    // ✨ Automatically generates: name, type="url", data-validate
                    <input id="website" r-field={form.website} class="form-control" />
                </div>

                <div class="form-group">
                    <label for="username">Username</label>
                    // ✨ Works with custom validators too!
                    // Note: #[custom("validate_username")] runs server-side only
                    // But HTML5 validation (minlength, maxlength, pattern) still generated!
                    <input id="username" r-field={form.username} class="form-control" />
                    <small>3-20 chars, alphanumeric and underscore only</small>
                </div>

                <button type="submit" class="btn-primary">Register</button>
            </form>

            <style>
                {r#"
                .container {
                    max-width: 600px;
                    margin: 50px auto;
                    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
                }

                .subtitle {
                    color: #666;
                    margin-bottom: 30px;
                }

                .form-group {
                    margin-bottom: 20px;
                }

                label {
                    display: block;
                    margin-bottom: 5px;
                    font-weight: 600;
                    color: #333;
                }

                .form-control {
                    width: 100%;
                    padding: 10px;
                    border: 2px solid #ddd;
                    border-radius: 5px;
                    font-size: 16px;
                    transition: border-color 0.3s;
                }

                .form-control:focus {
                    outline: none;
                    border-color: #2196f3;
                }

                .form-control:invalid {
                    border-color: #f44336;
                }

                small {
                    display: block;
                    margin-top: 5px;
                    color: #666;
                    font-size: 14px;
                }

                .btn-primary {
                    background: #2196f3;
                    color: white;
                    padding: 12px 30px;
                    border: none;
                    border-radius: 5px;
                    font-size: 16px;
                    font-weight: 600;
                    cursor: pointer;
                    transition: background 0.3s;
                }

                .btn-primary:hover {
                    background: #1976d2;
                }
                "#}
            </style>
        </div>
    }
}

// ============================================================================
// Comparison: Before vs After
// ============================================================================

fn registration_form_old_way(form: &RegisterForm) -> String {
    // OLD WAY: Manual attribute management (string concatenation)
    let email_attrs = form.field_attrs("email");
    let password_attrs = form.field_attrs("password");
    let name_attrs = form.field_attrs("name");

    format!(
        r#"
        <form>
            <div>
                <label>Email</label>
                <input name="email" {} />
            </div>
            <div>
                <label>Password</label>
                <input name="password" type="password" {} />
            </div>
            <div>
                <label>Name</label>
                <input name="name" {} />
            </div>
        </form>
        "#,
        email_attrs.render_all(),
        password_attrs.render_all(),
        name_attrs.render_all()
    )
}

fn registration_form_new_way(form: &RegisterForm) -> Html {
    // NEW WAY: r-field directive
    html! {
        <form>
            <div>
                <label>Email</label>
                // ✅ Concise: just r-field!
                <input r-field={form.email} />
            </div>

            <div>
                <label>Password</label>
                // ✅ Concise: just r-field!
                <input type="password" r-field={form.password} />
            </div>

            <div>
                <label>Name</label>
                // ✅ Concise: just r-field!
                <input r-field={form.name} />
            </div>
        </form>
    }
}

// ============================================================================
// Key Features
// ============================================================================
//
// 1. ✨ **Automatic name attribute**: Extracted from field expression
//    `r-field={form.email}` → `name="email"`
//
// 2. ✨ **HTML5 validation**: Automatically generated from struct attributes
//    `#[email]` → `type="email"`
//    `#[required]` → `required`
//    `#[min_length(8)]` → `minlength="8"`
//
// 3. ✨ **Client-side validation**: data-validate JSON automatically generated
//    `#[email]` → `data-validate='{"email":true}'`
//
// 4. ✨ **Custom validators handled**: Server-side only, no client leakage
//    `#[custom("validate_username")]` → runs on server, not in HTML
//
// 5. ✨ **Composable**: Can still add your own attributes
//    `<input r-field={form.email} class="custom" placeholder="Enter email" />`
//
// 6. ✨ **Works with nested fields**:
//    `<input r-field={user.profile.email} />` → `name="email"`
//
// ============================================================================

fn main() {
    println!("========================================");
    println!("  r-field Directive Demo");
    println!("  Zero Boilerplate Form Generation");
    println!("========================================\n");

    let form = RegisterForm {
        email: String::new(),
        password: String::new(),
        name: String::new(),
        age: 18,
        website: None,
        username: String::new(),
    };

    println!("✨ OLD WAY (Verbose):");
    println!("let attrs = form.field_attrs(\"email\");");
    println!("<input name=\"email\" {{attrs.render_all()}} />\n");

    println!("✨ NEW WAY (r-field):");
    println!("<input r-field={{form.email}} />\n");

    println!("========================================");
    println!("Generated HTML:");
    println!("========================================\n");

    let html = registration_form_with_r_field(&form);
    println!("{}", html);

    println!("\n========================================");
    println!("Benefits:");
    println!("========================================");
    println!("✅ Zero boilerplate - just use r-field");
    println!("✅ Automatic name attribute extraction");
    println!("✅ All validation attributes generated");
    println!("✅ Works with custom validators");
    println!("✅ Fully composable with other attributes");
    println!("✅ Type-safe and compile-time checked");
}
