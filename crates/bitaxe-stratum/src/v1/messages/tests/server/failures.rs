use crate::error::StratumV1Error;
use crate::v1::messages::parse_server_message;

#[test]
fn malformed_or_non_object_roots_fail_as_invalid_json() {
    // Arrange
    let inputs = ["{", "[]", "null", "42"];

    // Act / Assert
    for input in inputs {
        assert_eq!(
            parse_server_message(input),
            Err(StratumV1Error::InvalidJson)
        );
    }
}

#[test]
fn request_ids_reject_non_unsigned_values() {
    // Arrange
    let inputs = [
        r#"{"id":-1,"result":true}"#,
        r#"{"id":"1","result":true}"#,
        r#"{"id":1.5,"result":true}"#,
    ];

    // Act / Assert
    for input in inputs {
        assert_eq!(
            parse_server_message(input),
            Err(StratumV1Error::InvalidField {
                field: "id",
                reason: "expected non-negative integer or null"
            })
        );
    }
}

#[test]
fn non_string_method_fails_closed() {
    // Arrange
    let input = r#"{"id":null,"method":7,"params":[]}"#;

    // Act
    let result = parse_server_message(input);

    // Assert
    assert_eq!(
        result,
        Err(StratumV1Error::InvalidField {
            field: "method",
            reason: "expected string"
        })
    );
}

#[test]
fn unknown_method_names_are_reported() {
    // Arrange
    let input = r#"{"id":null,"method":"mining.unknown","params":[]}"#;

    // Act
    let result = parse_server_message(input);

    // Assert
    assert_eq!(
        result,
        Err(StratumV1Error::UnknownMethod {
            method: "mining.unknown".to_owned()
        })
    );
}

#[test]
fn response_without_result_fails_closed() {
    // Arrange
    let input = r#"{"id":1,"error":null}"#;

    // Act
    let result = parse_server_message(input);

    // Assert
    assert_eq!(result, Err(StratumV1Error::MissingField("result")));
}

#[test]
fn response_rejects_unsupported_result_shapes() {
    // Arrange
    let inputs = [
        r#"{"id":1,"result":null}"#,
        r#"{"id":1,"result":"true"}"#,
        r#"{"id":1,"result":42}"#,
    ];

    // Act / Assert
    for input in inputs {
        assert_eq!(
            parse_server_message(input),
            Err(StratumV1Error::InvalidParams { method: "response" })
        );
    }
}

#[test]
fn response_error_array_requires_string_message() {
    // Arrange
    let inputs = [
        r#"{"id":1,"result":null,"error":[]}"#,
        r#"{"id":1,"result":null,"error":[20,7]}"#,
    ];

    // Act / Assert
    for input in inputs {
        assert_eq!(
            parse_server_message(input),
            Err(StratumV1Error::InvalidParams { method: "response" })
        );
    }
}

#[test]
fn response_error_object_requires_string_message() {
    // Arrange
    let inputs = [
        r#"{"id":1,"result":null,"error":{}}"#,
        r#"{"id":1,"result":null,"error":{"message":7}}"#,
    ];

    // Act / Assert
    for input in inputs {
        assert_eq!(
            parse_server_message(input),
            Err(StratumV1Error::InvalidParams { method: "response" })
        );
    }
}

#[test]
fn response_error_rejects_scalar_shapes() {
    // Arrange
    let input = r#"{"id":1,"result":null,"error":7}"#;

    // Act
    let result = parse_server_message(input);

    // Assert
    assert_eq!(
        result,
        Err(StratumV1Error::InvalidParams { method: "response" })
    );
}

#[test]
fn notify_requires_array_with_complete_params() {
    // Arrange
    let inputs = [
        r#"{"method":"mining.notify"}"#,
        r#"{"method":"mining.notify","params":{}}"#,
        r#"{"method":"mining.notify","params":["job"]}"#,
    ];

    // Act / Assert
    for input in inputs {
        assert_eq!(
            parse_server_message(input),
            Err(StratumV1Error::InvalidParams {
                method: "mining.notify"
            })
        );
    }
}

#[test]
fn notify_requires_string_identity_params() {
    // Arrange
    let input = concat!(
        r#"{"method":"mining.notify","params":[7,"#,
        r#""0000000000000000000000000000000000000000000000000000000000000000","#,
        r#""aa","bb",[],"20000004","1705ae3a","647025b5",true]}"#
    );

    // Act
    let result = parse_server_message(input);

    // Assert
    assert_eq!(
        result,
        Err(StratumV1Error::InvalidParams {
            method: "mining.notify"
        })
    );
}

#[test]
fn notify_requires_merkle_branch_array() {
    // Arrange
    let input = valid_notify_with_merkle(r#""not-an-array""#);

    // Act
    let result = parse_server_message(&input);

    // Assert
    assert_eq!(
        result,
        Err(StratumV1Error::InvalidParams {
            method: "mining.notify"
        })
    );
}

#[test]
fn notify_requires_string_merkle_branches() {
    // Arrange
    let input = valid_notify_with_merkle("[7]");

    // Act
    let result = parse_server_message(&input);

    // Assert
    assert_eq!(
        result,
        Err(StratumV1Error::InvalidParams {
            method: "mining.notify"
        })
    );
}

#[test]
fn notify_hex_fields_require_strings() {
    // Arrange
    let input = concat!(
        r#"{"method":"mining.notify","params":["job","#,
        r#""0000000000000000000000000000000000000000000000000000000000000000","#,
        r#""aa","bb",[],7,"1705ae3a","647025b5",true]}"#
    );

    // Act
    let result = parse_server_message(input);

    // Assert
    assert_eq!(
        result,
        Err(StratumV1Error::InvalidParams {
            method: "mining.notify"
        })
    );
}

#[test]
fn notify_hex_fields_reject_malformed_or_oversized_values() {
    // Arrange
    let inputs = [
        valid_notify_with_version(r#""not-hex""#),
        valid_notify_with_version(r#""100000000""#),
    ];

    // Act / Assert
    for input in inputs {
        assert_eq!(
            parse_server_message(&input),
            Err(StratumV1Error::InvalidField {
                field: "version",
                reason: "expected lowercase or uppercase hexadecimal u32"
            })
        );
    }
}

#[test]
fn notify_requires_boolean_clean_jobs() {
    // Arrange
    let input = concat!(
        r#"{"method":"mining.notify","params":["job","#,
        r#""0000000000000000000000000000000000000000000000000000000000000000","#,
        r#""aa","bb",[],"20000004","1705ae3a","647025b5","true"]}"#
    );

    // Act
    let result = parse_server_message(input);

    // Assert
    assert_eq!(
        result,
        Err(StratumV1Error::InvalidParams {
            method: "mining.notify"
        })
    );
}

#[test]
fn set_difficulty_requires_numeric_first_param() {
    // Arrange
    let inputs = [
        r#"{"method":"mining.set_difficulty","params":[]}"#,
        r#"{"method":"mining.set_difficulty","params":["1"]}"#,
    ];

    // Act / Assert
    for input in inputs {
        assert_eq!(
            parse_server_message(input),
            Err(StratumV1Error::InvalidParams {
                method: "mining.set_difficulty"
            })
        );
    }
}

#[test]
fn set_extranonce_requires_two_params() {
    // Arrange
    let input = r#"{"method":"mining.set_extranonce","params":["abcd"]}"#;

    // Act
    let result = parse_server_message(input);

    // Assert
    assert_eq!(
        result,
        Err(StratumV1Error::InvalidParams {
            method: "mining.set_extranonce"
        })
    );
}

#[test]
fn extranonce_length_requires_non_negative_integer() {
    // Arrange
    let inputs = [
        r#"{"method":"mining.set_extranonce","params":["abcd",-1]}"#,
        r#"{"method":"mining.set_extranonce","params":["abcd","4"]}"#,
    ];

    // Act / Assert
    for input in inputs {
        assert_eq!(
            parse_server_message(input),
            Err(StratumV1Error::InvalidField {
                field: "extranonce2_len",
                reason: "expected non-negative integer"
            })
        );
    }
}

#[test]
fn extranonce_length_rejects_values_above_maximum() {
    // Arrange
    let input = r#"{"method":"mining.set_extranonce","params":["abcd",33]}"#;

    // Act
    let result = parse_server_message(input);

    // Assert
    assert_eq!(
        result,
        Err(StratumV1Error::InvalidField {
            field: "extranonce2_len",
            reason: "exceeds MAX_EXTRANONCE_2_LEN 32"
        })
    );
}

#[test]
fn set_version_mask_requires_first_param() {
    // Arrange
    let input = r#"{"method":"mining.set_version_mask","params":[]}"#;

    // Act
    let result = parse_server_message(input);

    // Assert
    assert_eq!(
        result,
        Err(StratumV1Error::InvalidParams {
            method: "mining.set_version_mask"
        })
    );
}

#[test]
fn subscribe_response_requires_complete_assignment() {
    // Arrange
    let inputs = [r#"{"id":2,"result":[]}"#, r#"{"id":2,"result":[[],7,4]}"#];

    // Act / Assert
    for input in inputs {
        assert_eq!(
            parse_server_message(input),
            Err(StratumV1Error::InvalidParams { method: "response" })
        );
    }
}

#[test]
fn configure_response_requires_enabled_version_rolling() {
    // Arrange
    let inputs = [
        r#"{"id":1,"result":{}}"#,
        r#"{"id":1,"result":{"version-rolling":false}}"#,
    ];

    // Act / Assert
    for input in inputs {
        assert_eq!(
            parse_server_message(input),
            Err(StratumV1Error::InvalidParams { method: "response" })
        );
    }
}

#[test]
fn configure_response_requires_version_mask() {
    // Arrange
    let input = r#"{"id":1,"result":{"version-rolling":true}}"#;

    // Act
    let result = parse_server_message(input);

    // Assert
    assert_eq!(
        result,
        Err(StratumV1Error::InvalidParams { method: "response" })
    );
}

#[test]
fn parameterless_methods_reject_non_array_params() {
    // Arrange
    let inputs = [
        (
            r#"{"method":"client.reconnect","params":{}}"#,
            "client.reconnect",
        ),
        (
            r#"{"method":"client.get_version","params":null}"#,
            "client.get_version",
        ),
        (r#"{"method":"mining.ping","params":7}"#, "mining.ping"),
    ];

    // Act / Assert
    for (input, method) in inputs {
        assert_eq!(
            parse_server_message(input),
            Err(StratumV1Error::InvalidParams { method })
        );
    }
}

fn valid_notify_with_merkle(merkle: &str) -> String {
    format!(
        concat!(
            r#"{{"method":"mining.notify","params":["job","#,
            r#""0000000000000000000000000000000000000000000000000000000000000000","#,
            r#""aa","bb",{},"20000004","1705ae3a","647025b5",true]}}"#
        ),
        merkle
    )
}

fn valid_notify_with_version(version: &str) -> String {
    format!(
        concat!(
            r#"{{"method":"mining.notify","params":["job","#,
            r#""0000000000000000000000000000000000000000000000000000000000000000","#,
            r#""aa","bb",[],{},"1705ae3a","647025b5",true]}}"#
        ),
        version
    )
}
