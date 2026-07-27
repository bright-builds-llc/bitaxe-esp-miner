use serde_json::Value;

use crate::error::StratumV1Error;
use crate::jsonrpc::StratumRequestId;

pub const MAX_EXTRANONCE_2_LEN: u8 = 32;

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
