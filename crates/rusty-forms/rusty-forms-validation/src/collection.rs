//! Collection validation functions

use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::BTreeSet;

/// Validates minimum number of items in a collection
pub fn validate_min_items<T>(items: &[T], min: usize) -> Result<(), String> {
    if items.len() >= min {
        Ok(())
    } else {
        Err(format!("Must have at least {} items", min))
    }
}

/// Validates maximum number of items in a collection
pub fn validate_max_items<T>(items: &[T], max: usize) -> Result<(), String> {
    if items.len() <= max {
        Ok(())
    } else {
        Err(format!("Must have at most {} items", max))
    }
}

/// Validates all items in collection are unique
pub fn validate_unique<T: Ord + Clone>(items: &[T]) -> Result<(), String> {
    let mut seen = BTreeSet::new();

    for item in items {
        if !seen.insert(item.clone()) {
            return Err("All items must be unique".to_string());
        }
    }

    Ok(())
}

/// Validates all items in a string collection are unique
pub fn validate_unique_strings(items: &[String]) -> Result<(), String> {
    let mut seen = BTreeSet::new();

    for item in items {
        if !seen.insert(item) {
            return Err("All items must be unique".to_string());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::string::ToString;

    #[test]
    fn test_min_items() {
        let items = vec![1, 2, 3];
        assert!(validate_min_items(&items, 2).is_ok());
        assert!(validate_min_items(&items, 3).is_ok());
        assert!(validate_min_items(&items, 5).is_err());
    }

    #[test]
    fn test_max_items() {
        let items = vec![1, 2, 3];
        assert!(validate_max_items(&items, 5).is_ok());
        assert!(validate_max_items(&items, 3).is_ok());
        assert!(validate_max_items(&items, 2).is_err());
    }

    #[test]
    fn test_unique() {
        let unique_items = vec![1, 2, 3, 4];
        assert!(validate_unique(&unique_items).is_ok());

        let duplicate_items = vec![1, 2, 3, 2];
        assert!(validate_unique(&duplicate_items).is_err());
    }

    #[test]
    fn test_unique_strings() {
        let unique = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert!(validate_unique_strings(&unique).is_ok());

        let duplicates = vec!["a".to_string(), "b".to_string(), "a".to_string()];
        assert!(validate_unique_strings(&duplicates).is_err());
    }
}
