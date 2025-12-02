/**
 * RHTMX Validation - TypeScript Definitions
 */

export interface ValidationError {
    field: string;
    message: string;
}

export interface FieldRules {
    // Email validators
    email?: boolean;
    noPublicDomains?: boolean;
    blockedDomains?: string[];

    // Password validators
    password?: 'basic' | 'medium' | 'strong';

    // String length
    minLength?: number;
    maxLength?: number;

    // String matching
    contains?: string;
    notContains?: string;
    startsWith?: string;
    endsWith?: string;

    // Equality
    equals?: string;
    notEquals?: string;

    // URL
    url?: boolean;

    // Required
    required?: boolean;

    // Custom message
    message?: string;
}

export interface ValidationOptions {
    /** CSS class to add on error (default: 'error') */
    errorClass?: string;

    /** Selector for error message element (default: '.error-message') */
    errorSelector?: string;

    /** Events to validate on (default: ['blur', 'input']) */
    validateOn?: ('blur' | 'input' | 'change')[];

    /** Debounce time for input events in milliseconds (default: 300) */
    debounceTime?: number;

    /** Custom callback after validation */
    onValidate?: (errors: ValidationError[], element: HTMLElement) => void;
}

/**
 * Initialize WASM module
 */
export function initValidation(): Promise<void>;

/**
 * Validate a single field
 * @param fieldName - Name of the field
 * @param value - Value to validate
 * @param rules - Validation rules
 * @returns Array of errors (empty if valid)
 */
export function validate(
    fieldName: string,
    value: string,
    rules: FieldRules
): Promise<ValidationError[]>;

/**
 * Debounce function for real-time validation
 * @param func - Function to debounce
 * @param wait - Wait time in milliseconds (default: 300)
 */
export function debounce<T extends (...args: any[]) => any>(
    func: T,
    wait?: number
): (...args: Parameters<T>) => void;

/**
 * Attach validation to a form field
 * @param element - Form element
 * @param rules - Validation rules
 * @param options - Configuration options
 */
export function attachValidation(
    element: HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement,
    rules: FieldRules,
    options?: ValidationOptions
): Promise<void>;

/**
 * Attach validation to all fields with data-validate attribute
 * @param container - Container element (default: document)
 */
export function autoAttachValidation(container?: Document | HTMLElement): Promise<void>;

/**
 * Validate entire form
 * @param form - Form element
 * @returns True if form is valid
 */
export function validateForm(form: HTMLFormElement): Promise<boolean>;

/**
 * Quick validators (don't require rule objects)
 */
export const quick: {
    /**
     * Check if email is valid
     */
    email(value: string): Promise<boolean>;

    /**
     * Check if password meets pattern requirements
     * @param pattern - 'basic', 'medium', or 'strong' (default: 'strong')
     */
    password(value: string, pattern?: 'basic' | 'medium' | 'strong'): Promise<boolean>;

    /**
     * Check if URL is valid
     */
    url(value: string): Promise<boolean>;
};

/**
 * Check if email is valid (direct WASM call)
 */
export function isValidEmail(email: string): boolean;

/**
 * Validate password (direct WASM call)
 * @returns Error message if invalid, undefined if valid
 */
export function validatePassword(password: string, pattern: string): string | undefined;

/**
 * Check if URL is valid (direct WASM call)
 */
export function isValidUrl(url: string): boolean;
