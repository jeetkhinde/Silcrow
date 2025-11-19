/// Lazy iterator that generates parent paths on-demand
///
/// For path `/a/b/c/d`, yields: `/a/b/c/d` → `/a/b/c` → `/a/b` → `/a` → `/`
///
/// Stops as soon as a match is found (short-circuit evaluation).
/// This is a **pure functional approach** using lazy evaluation.
///
/// # Performance
///
/// - **Memory**: 16 bytes (single pointer on stack)
/// - **Allocations**: Zero (only borrows from input string)
/// - **Complexity**: O(depth) where depth is path levels
///
/// # Examples
///
/// ```
/// use rhtmx_router::path::PathHierarchy;
///
/// let paths: Vec<&str> = PathHierarchy::new("/a/b/c").collect();
/// assert_eq!(paths, vec!["/a/b/c", "/a/b", "/a", "/"]);
/// ```
///
/// # Functional Programming
///
/// This iterator embodies several FP principles:
/// - **Lazy evaluation**: Paths generated only when requested
/// - **Immutability**: Input string never modified
/// - **Zero-copy**: Returns borrowed slices, no allocations
/// - **Composable**: Works with iterator combinators (`.find()`, `.filter()`, etc.)
pub struct PathHierarchy<'a> {
    current: Option<&'a str>,
}

impl<'a> PathHierarchy<'a> {
    /// Creates a new path hierarchy iterator starting from the given path
    ///
    /// # Examples
    ///
    /// ```
    /// use rhtmx_router::path::PathHierarchy;
    ///
    /// let iter = PathHierarchy::new("/users/123");
    /// let first = iter.clone().next();
    /// assert_eq!(first, Some("/users/123"));
    /// ```
    pub fn new(path: &'a str) -> Self {
        Self {
            current: Some(path),
        }
    }
}

impl<'a> Iterator for PathHierarchy<'a> {
    type Item = &'a str;

    /// Returns the next parent path in the hierarchy
    ///
    /// Pure function: given same state, produces same output.
    ///
    /// # Mutable State Note
    ///
    /// **The mutation of `self.current` is acceptable and necessary:**
    /// - Required by the `Iterator` trait signature (`fn next(&mut self)`)
    /// - Mutation is **encapsulated** within the iterator (not exposed externally)
    /// - Each call to `next()` is **deterministic** given the current state
    /// - Follows the functional principle of **state transitions** (old state → new state)
    /// - Enables **lazy evaluation** without heap allocations
    ///
    /// This is a standard functional programming pattern where an iterator
    /// maintains internal state while appearing pure from the caller's perspective.
    ///
    /// # Algorithm
    ///
    /// 1. If at root ("/"), stop iteration
    /// 2. Find last '/' in current path
    /// 3. Return substring before last '/'
    /// 4. If no '/' found, stop iteration
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        let result = current;

        // Calculate next parent using functional pattern matching
        self.current = if current == "/" {
            None // Reached root, stop iteration
        } else if let Some(slash_pos) = current.rfind('/') {
            if slash_pos == 0 {
                Some("/") // Next is root
            } else {
                Some(&current[..slash_pos]) // Move to parent (zero-copy slice)
            }
        } else {
            None // No more parents
        };

        Some(result)
    }
}

// Make it clonable for reuse
impl<'a> Clone for PathHierarchy<'a> {
    fn clone(&self) -> Self {
        Self {
            current: self.current,
        }
    }
}
