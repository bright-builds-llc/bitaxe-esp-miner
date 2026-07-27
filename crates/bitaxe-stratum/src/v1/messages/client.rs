use serde_json::{json, Value};

use crate::error::StratumV1Error;
use crate::jsonrpc::StratumRequestId;

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
