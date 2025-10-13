# Unreleased

## [0.0.0] - Unreleased

> **WARNING:** Early development version. Not recommended for production use.

### Added

#### Core Features

- Cross-platform Discord IPC support (Unix sockets for Linux/macOS, named pipes for Windows)
- Automatic discovery of Discord IPC pipes/sockets
- Flatpak Discord support with automatic detection
- Synchronous client API (`DiscordIpcClient`)
- Unified async API (`AsyncDiscordIpcClient`) with runtime-agnostic design
- Support for Tokio, async-std, and smol runtimes via feature flags
- Activity builder pattern for creating Rich Presence activities
- Full Discord Rich Presence field support (state, details, timestamps, assets, buttons, party)
- Basic input validation for all Discord field length limits
- Pipe discovery and custom path selection
- Connection timeout configuration
- Retry logic with exponential backoff (sync and async)
- Comprehensive error handling with categorization
- UUID v4-based cryptographic nonces for request tracking

#### Testing

- Unit tests for core functionality
- Integration tests for activity builder, serialization, error handling, IPC protocol, and retry logic

### Feature Flags

- `tokio-runtime` - Tokio async runtime support
- `async-std-runtime` - async-std runtime support
- `smol-runtime` - smol runtime support
