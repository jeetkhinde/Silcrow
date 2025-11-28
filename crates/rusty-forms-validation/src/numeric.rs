//! Numeric validation functions

use alloc::string::String;
use alloc::format;

/// Validates minimum value for numeric types
pub fn validate_min<T: PartialOrd + core::fmt::Display>(value: T, min: T) -> Result<(), String> {
    if value >= min {
        Ok(())
    } else {
        Err(format!("Must be at least {}", min))
    }
}

/// Validates maximum value for numeric types
pub fn validate_max<T: PartialOrd + core::fmt::Display>(value: T, max: T) -> Result<(), String> {
    if value <= max {
        Ok(())
    } else {
        Err(format!("Must be at most {}", max))
    }
}

/// Validates value is within range
pub fn validate_range<T: PartialOrd + core::fmt::Display>(
    value: T,
    min: T,
    max: T,
) -> Result<(), String> {
    if value >= min && value <= max {
        Ok(())
    } else {
        Err(format!("Must be between {} and {}", min, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_validation() {
        assert!(validate_min(10, 5).is_ok());
        assert!(validate_min(5, 5).is_ok());
        assert!(validate_min(3, 5).is_err());

        assert!(validate_min(18.5, 18.0).is_ok());
        assert!(validate_min(17.9, 18.0).is_err());
    }

    #[test]
    fn test_max_validation() {
        assert!(validate_max(5, 10).is_ok());
        assert!(validate_max(10, 10).is_ok());
        assert!(validate_max(15, 10).is_err());

        assert!(validate_max(99.9, 100.0).is_ok());
        assert!(validate_max(100.1, 100.0).is_err());
    }

    #[test]
    fn test_range_validation() {
        assert!(validate_range(5, 1, 10).is_ok());
        assert!(validate_range(1, 1, 10).is_ok());
        assert!(validate_range(10, 1, 10).is_ok());
        assert!(validate_range(0, 1, 10).is_err());
        assert!(validate_range(11, 1, 10).is_err());
    }
}
