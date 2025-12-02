# rusty-forms-wasm

Real-time form validation for the browser using WebAssembly. Same validation logic as server-side, zero network latency.

## 🚀 Features

- **Instant Validation** - Validate as users type, no server round-trips
- **Same Logic** - Uses identical validators as server-side rusty-forms
- **Tiny Bundle** - Optimized WASM binary (~50KB gzipped)
- **Type-Safe** - Full TypeScript support
- **Zero Dependencies** - Pure Rust compiled to WASM
- **30+ Validators** - Email, password, string, numeric, collections

## 📦 Installation

### Prerequisites

Install `wasm-pack`:
```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Build

```bash
# Build for web (ES modules)
wasm-pack build --target web

# Build for production (optimized)
wasm-pack build --target web --release

# Build for Node.js
wasm-pack build --target nodejs
```

This creates a `pkg/` directory with the WASM binary and JavaScript bindings.

## 🎯 Quick Start

### HTML + JavaScript

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Validation Demo</title>
</head>
<body>
    <form id="myForm">
        <input
            type="email"
            id="email"
            data-validate='{"email": true, "required": true}'
        />
        <span class="error-message"></span>

        <button type="submit">Submit</button>
    </form>

    <script type="module">
        import { initValidation, autoAttachValidation, validateForm } from './validation.js';

        // Initialize WASM
        await initValidation();

        // Auto-attach to all fields with data-validate
        await autoAttachValidation();

        // Validate on submit
        document.getElementById('myForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            const isValid = await validateForm(e.target);

            if (isValid) {
                console.log('Form is valid!');
                // Submit to server...
            }
        });
    </script>
</body>
</html>
```

### Manual Validation

```javascript
import { validate } from './validation.js';

const errors = await validate('email', 'user@example.com', {
    email: true,
    noPublicDomains: true,
    required: true
});

if (errors.length > 0) {
    console.log('Validation failed:', errors[0].message);
} else {
    console.log('Valid!');
}
```

### TypeScript

```typescript
import { validate, FieldRules, ValidationError } from './validation.js';

const rules: FieldRules = {
    email: true,
    noPublicDomains: true,
    required: true
};

const errors: ValidationError[] = await validate('email', value, rules);
```

## 📚 API Reference

### `initValidation()`

Initialize the WASM module. Call this before using any validation functions.

```javascript
await initValidation();
```

### `validate(fieldName, value, rules)`

Validate a single field.

**Parameters:**
- `fieldName` (string) - Name of the field
- `value` (string) - Value to validate
- `rules` (object) - Validation rules

**Returns:** Promise<ValidationError[]>

```javascript
const errors = await validate('password', 'weak', {
    password: 'strong',
    required: true
});
```

### `attachValidation(element, rules, options)`

Attach real-time validation to a form field.

```javascript
const input = document.getElementById('email');

await attachValidation(input, {
    email: true,
    required: true
}, {
    validateOn: ['blur', 'input'],
    debounceTime: 300,
    errorClass: 'is-invalid',
    onValidate: (errors, element) => {
        console.log('Validated:', errors);
    }
});
```

### `autoAttachValidation(container)`

Automatically attach validation to all elements with `data-validate` attribute.

```javascript
// Attach to all fields in document
await autoAttachValidation();

// Attach to fields in specific container
await autoAttachValidation(document.getElementById('myForm'));
```

### `validateForm(form)`

Validate all fields in a form.

**Returns:** Promise<boolean> - true if all fields valid

```javascript
const form = document.getElementById('myForm');
const isValid = await validateForm(form);
```

## 🔧 Validation Rules

### Email Validators

```javascript
{
    email: true,                              // Valid email format
    noPublicDomains: true,                    // No gmail, yahoo, etc.
    blockedDomains: ['spam.com', 'temp.com'], // Block specific domains
}
```

### Password Validators

```javascript
{
    password: 'basic',   // 6+ characters
    password: 'medium',  // 8+ chars, upper, lower, digit
    password: 'strong',  // 8+ chars, upper, lower, digit, special
}
```

### String Length

```javascript
{
    minLength: 3,         // Minimum length
    maxLength: 50,        // Maximum length
    required: true,       // Cannot be empty
}
```

### String Matching

```javascript
{
    contains: 'text',     // Must contain substring
    notContains: 'bad',   // Must not contain substring
    startsWith: 'user_',  // Must start with prefix
    endsWith: '.com',     // Must end with suffix
}
```

### Equality

```javascript
{
    equals: 'expected',    // Must equal value
    notEquals: 'forbidden' // Must not equal value
}
```

### URL Validation

```javascript
{
    url: true  // Valid http:// or https:// URL
}
```

### Custom Message

```javascript
{
    email: true,
    required: true,
    message: 'Please enter a valid email address'
}
```

## 🎨 Styling

```css
/* Error state */
input.error {
    border-color: #e74c3c;
}

/* Error message */
.error-message {
    color: #e74c3c;
    font-size: 0.875rem;
    display: none;
}

.error-message.visible {
    display: block;
}
```

## 🧪 Testing

```bash
# Run tests in headless browser
wasm-pack test --headless --firefox

# Run tests in Chrome
wasm-pack test --headless --chrome
```

## 📊 Bundle Size

| Build | Size (gzipped) |
|-------|---------------|
| Debug | ~120KB |
| Release (`-O3`) | ~50KB |
| Release (`-Oz`) | ~35KB |

## 🌐 Browser Support

- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 79+

All modern browsers with WebAssembly support.

## 🔄 Integration with HTMX

Perfect companion for HTMX forms:

```html
<form hx-post="/users" hx-target="#result">
    <!-- Client-side validation on blur -->
    <input
        name="email"
        data-validate='{"email": true, "required": true}'
    />

    <!-- Server validates on submit via HTMX -->
    <button type="submit">Submit</button>
</form>

<script type="module">
    import { autoAttachValidation } from './validation.js';
    await autoAttachValidation();
</script>
```

Benefits:
1. **Instant feedback** - Client-side validation as user types
2. **Server validation** - HTMX sends to server on submit
3. **Same rules** - Identical validation logic on both sides
4. **No duplication** - Single source of truth

## 📝 Examples

See `demo.html` for a complete working example with:
- Email validation
- Password strength validation
- Password confirmation
- Real-time feedback
- Form submission

Run the demo:
```bash
# Build WASM
wasm-pack build --target web

# Serve demo (any HTTP server)
python3 -m http.server 8000

# Open http://localhost:8000/demo.html
```

## 🤝 Contributing

This WASM module is part of the RHTMX framework. See the main repository for contribution guidelines.

## 📄 License

MIT
