/**
 * rusty-forms-wasm - JavaScript Wrapper
 *
 * High-level API for using rusty-forms validation in the browser.
 * Provides real-time validation with debouncing, DOM manipulation, and error display.
 */

import init, { validateField, isValidEmail, validatePassword, isValidUrl } from './pkg/rusty_forms_wasm.js';

let wasmInitialized = false;

/**
 * Initialize WASM module
 * @returns {Promise<void>}
 */
export async function initValidation() {
    if (!wasmInitialized) {
        await init();
        wasmInitialized = true;
    }
}

/**
 * Validate a single field
 * @param {string} fieldName - Name of the field
 * @param {string} value - Value to validate
 * @param {Object} rules - Validation rules
 * @returns {Promise<Array>} Array of errors (empty if valid)
 */
export async function validate(fieldName, value, rules) {
    await initValidation();
    return validateField(fieldName, value, rules);
}

/**
 * Debounce function for real-time validation
 * @param {Function} func - Function to debounce
 * @param {number} wait - Wait time in milliseconds
 * @returns {Function}
 */
export function debounce(func, wait = 300) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

/**
 * Attach validation to a form field
 * @param {HTMLInputElement|HTMLTextAreaElement} element - Form element
 * @param {Object} rules - Validation rules
 * @param {Object} options - Configuration options
 */
export async function attachValidation(element, rules, options = {}) {
    await initValidation();

    const {
        errorClass = 'error',
        errorSelector = '.error-message',
        validateOn = ['blur', 'input'],
        debounceTime = 300,
        onValidate = null,
    } = options;

    // Find or create error element
    let errorElement = element.parentElement?.querySelector(errorSelector);
    if (!errorElement) {
        errorElement = document.createElement('span');
        errorElement.className = errorSelector.substring(1); // Remove the dot
        element.parentElement?.appendChild(errorElement);
    }

    // Validation handler
    const handleValidation = async () => {
        const fieldName = element.name || element.id || 'field';
        const value = element.value;

        try {
            const errors = await validate(fieldName, value, rules);

            if (errors.length > 0) {
                // Show error
                errorElement.textContent = errors[0].message;
                errorElement.classList.add('visible');
                element.classList.add(errorClass);
                element.setAttribute('aria-invalid', 'true');
            } else {
                // Clear error
                errorElement.textContent = '';
                errorElement.classList.remove('visible');
                element.classList.remove(errorClass);
                element.setAttribute('aria-invalid', 'false');
            }

            // Call custom callback
            if (onValidate) {
                onValidate(errors, element);
            }
        } catch (error) {
            console.error('Validation error:', error);
        }
    };

    // Attach event listeners
    if (validateOn.includes('blur')) {
        element.addEventListener('blur', handleValidation);
    }

    if (validateOn.includes('input')) {
        element.addEventListener('input', debounce(handleValidation, debounceTime));
    }
}

/**
 * Attach validation to all fields with data-validate attribute
 * @param {HTMLElement} container - Container element (default: document)
 */
export async function autoAttachValidation(container = document) {
    await initValidation();

    const fields = container.querySelectorAll('[data-validate]');

    fields.forEach(field => {
        try {
            const rules = JSON.parse(field.getAttribute('data-validate'));
            const options = field.getAttribute('data-validate-options')
                ? JSON.parse(field.getAttribute('data-validate-options'))
                : {};

            attachValidation(field, rules, options);
        } catch (error) {
            console.error('Failed to attach validation to field:', field, error);
        }
    });
}

/**
 * Validate entire form
 * @param {HTMLFormElement} form - Form element
 * @returns {Promise<boolean>} True if form is valid
 */
export async function validateForm(form) {
    await initValidation();

    const fields = form.querySelectorAll('[data-validate]');
    let isValid = true;

    for (const field of fields) {
        try {
            const rules = JSON.parse(field.getAttribute('data-validate'));
            const fieldName = field.name || field.id || 'field';
            const errors = await validate(fieldName, field.value, rules);

            if (errors.length > 0) {
                isValid = false;

                // Display error
                const errorElement = field.parentElement?.querySelector('.error-message');
                if (errorElement) {
                    errorElement.textContent = errors[0].message;
                    errorElement.classList.add('visible');
                }

                field.classList.add('error');
                field.setAttribute('aria-invalid', 'true');
            }
        } catch (error) {
            console.error('Validation error:', error);
            isValid = false;
        }
    }

    return isValid;
}

/**
 * Quick validators (don't require rule objects)
 */
export const quick = {
    async email(value) {
        await initValidation();
        return isValidEmail(value);
    },

    async password(value, pattern = 'strong') {
        await initValidation();
        const error = validatePassword(value, pattern);
        return error === undefined;
    },

    async url(value) {
        await initValidation();
        return isValidUrl(value);
    },
};

// Export for direct use
export { isValidEmail, validatePassword, isValidUrl };
