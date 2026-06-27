//! Typed Stratum v1 message parsing and serialization.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/components/stratum/stratum_api.c`
//! - `reference/esp-miner/components/stratum/include/stratum_api.h`
//! - Parity checklist row `STR-001`

use serde_json::{json, Value};

use crate::error::StratumV1Error;
use crate::jsonrpc::StratumRequestId;

pub const MAX_EXTRANONCE_2_LEN: u8 = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StratumV1ClientMessage {
    Subscribe {
        id: StratumRequestId,
        user_agent: String,
    },
    Authorize {
        id: StratumRequestId,
        username: String,
        password: String,
    },
    ConfigureVersionRolling {
        id: StratumRequestId,
        mask: u32,
    },
    SuggestDifficulty {
        id: StratumRequestId,
        difficulty: u32,
    },
    ExtranonceSubscribe {
        id: StratumRequestId,
    },
    Pong {
        id: StratumRequestId,
    },
    SendVersion {
        id: StratumRequestId,
        version: String,
    },
    SubmitShare {
        id: StratumRequestId,
        username: String,
        job_id: String,
        extranonce2: String,
        ntime: u32,
        nonce: u32,
        version_bits: u32,
    },
}

impl StratumV1ClientMessage {
    pub fn subscribe(id: StratumRequestId, model: &str, version: &str) -> Self {
        Self::Subscribe {
            id,
            user_agent: format!("bitaxe/{model}/{version}"),
        }
    }

    pub fn authorize(id: StratumRequestId, username: &str, password: &str) -> Self {
        Self::Authorize {
            id,
            username: username.to_owned(),
            password: password.to_owned(),
        }
    }

    pub const fn suggest_difficulty(id: StratumRequestId, difficulty: u32) -> Self {
        Self::SuggestDifficulty { id, difficulty }
    }

    pub const fn extranonce_subscribe(id: StratumRequestId) -> Self {
        Self::ExtranonceSubscribe { id }
    }

    pub fn submit_share(
        id: StratumRequestId,
        username: &str,
        job_id: &str,
        extranonce2: &str,
        ntime: u32,
        nonce: u32,
        version_bits: u32,
    ) -> Self {
        Self::SubmitShare {
            id,
            username: username.to_owned(),
            job_id: job_id.to_owned(),
            extranonce2: extranonce2.to_owned(),
            ntime,
            nonce,
            version_bits,
        }
    }

    pub fn to_json_line(&self) -> Result<String, StratumV1Error> {
        let value = match self {
            Self::Subscribe { id, user_agent } => {
                request_value(*id, "mining.subscribe", json!([user_agent]))
            }
            Self::Authorize {
                id,
                username,
                password,
            } => request_value(*id, "mining.authorize", json!([username, password])),
            Self::ConfigureVersionRolling { id, mask } => request_value(
                *id,
                "mining.configure",
                json!([
                    ["version-rolling"],
                    {"version-rolling.mask": format!("{mask:08x}")}
                ]),
            ),
            Self::SuggestDifficulty { id, difficulty } => {
                request_value(*id, "mining.suggest_difficulty", json!([difficulty]))
            }
            Self::ExtranonceSubscribe { id } => {
                request_value(*id, "mining.extranonce.subscribe", json!([]))
            }
            Self::Pong { id } => request_value(*id, "pong", json!([])),
            Self::SendVersion { id, version } => json!({
                "id": id.raw(),
                "result": version,
                "error": null
            }),
            Self::SubmitShare {
                id,
                username,
                job_id,
                extranonce2,
                ntime,
                nonce,
                version_bits,
            } => request_value(
                *id,
                "mining.submit",
                json!([
                    username,
                    job_id,
                    extranonce2,
                    format!("{ntime:08x}"),
                    format!("{nonce:08x}"),
                    format!("{version_bits:08x}")
                ]),
            ),
        };

        let mut line =
            serde_json::to_string(&value).map_err(|_| StratumV1Error::SerializationFailed)?;
        line.push('\n');
        Ok(line)
    }
}

fn request_value(id: StratumRequestId, method: &'static str, params: Value) -> Value {
    json!({
        "id": id.raw(),
        "method": method,
        "params": params
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum StratumV1ServerMessage {
    Notify(MiningNotify),
    SetDifficulty(PoolDifficulty),
    SetExtranonce(ExtranonceAssignment),
    SetVersionMask(VersionMask),
    Response(StratumResponse),
    ClientReconnect,
    ClientShowMessage(String),
    ClientGetVersion,
    Ping { maybe_id: Option<StratumRequestId> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiningNotify {
    pub job_id: String,
    pub prev_block_hash: String,
    pub coinbase_1: String,
    pub coinbase_2: String,
    pub merkle_branches: Vec<String>,
    pub version: u32,
    pub nbits: u32,
    pub ntime: u32,
    pub clean_jobs: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PoolDifficulty {
    pub difficulty: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtranonceAssignment {
    pub extranonce1: String,
    pub extranonce2_len: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionMask {
    pub mask: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StratumResponse {
    pub maybe_id: Option<StratumRequestId>,
    pub success: bool,
    pub maybe_error: Option<StratumResponseError>,
    pub maybe_extranonce: Option<ExtranonceAssignment>,
    pub maybe_version_mask: Option<VersionMask>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StratumResponseError {
    pub maybe_code: Option<i64>,
    pub message: String,
}

pub fn parse_server_message(input: &str) -> Result<StratumV1ServerMessage, StratumV1Error> {
    let value: Value = serde_json::from_str(input).map_err(|_| StratumV1Error::InvalidJson)?;
    let Value::Object(root) = &value else {
        return Err(StratumV1Error::InvalidJson);
    };

    let maybe_id = parse_request_id(root.get("id"))?;
    let maybe_method = root.get("method");
    let Some(method_value) = maybe_method else {
        return parse_response(&value, maybe_id);
    };
    let Some(method) = method_value.as_str() else {
        return Err(StratumV1Error::InvalidField {
            field: "method",
            reason: "expected string",
        });
    };

    match method {
        "mining.notify" => Ok(StratumV1ServerMessage::Notify(parse_mining_notify(&value)?)),
        "mining.set_difficulty" => Ok(StratumV1ServerMessage::SetDifficulty(parse_set_difficulty(
            &value,
        )?)),
        "mining.set_extranonce" => Ok(StratumV1ServerMessage::SetExtranonce(parse_set_extranonce(
            &value,
            "mining.set_extranonce",
        )?)),
        "mining.set_version_mask" => Ok(StratumV1ServerMessage::SetVersionMask(
            parse_set_version_mask(&value)?,
        )),
        "client.reconnect" => {
            ensure_params_absent_or_array(&value, "client.reconnect")?;
            Ok(StratumV1ServerMessage::ClientReconnect)
        }
        "client.show_message" => Ok(StratumV1ServerMessage::ClientShowMessage(
            parse_show_message(&value)?,
        )),
        "client.get_version" => {
            ensure_params_absent_or_array(&value, "client.get_version")?;
            Ok(StratumV1ServerMessage::ClientGetVersion)
        }
        "mining.ping" => {
            ensure_params_absent_or_array(&value, "mining.ping")?;
            Ok(StratumV1ServerMessage::Ping { maybe_id })
        }
        _ => Err(StratumV1Error::UnknownMethod {
            method: method.to_owned(),
        }),
    }
}

fn parse_request_id(
    maybe_value: Option<&Value>,
) -> Result<Option<StratumRequestId>, StratumV1Error> {
    let Some(value) = maybe_value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let Some(raw) = value.as_u64() else {
        return Err(StratumV1Error::InvalidField {
            field: "id",
            reason: "expected non-negative integer or null",
        });
    };

    Ok(Some(StratumRequestId::new(raw)))
}

fn parse_response(
    root: &Value,
    maybe_id: Option<StratumRequestId>,
) -> Result<StratumV1ServerMessage, StratumV1Error> {
    let maybe_error = parse_response_error(root)?;
    if let Some(error) = maybe_error {
        return Ok(StratumV1ServerMessage::Response(StratumResponse {
            maybe_id,
            success: false,
            maybe_error: Some(error),
            maybe_extranonce: None,
            maybe_version_mask: None,
        }));
    }

    let result = root
        .get("result")
        .ok_or(StratumV1Error::MissingField("result"))?;
    match result {
        Value::Bool(success) => Ok(StratumV1ServerMessage::Response(StratumResponse {
            maybe_id,
            success: *success,
            maybe_error: response_error_for_false_result(root, *success),
            maybe_extranonce: None,
            maybe_version_mask: None,
        })),
        Value::Array(_) => Ok(StratumV1ServerMessage::Response(StratumResponse {
            maybe_id,
            success: true,
            maybe_error: None,
            maybe_extranonce: Some(parse_subscribe_result(root)?),
            maybe_version_mask: None,
        })),
        Value::Object(_) => Ok(StratumV1ServerMessage::Response(StratumResponse {
            maybe_id,
            success: true,
            maybe_error: None,
            maybe_extranonce: None,
            maybe_version_mask: Some(parse_configure_result(root)?),
        })),
        _ => Err(StratumV1Error::InvalidParams { method: "response" }),
    }
}

fn response_error_for_false_result(root: &Value, success: bool) -> Option<StratumResponseError> {
    if success {
        return None;
    }

    let message = root
        .get("reject-reason")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_owned();
    Some(StratumResponseError {
        maybe_code: None,
        message,
    })
}

fn parse_response_error(root: &Value) -> Result<Option<StratumResponseError>, StratumV1Error> {
    let Some(error) = root.get("error") else {
        return Ok(None);
    };
    if error.is_null() {
        return Ok(None);
    }

    if let Some(message) = error.as_str() {
        return Ok(Some(StratumResponseError {
            maybe_code: None,
            message: message.to_owned(),
        }));
    }

    if let Some(array) = error.as_array() {
        let Some(message_value) = array.get(1) else {
            return Err(StratumV1Error::InvalidParams { method: "response" });
        };
        let Some(message) = message_value.as_str() else {
            return Err(StratumV1Error::InvalidParams { method: "response" });
        };
        let maybe_code = array.first().and_then(Value::as_i64);
        return Ok(Some(StratumResponseError {
            maybe_code,
            message: message.to_owned(),
        }));
    }

    if let Some(object) = error.as_object() {
        let Some(message) = object.get("message").and_then(Value::as_str) else {
            return Err(StratumV1Error::InvalidParams { method: "response" });
        };
        let maybe_code = object.get("code").and_then(Value::as_i64);
        return Ok(Some(StratumResponseError {
            maybe_code,
            message: message.to_owned(),
        }));
    }

    Err(StratumV1Error::InvalidParams { method: "response" })
}

fn parse_mining_notify(root: &Value) -> Result<MiningNotify, StratumV1Error> {
    let params = params_array(root, "mining.notify")?;
    if params.len() < 8 {
        return Err(StratumV1Error::InvalidParams {
            method: "mining.notify",
        });
    }

    let job_id = param_string(params, 0, "mining.notify")?;
    let prev_block_hash = param_string(params, 1, "mining.notify")?;
    let coinbase_1 = param_string(params, 2, "mining.notify")?;
    let coinbase_2 = param_string(params, 3, "mining.notify")?;
    let merkle_branches = merkle_branches(params.get(4))?;
    let version = parse_hex_u32(&params[5], "version", "mining.notify")?;
    let nbits = parse_hex_u32(&params[6], "nbits", "mining.notify")?;
    let ntime = parse_hex_u32(&params[7], "ntime", "mining.notify")?;
    let Some(clean_jobs) = params.last().and_then(Value::as_bool) else {
        return Err(StratumV1Error::InvalidParams {
            method: "mining.notify",
        });
    };

    Ok(MiningNotify {
        job_id,
        prev_block_hash,
        coinbase_1,
        coinbase_2,
        merkle_branches,
        version,
        nbits,
        ntime,
        clean_jobs,
    })
}

fn parse_set_difficulty(root: &Value) -> Result<PoolDifficulty, StratumV1Error> {
    let params = params_array(root, "mining.set_difficulty")?;
    let Some(difficulty) = params.first().and_then(Value::as_f64) else {
        return Err(StratumV1Error::InvalidParams {
            method: "mining.set_difficulty",
        });
    };

    Ok(PoolDifficulty { difficulty })
}

fn parse_set_extranonce(
    root: &Value,
    method: &'static str,
) -> Result<ExtranonceAssignment, StratumV1Error> {
    let params = params_array(root, method)?;
    if params.len() < 2 {
        return Err(StratumV1Error::InvalidParams { method });
    }
    let extranonce1 = param_string(params, 0, method)?;
    let extranonce2_len = parse_extranonce2_len(&params[1])?;

    Ok(ExtranonceAssignment {
        extranonce1,
        extranonce2_len,
    })
}

fn parse_set_version_mask(root: &Value) -> Result<VersionMask, StratumV1Error> {
    let params = params_array(root, "mining.set_version_mask")?;
    let Some(mask_value) = params.first() else {
        return Err(StratumV1Error::InvalidParams {
            method: "mining.set_version_mask",
        });
    };
    let mask = parse_hex_u32(mask_value, "version_mask", "mining.set_version_mask")?;

    Ok(VersionMask { mask })
}

fn parse_show_message(root: &Value) -> Result<String, StratumV1Error> {
    let params = params_array(root, "client.show_message")?;
    param_string(params, 0, "client.show_message")
}

fn parse_subscribe_result(root: &Value) -> Result<ExtranonceAssignment, StratumV1Error> {
    let result = root
        .get("result")
        .and_then(Value::as_array)
        .ok_or(StratumV1Error::InvalidParams { method: "response" })?;
    if result.len() < 3 {
        return Err(StratumV1Error::InvalidParams { method: "response" });
    }
    let Some(extranonce1) = result.get(1).and_then(Value::as_str) else {
        return Err(StratumV1Error::InvalidParams { method: "response" });
    };
    let extranonce2_len = parse_extranonce2_len(&result[2])?;

    Ok(ExtranonceAssignment {
        extranonce1: extranonce1.to_owned(),
        extranonce2_len,
    })
}

fn parse_configure_result(root: &Value) -> Result<VersionMask, StratumV1Error> {
    let result = root
        .get("result")
        .and_then(Value::as_object)
        .ok_or(StratumV1Error::InvalidParams { method: "response" })?;
    let enabled = result
        .get("version-rolling")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !enabled {
        return Err(StratumV1Error::InvalidParams { method: "response" });
    }
    let Some(mask_value) = result.get("version-rolling.mask") else {
        return Err(StratumV1Error::InvalidParams { method: "response" });
    };
    let mask = parse_hex_u32(mask_value, "version_mask", "response")?;

    Ok(VersionMask { mask })
}

fn params_array<'a>(root: &'a Value, method: &'static str) -> Result<&'a [Value], StratumV1Error> {
    let params = root
        .get("params")
        .and_then(Value::as_array)
        .ok_or(StratumV1Error::InvalidParams { method })?;

    Ok(params.as_slice())
}

fn ensure_params_absent_or_array(root: &Value, method: &'static str) -> Result<(), StratumV1Error> {
    let Some(params) = root.get("params") else {
        return Ok(());
    };
    if params.is_array() {
        return Ok(());
    }

    Err(StratumV1Error::InvalidParams { method })
}

fn param_string(
    params: &[Value],
    index: usize,
    method: &'static str,
) -> Result<String, StratumV1Error> {
    let Some(value) = params.get(index).and_then(Value::as_str) else {
        return Err(StratumV1Error::InvalidParams { method });
    };

    Ok(value.to_owned())
}

fn merkle_branches(maybe_value: Option<&Value>) -> Result<Vec<String>, StratumV1Error> {
    let Some(Value::Array(branches)) = maybe_value else {
        return Err(StratumV1Error::InvalidParams {
            method: "mining.notify",
        });
    };

    let mut parsed = Vec::with_capacity(branches.len());
    for branch in branches {
        let Some(raw) = branch.as_str() else {
            return Err(StratumV1Error::InvalidParams {
                method: "mining.notify",
            });
        };
        parsed.push(raw.to_owned());
    }

    Ok(parsed)
}

fn parse_hex_u32(
    value: &Value,
    field: &'static str,
    method: &'static str,
) -> Result<u32, StratumV1Error> {
    let Some(raw) = value.as_str() else {
        return Err(StratumV1Error::InvalidParams { method });
    };

    u32::from_str_radix(raw, 16).map_err(|_| StratumV1Error::InvalidField {
        field,
        reason: "expected lowercase or uppercase hexadecimal u32",
    })
}

fn parse_extranonce2_len(value: &Value) -> Result<u8, StratumV1Error> {
    let Some(raw) = value.as_u64() else {
        return Err(StratumV1Error::InvalidField {
            field: "extranonce2_len",
            reason: "expected non-negative integer",
        });
    };
    if raw > u64::from(MAX_EXTRANONCE_2_LEN) {
        return Err(StratumV1Error::InvalidField {
            field: "extranonce2_len",
            reason: "exceeds MAX_EXTRANONCE_2_LEN 32",
        });
    }

    Ok(raw as u8)
}

#[cfg(test)]
mod tests {
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
        let set_extranonce =
            r#"{"id":null,"method":"mining.set_extranonce","params":["deadbeef",8]}"#;
        let set_version_mask =
            r#"{"id":1,"method":"mining.set_version_mask","params":["1fffe000"]}"#;
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
}
