//! Strict wire shape of the role-separated deployment trust document.
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct WireDeploymentTrust {
    pub(super) profile: String,
    pub(super) update_authority: WireUpdateAuthority,
    pub(super) work_lease_authority: WireWorkLeaseAuthority,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireUpdateAuthority {
    pub(super) issuer: String,
    pub(super) audience: String,
    pub(super) role: String,
    pub(super) keys: Vec<WirePublicKey>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WireWorkLeaseAuthority {
    pub(super) profile: String,
    pub(super) issuer: String,
    pub(super) audience: String,
    pub(super) role: String,
    pub(super) keys: Vec<WirePublicKey>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WirePublicKey {
    #[serde(rename = "kid")]
    pub(super) key_id: String,
    #[serde(rename = "kty")]
    pub(super) key_type: String,
    #[serde(rename = "crv")]
    pub(super) curve: String,
    #[serde(rename = "x")]
    pub(super) public_key: String,
    #[serde(rename = "alg")]
    pub(super) algorithm: String,
    #[serde(rename = "use")]
    pub(super) use_: String,
    #[serde(rename = "key_ops")]
    pub(super) key_operations: Vec<String>,
}
