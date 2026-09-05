use std::fmt;

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

const PROTOCOL_VERSION: &str = "bwg-worker-controller/0.4";

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct WireStratumConfig {
    endpoint: Zeroizing<String>,
    username: Zeroizing<String>,
    password: Zeroizing<String>,
}

/// Strict authenticated Start input whose secret fields are zeroized on drop.
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WorkerLeaseGrant {
    protocol_version: String,
    lease_id: String,
    challenge_id: String,
    authorization: Zeroizing<String>,
    duration_milliseconds: u64,
    renew_after_milliseconds: u64,
    stratum: WireStratumConfig,
    #[serde(
        default,
        rename = "acceptanceCampaign",
        deserialize_with = "acceptance_campaign"
    )]
    maybe_acceptance_campaign: Option<AcceptanceCampaign>,
}

impl WorkerLeaseGrant {
    pub(crate) fn validate(&self) -> bool {
        self.protocol_version == PROTOCOL_VERSION
            && identifier(&self.lease_id)
            && identifier(&self.challenge_id)
            && secret(&self.authorization)
            && secret(&self.stratum.username)
            && secret(&self.stratum.password)
            && stratum_endpoint(&self.stratum.endpoint)
            && valid_window(self.duration_milliseconds, self.renew_after_milliseconds)
            && self
                .maybe_acceptance_campaign
                .as_ref()
                .is_none_or(AcceptanceCampaign::validate)
    }

    #[must_use]
    pub fn lease_id(&self) -> &str {
        &self.lease_id
    }

    #[must_use]
    pub fn challenge_id(&self) -> &str {
        &self.challenge_id
    }

    #[must_use]
    pub fn authorization(&self) -> &str {
        &self.authorization
    }

    #[must_use]
    pub fn stratum_endpoint(&self) -> &str {
        &self.stratum.endpoint
    }

    #[must_use]
    pub fn stratum_username(&self) -> &str {
        &self.stratum.username
    }

    #[must_use]
    pub fn stratum_password(&self) -> &str {
        &self.stratum.password
    }

    #[must_use]
    pub const fn duration_milliseconds(&self) -> u64 {
        self.duration_milliseconds
    }

    #[must_use]
    pub const fn renew_after_milliseconds(&self) -> u64 {
        self.renew_after_milliseconds
    }

    #[must_use]
    pub fn maybe_acceptance_campaign(&self) -> Option<&AcceptanceCampaign> {
        self.maybe_acceptance_campaign.as_ref()
    }

    pub(crate) fn authorizationless(&self) -> impl Serialize + '_ {
        AuthorizationlessGrant {
            maybe_acceptance_campaign: self.maybe_acceptance_campaign.as_ref(),
            challenge_id: &self.challenge_id,
            duration_milliseconds: self.duration_milliseconds,
            lease_id: &self.lease_id,
            protocol_version: &self.protocol_version,
            renew_after_milliseconds: self.renew_after_milliseconds,
            stratum: AuthorizationlessStratum {
                endpoint: &self.stratum.endpoint,
                password: &self.stratum.password,
                username: &self.stratum.username,
            },
        }
    }
}

/// Signed, durable no-refund hardware acceptance window.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AcceptanceCampaign {
    id: String,
    window: u8,
    maximum_active_milliseconds: u64,
}

impl AcceptanceCampaign {
    fn validate(&self) -> bool {
        crate::serial::canonical_nonce(&self.id, 16)
            && matches!(
                (self.window, self.maximum_active_milliseconds),
                (0, 180_000) | (1 | 2, 30_000)
            )
    }
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[must_use]
    pub const fn window(&self) -> u8 {
        self.window
    }
    #[must_use]
    pub const fn maximum_active_milliseconds(&self) -> u64 {
        self.maximum_active_milliseconds
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorizationlessGrant<'a> {
    #[serde(rename = "acceptanceCampaign", skip_serializing_if = "Option::is_none")]
    maybe_acceptance_campaign: Option<&'a AcceptanceCampaign>,
    challenge_id: &'a str,
    duration_milliseconds: u64,
    lease_id: &'a str,
    protocol_version: &'a str,
    renew_after_milliseconds: u64,
    stratum: AuthorizationlessStratum<'a>,
}

#[derive(Serialize)]
struct AuthorizationlessStratum<'a> {
    endpoint: &'a str,
    password: &'a str,
    username: &'a str,
}

impl fmt::Debug for WorkerLeaseGrant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorkerLeaseGrant")
            .field("lease_id", &self.lease_id)
            .field("challenge_id", &self.challenge_id)
            .field("authorization", &"[redacted]")
            .field("stratum", &"[redacted]")
            .finish()
    }
}

/// Strict authenticated Renew input whose secret field is zeroized on drop.
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct WorkerLeaseRenewal {
    protocol_version: String,
    lease_id: String,
    authorization: Zeroizing<String>,
    duration_milliseconds: u64,
    renew_after_milliseconds: u64,
}

impl WorkerLeaseRenewal {
    pub(crate) fn validate(&self) -> bool {
        self.protocol_version == PROTOCOL_VERSION
            && identifier(&self.lease_id)
            && secret(&self.authorization)
            && valid_window(self.duration_milliseconds, self.renew_after_milliseconds)
    }

    #[must_use]
    pub fn lease_id(&self) -> &str {
        &self.lease_id
    }

    #[must_use]
    pub fn authorization(&self) -> &str {
        &self.authorization
    }

    #[must_use]
    pub const fn duration_milliseconds(&self) -> u64 {
        self.duration_milliseconds
    }

    #[must_use]
    pub const fn renew_after_milliseconds(&self) -> u64 {
        self.renew_after_milliseconds
    }

    pub(crate) fn authorizationless(&self) -> impl Serialize + '_ {
        AuthorizationlessRenewal {
            duration_milliseconds: self.duration_milliseconds,
            lease_id: &self.lease_id,
            protocol_version: &self.protocol_version,
            renew_after_milliseconds: self.renew_after_milliseconds,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorizationlessRenewal<'a> {
    duration_milliseconds: u64,
    lease_id: &'a str,
    protocol_version: &'a str,
    renew_after_milliseconds: u64,
}

impl fmt::Debug for WorkerLeaseRenewal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorkerLeaseRenewal")
            .field("lease_id", &self.lease_id)
            .field("authorization", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LeaseDeadlines {
    renew_at_monotonic_milliseconds: u64,
    expires_at_monotonic_milliseconds: u64,
}

impl LeaseDeadlines {
    pub(crate) fn from_window(now: u64, duration: u64, renew_after: u64) -> Option<Self> {
        Some(Self {
            renew_at_monotonic_milliseconds: now.checked_add(renew_after)?,
            expires_at_monotonic_milliseconds: now.checked_add(duration)?,
        })
    }

    #[must_use]
    pub const fn renew_at_monotonic_milliseconds(self) -> u64 {
        self.renew_at_monotonic_milliseconds
    }

    #[must_use]
    pub const fn expires_at_monotonic_milliseconds(self) -> u64 {
        self.expires_at_monotonic_milliseconds
    }
}

fn valid_window(duration: u64, renew_after: u64) -> bool {
    (1..=60_000).contains(&duration)
        && (1..=20_000).contains(&renew_after)
        && renew_after < duration
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn secret(value: &str) -> bool {
    !value.is_empty() && value.len() <= 512
}

fn stratum_endpoint(value: &str) -> bool {
    let Some(authority) = value.strip_prefix("stratum+tcp://") else {
        return false;
    };
    let Some(host_port) = authority.strip_suffix('/') else {
        return false;
    };
    if host_port.contains(['@', '?', '#', '/']) {
        return false;
    }
    let Some((host, port)) = host_port.rsplit_once(':') else {
        return false;
    };
    !host.is_empty()
        && port
            .parse::<u16>()
            .is_ok_and(|parsed| parsed > 0 && parsed.to_string() == port)
}

fn acceptance_campaign<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<AcceptanceCampaign>, D::Error> {
    AcceptanceCampaign::deserialize(deserializer).map(Some)
}
