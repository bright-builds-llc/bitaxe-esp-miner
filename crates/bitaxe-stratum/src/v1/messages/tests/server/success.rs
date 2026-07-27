use crate::v1::messages::{
    parse_server_message, ExtranonceAssignment, PoolDifficulty, StratumResponseError,
    StratumV1ServerMessage, VersionMask,
};

#[test]
fn notify_parses_upstream_self_test_shape() {
    // Arrange
    let input = concat!(
        r#"{"id":null,"method":"mining.notify","params":["0","#,
        r#""0100000000000000000000000000000000000000000000000000000000000000","#,
        r#""ffffffff","ffffffff",["#,
        r#""0000000000000000000000000000000000000000000000000000000000000001""#,
        r#"],"20000004","1705ae3a","647025b5",true]}"#
    );

    // Act
    let message = parse_server_message(input).expect("notify should parse");

    // Assert
    let StratumV1ServerMessage::Notify(notify) = message else {
        panic!("expected mining.notify message");
    };
    assert_eq!(notify.job_id, "0");
    assert_eq!(notify.merkle_branches.len(), 1);
    assert!(notify.clean_jobs);
    assert_eq!(notify.version, 0x2000_0004);
    assert_eq!(notify.nbits, 0x1705_ae3a);
    assert_eq!(notify.ntime, 0x6470_25b5);
}

#[test]
fn set_difficulty_accepts_large_pool_value() {
    // Arrange
    let input = r#"{"id":null,"method":"mining.set_difficulty","params":[4294967295]}"#;

    // Act
    let message = parse_server_message(input).expect("difficulty should parse");

    // Assert
    assert_eq!(
        message,
        StratumV1ServerMessage::SetDifficulty(PoolDifficulty {
            difficulty: 4_294_967_295.0
        })
    );
}

#[test]
fn set_extranonce_parses_assignment() {
    // Arrange
    let input = r#"{"id":null,"method":"mining.set_extranonce","params":["deadbeef",8]}"#;

    // Act
    let message = parse_server_message(input).expect("extranonce should parse");

    // Assert
    assert_eq!(
        message,
        StratumV1ServerMessage::SetExtranonce(ExtranonceAssignment {
            extranonce1: "deadbeef".to_owned(),
            extranonce2_len: 8
        })
    );
}

#[test]
fn set_version_mask_parses_hexadecimal_mask() {
    // Arrange
    let input = r#"{"id":1,"method":"mining.set_version_mask","params":["1fffe000"]}"#;

    // Act
    let message = parse_server_message(input).expect("version mask should parse");

    // Assert
    assert_eq!(
        message,
        StratumV1ServerMessage::SetVersionMask(VersionMask { mask: 0x1fff_e000 })
    );
}

#[test]
fn reconnect_accepts_absent_params() {
    // Arrange
    let input = r#"{"id":null,"method":"client.reconnect"}"#;

    // Act
    let message = parse_server_message(input).expect("reconnect should parse");

    // Assert
    assert_eq!(message, StratumV1ServerMessage::ClientReconnect);
}

#[test]
fn get_version_accepts_array_params() {
    // Arrange
    let input = r#"{"id":null,"method":"client.get_version","params":[]}"#;

    // Act
    let message = parse_server_message(input).expect("get_version should parse");

    // Assert
    assert_eq!(message, StratumV1ServerMessage::ClientGetVersion);
}

#[test]
fn show_message_parses_pool_text() {
    // Arrange
    let input = r#"{"id":null,"method":"client.show_message","params":["Welcome to the pool!"]}"#;

    // Act
    let message = parse_server_message(input).expect("show_message should parse");

    // Assert
    assert_eq!(
        message,
        StratumV1ServerMessage::ClientShowMessage("Welcome to the pool!".to_owned())
    );
}

#[test]
fn ping_preserves_numeric_request_id() {
    // Arrange
    let input = r#"{"id":42,"method":"mining.ping"}"#;

    // Act
    let message = parse_server_message(input).expect("ping should parse");

    // Assert
    assert!(
        matches!(message, StratumV1ServerMessage::Ping { maybe_id: Some(id) } if id.raw() == 42)
    );
}

#[test]
fn boolean_success_response_preserves_id() {
    // Arrange
    let input = r#"{"id":1,"result":true,"error":null}"#;

    // Act
    let message = parse_server_message(input).expect("response should parse");

    // Assert
    assert!(matches!(
        message,
        StratumV1ServerMessage::Response(response)
            if response.success
                && response.maybe_id.is_some_and(|id| id.raw() == 1)
                && response.maybe_error.is_none()
    ));
}

#[test]
fn false_response_uses_reject_reason() {
    // Arrange
    let input = r#"{"id":2,"result":false,"reject-reason":"stale share"}"#;

    // Act
    let message = parse_server_message(input).expect("rejection should parse");

    // Assert
    assert!(matches!(
        message,
        StratumV1ServerMessage::Response(response)
            if !response.success
                && response.maybe_error == Some(StratumResponseError {
                    maybe_code: None,
                    message: "stale share".to_owned()
                })
    ));
}

#[test]
fn false_response_defaults_unknown_reject_reason() {
    // Arrange
    let input = r#"{"id":2,"result":false}"#;

    // Act
    let message = parse_server_message(input).expect("rejection should parse");

    // Assert
    assert!(matches!(
        message,
        StratumV1ServerMessage::Response(response)
            if response
                .maybe_error
                .as_ref()
                .is_some_and(|error| error.message == "unknown")
    ));
}

#[test]
fn subscribe_response_parses_extranonce_assignment() {
    // Arrange
    let input = r#"{"id":2,"result":[[],"abcd",4],"error":null}"#;

    // Act
    let message = parse_server_message(input).expect("subscribe response should parse");

    // Assert
    assert!(matches!(
        message,
        StratumV1ServerMessage::Response(response)
            if response.maybe_extranonce == Some(ExtranonceAssignment {
                extranonce1: "abcd".to_owned(),
                extranonce2_len: 4
            })
    ));
}

#[test]
fn configure_response_parses_version_mask() {
    // Arrange
    let input = concat!(
        r#"{"id":1,"result":{"version-rolling":true,"#,
        r#""version-rolling.mask":"1fffe000"},"error":null}"#
    );

    // Act
    let message = parse_server_message(input).expect("configure response should parse");

    // Assert
    assert!(matches!(
        message,
        StratumV1ServerMessage::Response(response)
            if response.maybe_version_mask == Some(VersionMask { mask: 0x1fff_e000 })
    ));
}

#[test]
fn string_response_error_is_preserved() {
    // Arrange
    let input = r#"{"id":9,"result":null,"error":"authorization rejected"}"#;

    // Act
    let message = parse_server_message(input).expect("string error should parse");

    // Assert
    assert!(matches!(
        message,
        StratumV1ServerMessage::Response(response)
            if response.maybe_error == Some(StratumResponseError {
                maybe_code: None,
                message: "authorization rejected".to_owned()
            })
    ));
}

#[test]
fn array_response_error_preserves_code_and_message() {
    // Arrange
    let input = r#"{"id":9,"result":null,"error":[21,"job not found",null]}"#;

    // Act
    let message = parse_server_message(input).expect("array error should parse");

    // Assert
    assert!(matches!(
        message,
        StratumV1ServerMessage::Response(response)
            if response.maybe_error == Some(StratumResponseError {
                maybe_code: Some(21),
                message: "job not found".to_owned()
            })
    ));
}

#[test]
fn object_response_error_preserves_code_and_message() {
    // Arrange
    let input = r#"{"id":9,"result":null,"error":{"code":22,"message":"duplicate share"}}"#;

    // Act
    let message = parse_server_message(input).expect("object error should parse");

    // Assert
    assert!(matches!(
        message,
        StratumV1ServerMessage::Response(response)
            if response.maybe_error == Some(StratumResponseError {
                maybe_code: Some(22),
                message: "duplicate share".to_owned()
            })
    ));
}
