// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025-2026 Sreehari Anil and project contributors

/// Global debug flag to control internal library debug output.
///
/// To see internal logs:
/// 1. Set `PRESENCEFORGE_DEBUG=1` environment variable.
/// 2. Set `RUST_LOG=debug` (or higher) environment variable.
/// 3. Initialize a logger (like `env_logger`) in your application.
use std::sync::OnceLock;

static DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

pub fn is_debug_enabled() -> bool {
    *DEBUG_ENABLED.get_or_init(|| {
        std::env::var("PRESENCEFORGE_DEBUG")
            .map(|val| val == "1" || val.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    })
}

/// Macro for conditional debug printing using the log crate.
///
/// Only emits logs if `PRESENCEFORGE_DEBUG` is set.
/// Requires an active logger (e.g., `RUST_LOG=debug`) to be visible.
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => {
        if $crate::macros::is_debug_enabled() {
            log::debug!($($arg)*);
        }
    };
}
