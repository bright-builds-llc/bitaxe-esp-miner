//! Single-pass profile selection avoids untagged retries and plain secret buffers.
use super::{WireStratumConfig, WireV1Stratum};
use crate::v2::{V2Profile, V2Stratum};
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;
use zeroize::Zeroizing;
const FIELDS: &[&str] = &[
    "endpoint",
    "username",
    "password",
    "suggestedDifficulty",
    "authorityPublicKey",
    "profile",
    "userIdentity",
];
impl<'de> Deserialize<'de> for WireStratumConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ProfileVisitor;
        impl<'de> Visitor<'de> for ProfileVisitor {
            type Value = WireStratumConfig;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("one closed V1 or Standard V2 stratum value")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut maybe_endpoint: Option<Zeroizing<String>> = None;
                let mut maybe_username: Option<Zeroizing<String>> = None;
                let mut maybe_password: Option<Zeroizing<String>> = None;
                let mut maybe_difficulty: Option<u16> = None;
                let mut maybe_authority: Option<Zeroizing<String>> = None;
                let mut maybe_profile: Option<V2Profile> = None;
                let mut maybe_user_identity: Option<Zeroizing<String>> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "endpoint" => read_once(&mut map, &mut maybe_endpoint, "endpoint")?,
                        "username" => read_once(&mut map, &mut maybe_username, "username")?,
                        "password" => read_once(&mut map, &mut maybe_password, "password")?,
                        "suggestedDifficulty" => {
                            read_once(&mut map, &mut maybe_difficulty, "suggestedDifficulty")?
                        }
                        "authorityPublicKey" => {
                            read_once(&mut map, &mut maybe_authority, "authorityPublicKey")?
                        }
                        "profile" => read_once(&mut map, &mut maybe_profile, "profile")?,
                        "userIdentity" => {
                            read_once(&mut map, &mut maybe_user_identity, "userIdentity")?
                        }
                        _ => return Err(de::Error::unknown_field(&key, FIELDS)),
                    }
                }
                let endpoint =
                    maybe_endpoint.ok_or_else(|| de::Error::missing_field("endpoint"))?;
                if let Some(profile) = maybe_profile {
                    if maybe_username.is_some()
                        || maybe_password.is_some()
                        || maybe_difficulty.is_some()
                    {
                        return Err(de::Error::custom("mixed stratum profiles"));
                    }
                    Ok(WireStratumConfig::V2(V2Stratum {
                        authority_public_key: maybe_authority
                            .ok_or_else(|| de::Error::missing_field("authorityPublicKey"))?,
                        endpoint,
                        profile,
                        user_identity: maybe_user_identity
                            .ok_or_else(|| de::Error::missing_field("userIdentity"))?,
                    }))
                } else {
                    if maybe_authority.is_some() || maybe_user_identity.is_some() {
                        return Err(de::Error::custom("V2 profile missing"));
                    }
                    Ok(WireStratumConfig::V1(WireV1Stratum {
                        endpoint,
                        username: maybe_username
                            .ok_or_else(|| de::Error::missing_field("username"))?,
                        password: maybe_password
                            .ok_or_else(|| de::Error::missing_field("password"))?,
                        maybe_suggested_difficulty: maybe_difficulty,
                    }))
                }
            }
        }
        deserializer.deserialize_map(ProfileVisitor)
    }
}
fn read_once<'de, M: MapAccess<'de>, T: Deserialize<'de>>(
    map: &mut M,
    maybe_value: &mut Option<T>,
    field: &'static str,
) -> Result<(), M::Error> {
    if maybe_value.is_some() {
        return Err(de::Error::duplicate_field(field));
    }
    *maybe_value = Some(map.next_value()?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_null_duplicate_and_unknown_profile_never_fall_back_to_v1() {
        // Arrange
        let bad = [
            r#"{"endpoint":"stratum+tcp://192.168.1.3:1/","username":"u","password":"p","profile":null}"#,
            r#"{"endpoint":"stratum+tcp://192.168.1.3:1/","username":"u","username":"u","password":"p"}"#,
            r#"{"endpoint":"stratum+tcp://192.168.1.3:1/","username":"u","password":"p","profile":"unknown"}"#,
            r#"{"endpoint":"stratum+tcp://192.168.1.3:1/","username":"u","password":"p","suggestedDifficulty":null}"#,
        ];
        // Act / Assert
        for input in bad {
            assert!(serde_json::from_str::<WireStratumConfig>(input).is_err());
        }
    }
}
