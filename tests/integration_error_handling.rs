use presenceforge::error::ErrorCategory;
use presenceforge::{DiscordIpcError, Opcode, ProtocolContext};

#[test]
fn error_category_matches_constructor() {
    let error = DiscordIpcError::SocketClosed;
    assert!(error.is_connection_error());
    assert_eq!(error.category(), ErrorCategory::Connection);

    let protocol_error = DiscordIpcError::InvalidResponse("bad".into());
    assert_eq!(protocol_error.category(), ErrorCategory::Protocol);
    assert!(protocol_error.is_recoverable());

    let app_error = DiscordIpcError::discord_error(5001, "Discord failure");
    assert_eq!(app_error.category(), ErrorCategory::Application);
    assert!(!app_error.is_recoverable());
}

#[test]
fn protocol_violation_context_is_preserved() {
    let context = ProtocolContext::with_opcodes(Opcode::Handshake.into(), Opcode::Frame.into());
    let error = DiscordIpcError::protocol_violation("unexpected opcode", context.clone());

    match error {
        DiscordIpcError::ProtocolViolation {
            context: received, ..
        } => {
            assert_eq!(received.expected_opcode, context.expected_opcode);
            assert_eq!(received.received_opcode, context.received_opcode);
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}
