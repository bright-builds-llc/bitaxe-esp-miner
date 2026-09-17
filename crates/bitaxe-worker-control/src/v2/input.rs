use crate::noise::canonical_bytes;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddrV4};
use zeroize::Zeroizing;

/// Ordered fields are the canonical signed V2 stratum value; secrets never enter Debug.
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct V2Stratum {
    pub authority_public_key: Zeroizing<String>,
    pub endpoint: Zeroizing<String>,
    pub profile: V2Profile,
    pub user_identity: Zeroizing<String>,
}
#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum V2Profile {
    #[serde(rename = "bwg-worker-stratum-v2-standard/0.1")]
    Standard,
}
impl V2Stratum {
    pub fn valid(&self) -> bool {
        canonical_bytes::<32>(&self.authority_public_key).is_some()
            && self.maybe_socket().is_some()
            && !self.user_identity.is_empty()
            && self.user_identity.len() <= 255
            && !self.user_identity.contains('\0')
    }
    pub fn maybe_socket(&self) -> Option<SocketAddrV4> {
        let value = self
            .endpoint
            .strip_prefix("stratum+tcp://")?
            .strip_suffix('/')?;
        let (host, port) = value.split_once(':')?;
        let ip = private_ipv4(host)?;
        let port_value = port.parse::<u16>().ok()?;
        (port_value != 0 && port_value.to_string() == port)
            .then(|| SocketAddrV4::new(ip, port_value))
    }
}
/// Canonical private literal parser shared by admission and native observations.
pub fn private_ipv4(value: &str) -> Option<Ipv4Addr> {
    let ip = value.parse::<Ipv4Addr>().ok()?;
    (ip.is_private() && ip.to_string() == value).then_some(ip)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ChannelStart {
    pub schema: ChannelStartSchema,
    pub attempt_id: String,
    pub expected_boot_ordinal: u64,
    pub network_observed_at_us: u64,
    pub stratum: V2Stratum,
}
#[derive(Deserialize)]
pub enum ChannelStartSchema {
    #[serde(rename = "worker-stratum-v2-channel-start-v1")]
    V1,
}
impl ChannelStart {
    pub fn valid(&self) -> bool {
        canonical_bytes::<16>(&self.attempt_id).is_some()
            && self.stratum.valid()
            && self.expected_boot_ordinal > 0
            && self.expected_boot_ordinal <= crate::noise::MAX_SAFE_INTEGER
            && self.network_observed_at_us <= crate::noise::MAX_SAFE_INTEGER
    }
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Channel,
    Share,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct V2Query {
    pub schema: QuerySchema,
    pub scope: Scope,
    #[serde(rename = "attemptId", deserialize_with = "nullable")]
    pub maybe_attempt_id: Option<String>,
}
fn nullable<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(d)
}
#[derive(Deserialize)]
pub enum QuerySchema {
    #[serde(rename = "worker-stratum-v2-query-v1")]
    V1,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ChannelCancel {
    pub schema: CancelSchema,
    pub attempt_id: String,
}
#[derive(Deserialize)]
pub enum CancelSchema {
    #[serde(rename = "worker-stratum-v2-channel-cancel-v1")]
    V1,
}
