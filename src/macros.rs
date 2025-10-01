/// Global debug flag to control debug output
/// Set to `true` to enable debug prints, `false` to disable
pub const DEBUG_MODE: bool = false;

/// Macro for conditional debug printing
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        if $crate::macros::DEBUG_MODE {
            println!($($arg)*);
        }
    };
}
