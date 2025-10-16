/// Global debug flag to control debug output
/// Set to `true` to enable debug prints or use PRESENCEFORGE_DEBUG=1 environment variable
use std::sync::OnceLock;

static DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

pub fn is_debug_enabled() -> bool {
    *DEBUG_ENABLED.get_or_init(|| {
        std::env::var("PRESENCEFORGE_DEBUG")
            .map(|val| val == "1" || val.to_lowercase() == "true")
            .unwrap_or(false)
    })
}
/// Macro for conditional debug printing
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        if $crate::macros::is_debug_enabled() {
            println!($($arg)*);
        }
    };
}
