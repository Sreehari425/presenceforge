// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2025-2026 Sreehari Anil and project contributors

use presenceforge::{Command, DiscordIpcClient, DiscordIpcError, IpcConfig, Opcode};
use serde_json::json;
use std::convert::TryFrom;

#[test]
fn ipc_config_variants_adjust_parameters() {
    let default = IpcConfig::default();
    let fast = IpcConfig::fast_connect();
    let extended = IpcConfig::extended();

    assert!(fast.max_sockets <= default.max_sockets);
    assert!(fast.retry_interval_ms <= default.retry_interval_ms);
    assert!(extended.retry_interval_ms >= default.retry_interval_ms);
}

#[test]
fn ipc_config_validation_rejects_invalid_values() {
    let invalid = IpcConfig::default().with_max_sockets(0);
    assert!(invalid.validate().is_err());

    let payload_too_large = IpcConfig::default().with_max_payload_size(200 * 1024 * 1024);
    assert!(payload_too_large.validate().is_err());
}

#[test]
fn opcode_try_from_handles_valid_and_invalid_cases() {
    assert!(matches!(Opcode::try_from(0).unwrap(), Opcode::Handshake));
    assert!(matches!(Opcode::try_from(4).unwrap(), Opcode::Pong));

    let err = Opcode::try_from(99).unwrap_err();
    match err {
        DiscordIpcError::ProtocolViolation { context, .. } => {
            assert_eq!(context.received_opcode, Some(99));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn command_serializes_to_expected_strings() {
    let json = serde_json::to_string(&Command::SetActivity).expect("serialize command");
    assert_eq!(json, "\"SET_ACTIVITY\"");

    let message = json!({
        "cmd": Command::Subscribe,
        "args": json!({"event": "ACTIVITY_JOIN"}),
        "nonce": "abc123"
    });

    let serialized = serde_json::to_string(&message).expect("serialize embed message");
    assert!(serialized.contains("\"SUBSCRIBE\""));
}

#[test]
fn ready_event_is_parsed_from_payload() {
    let payload = json!({
        "cmd": "DISPATCH",
        "evt": "READY",
        "data": {
            "user": {
                "id": "1234",
                "username": "tester"
            }
        }
    });

    let ready = DiscordIpcClient::ready_event_from_payload(&payload)
        .expect("payload should deserialize")
        .expect("ready event should be present");

    assert_eq!(
        ready.user.and_then(|u| u.username),
        Some("tester".to_string())
    );
}
