/// Level of route interception for intercepting routes (Phase 5.2)
///
/// Defines how an intercepting route should intercept navigation.
/// This enables modal/overlay patterns like Next.js App Router.
///
/// # Examples
///
/// ```
/// use rhtmx_router::InterceptLevel;
///
/// // pages/feed/(.)/photo/[id].rsx → SameLevel
/// let same = InterceptLevel::SameLevel;
///
/// // pages/feed/(..)/photo/[id].rsx → OneLevelUp
/// let up = InterceptLevel::OneLevelUp;
///
/// // pages/feed/(...)/photo/[id].rsx → FromRoot
/// let root = InterceptLevel::FromRoot;
/// ```
///
/// # Interception Patterns
///
/// - `(.)` → **SameLevel**: Intercept at same directory level
/// - `(..)` → **OneLevelUp**: Intercept one directory level up
/// - `(...)` → **FromRoot**: Intercept from application root
/// - `(....)` → **TwoLevelsUp**: Intercept two directory levels up
#[derive(Debug, Clone, PartialEq)]
pub enum InterceptLevel {
    /// (.) - Intercept segments at the same level
    SameLevel,
    /// (..) - Intercept segments one level up
    OneLevelUp,
    /// (...) - Intercept segments from the root
    FromRoot,
    /// (....) - Intercept segments two levels up
    TwoLevelsUp,
}
