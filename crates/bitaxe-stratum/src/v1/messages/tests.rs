use super::*;
use crate::jsonrpc::StratumRequestId;

#[test]
fn stratum_v1_protocol_subscribe_serializes_user_agent() {
    // Arrange
    let message = StratumV1ClientMessage::subscribe(StratumRequestId::new(1), "ultra", "205");

    // Act
    let json_line = match message.to_json_line() {
        Ok(json_line) => json_line,
        Err(error) => panic!("subscribe serialization failed: {error}"),
    };

    // Assert
    assert!(json_line.contains("\"method\":\"mining.subscribe\""));
    assert!(json_line.contains("bitaxe/ultra/205"));
    assert!(json_line.ends_with('\n'));
}

#[test]
fn stratum_v1_protocol_authorize_serializes_credentials() {
    // Arrange
    let message =
        StratumV1ClientMessage::authorize(StratumRequestId::new(2), "synthetic-user", "x");

    // Act
    let json_line = match message.to_json_line() {
        Ok(json_line) => json_line,
        Err(error) => panic!("authorize serialization failed: {error}"),
    };

    // Assert
    assert!(json_line.contains("\"method\":\"mining.authorize\""));
    assert!(json_line.contains("synthetic-user"));
    assert!(json_line.contains("\"x\""));
}

#[test]
fn stratum_v1_protocol_client_method_classes_serialize() {
    // Arrange
    let configure = StratumV1ClientMessage::ConfigureVersionRolling {
        id: StratumRequestId::new(3),
        mask: 0xffff_fffe,
    };
    let suggest = StratumV1ClientMessage::suggest_difficulty(StratumRequestId::new(4), 1000);
    let extranonce = StratumV1ClientMessage::extranonce_subscribe(StratumRequestId::new(5));
    let submit = StratumV1ClientMessage::submit_share(
        StratumRequestId::new(6),
        "synthetic-user",
        "job",
        "00000000",
        0x6470_25b5,
        0x1234_5678,
        0,
    );

    // Act
    let rendered = [
        configure.to_json_line(),
        suggest.to_json_line(),
        extranonce.to_json_line(),
        submit.to_json_line(),
    ];

    // Assert
    assert!(matches!(&rendered[0], Ok(line) if line.contains("mining.configure")));
    assert!(matches!(&rendered[1], Ok(line) if line.contains("mining.suggest_difficulty")));
    assert!(matches!(&rendered[2], Ok(line) if line.contains("mining.extranonce.subscribe")));
    assert!(matches!(&rendered[3], Ok(line) if line.contains("mining.submit")));
}

#[test]
fn stratum_v1_protocol_set_difficulty_accepts_large_pool_value() {
    // Arrange
    let input = r#"{"id":null,"method":"mining.set_difficulty","params":[4294967295]}"#;

    // Act
    let message = match parse_server_message(input) {
        Ok(message) => message,
        Err(error) => panic!("set_difficulty parse failed: {error}"),
    };

    // Assert
    assert_eq!(
        message,
        StratumV1ServerMessage::SetDifficulty(PoolDifficulty {
            difficulty: 4_294_967_295.0
        })
    );
}

#[test]
fn stratum_v1_protocol_notify_parses_upstream_self_test_shape() {
    // Arrange
    let input = r#"{"id":null,"method":"mining.notify","params":["0","0100000000000000000000000000000000000000000000000000000000000000","ffffffff","ffffffff",[],"20000004","1705ae3a","647025b5",true]}"#;

    // Act
    let message = match parse_server_message(input) {
        Ok(message) => message,
        Err(error) => panic!("notify parse failed: {error}"),
    };

    // Assert
    let StratumV1ServerMessage::Notify(notify) = message else {
        panic!("expected mining.notify message");
    };
    assert_eq!(notify.job_id, "0");
    assert!(notify.clean_jobs);
    assert_eq!(notify.version, 0x2000_0004);
    assert_eq!(notify.nbits, 0x1705_ae3a);
    assert_eq!(notify.ntime, 0x6470_25b5);
}

#[test]
fn stratum_v1_protocol_server_method_classes_parse() {
    // Arrange
    let set_extranonce = r#"{"id":null,"method":"mining.set_extranonce","params":["deadbeef",8]}"#;
    let set_version_mask = r#"{"id":1,"method":"mining.set_version_mask","params":["1fffe000"]}"#;
    let show_message =
        r#"{"id":null,"method":"client.show_message","params":["Welcome to the pool!"]}"#;

    // Act
    let extranonce = parse_server_message(set_extranonce);
    let version_mask = parse_server_message(set_version_mask);
    let pool_message = parse_server_message(show_message);

    // Assert
    assert!(matches!(
        extranonce,
        Ok(StratumV1ServerMessage::SetExtranonce(
            ExtranonceAssignment {
                extranonce2_len: 8,
                ..
            }
        ))
    ));
    assert!(matches!(
        version_mask,
        Ok(StratumV1ServerMessage::SetVersionMask(VersionMask {
            mask: 0x1fff_e000
        }))
    ));
    assert!(matches!(
        pool_message,
        Ok(StratumV1ServerMessage::ClientShowMessage(message)) if message == "Welcome to the pool!"
    ));
}

#[test]
fn stratum_v1_protocol_response_success_and_unknown_method_parse() {
    // Arrange
    let success = r#"{"id":1,"result":true,"error":null}"#;
    let unknown = r#"{"id":null,"method":"mining.unknown","params":[]}"#;

    // Act
    let success_message = parse_server_message(success);
    let unknown_error = parse_server_message(unknown);

    // Assert
    assert!(matches!(
        success_message,
        Ok(StratumV1ServerMessage::Response(StratumResponse {
            maybe_id: Some(id),
            success: true,
            ..
        })) if id.raw() == 1
    ));
    assert!(matches!(
        unknown_error,
        Err(crate::error::StratumV1Error::UnknownMethod { method }) if method == "mining.unknown"
    ));
}
