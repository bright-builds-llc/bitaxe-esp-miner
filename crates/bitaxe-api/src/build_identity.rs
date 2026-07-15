//! Canonical build provenance shared by host tooling and firmware projections.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

pub const BUILD_PROVENANCE_SCHEMA_VERSION: u32 = 1;
pub const BUILD_LABEL_MAX_BYTES: usize = 22;
pub const FULL_COMMIT_BYTES: usize = 40;
pub const SHORT_COMMIT_BYTES: usize = 12;
const MAX_SEMANTIC_VERSION_BYTES: usize = 31;
const MAX_RELEASE_TAG_BYTES: usize = 31;
const UNAVAILABLE: &str = "unavailable";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildChannel {
    Release,
    Dev,
}

impl BuildChannel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::Dev => "dev",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildIdentity {
    source_commit: String,
    short_commit: String,
    build_label: String,
    build_channel: BuildChannel,
    source_dirty: bool,
    maybe_release_tag: Option<String>,
}

impl BuildIdentity {
    pub fn new(
        source_commit: impl Into<String>,
        source_dirty: bool,
        maybe_release_tag: Option<impl Into<String>>,
    ) -> Result<Self, BuildIdentityError> {
        let source_commit = source_commit.into();
        validate_commit("source_commit", &source_commit)?;
        let maybe_release_tag = maybe_release_tag.map(Into::into);
        if let Some(release_tag) = &maybe_release_tag {
            validate_release_tag(release_tag)?;
        }

        let short_commit = source_commit[..SHORT_COMMIT_BYTES].to_owned();
        let build_channel = if maybe_release_tag.is_some() {
            BuildChannel::Release
        } else {
            BuildChannel::Dev
        };
        let mut build_label = short_commit.clone();
        if source_dirty {
            build_label.push_str("-dirty");
        }
        if build_channel == BuildChannel::Dev {
            build_label.push_str("-dev");
        }
        if build_label.len() > BUILD_LABEL_MAX_BYTES {
            return Err(BuildIdentityError::new("build_label exceeds 22 bytes"));
        }

        Ok(Self {
            source_commit,
            short_commit,
            build_label,
            build_channel,
            source_dirty,
            maybe_release_tag,
        })
    }

    pub fn source_commit(&self) -> &str {
        &self.source_commit
    }

    pub fn short_commit(&self) -> &str {
        &self.short_commit
    }

    pub fn build_label(&self) -> &str {
        &self.build_label
    }

    pub const fn build_channel(&self) -> BuildChannel {
        self.build_channel
    }

    pub const fn source_dirty(&self) -> bool {
        self.source_dirty
    }

    pub fn maybe_release_tag(&self) -> Option<&str> {
        self.maybe_release_tag.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildProvenance {
    semantic_version: String,
    build_identity: BuildIdentity,
    reference_commit: String,
}

impl BuildProvenance {
    pub fn new(
        semantic_version: impl Into<String>,
        source_commit: impl Into<String>,
        source_dirty: bool,
        maybe_release_tag: Option<impl Into<String>>,
        reference_commit: impl Into<String>,
    ) -> Result<Self, BuildIdentityError> {
        let semantic_version = semantic_version.into();
        validate_semantic_version(&semantic_version)?;
        let build_identity = BuildIdentity::new(source_commit, source_dirty, maybe_release_tag)?;
        let reference_commit = reference_commit.into();
        validate_commit("reference_commit", &reference_commit)?;

        Ok(Self {
            semantic_version,
            build_identity,
            reference_commit,
        })
    }

    pub fn semantic_version(&self) -> &str {
        &self.semantic_version
    }

    pub const fn build_identity(&self) -> &BuildIdentity {
        &self.build_identity
    }

    pub fn reference_commit(&self) -> &str {
        &self.reference_commit
    }

    pub fn render_stamp(&self) -> String {
        let release_tag = self
            .build_identity
            .maybe_release_tag()
            .unwrap_or(UNAVAILABLE);
        format!(
            "schema_version={BUILD_PROVENANCE_SCHEMA_VERSION}\nsemantic_version={}\nsource_commit={}\nshort_commit={}\nbuild_label={}\nbuild_channel={}\nsource_dirty={}\nrelease_tag={}\nreference_commit={}\n",
            self.semantic_version,
            self.build_identity.source_commit(),
            self.build_identity.short_commit(),
            self.build_identity.build_label(),
            self.build_identity.build_channel().as_str(),
            self.build_identity.source_dirty(),
            release_tag,
            self.reference_commit,
        )
    }

    pub fn runtime_identity_record(&self) -> String {
        let identity = self.build_identity();
        format!(
            "runtime_build_identity semantic_version={} label={} channel={} source_dirty={} release_tag={} redacted=true",
            self.semantic_version(),
            identity.build_label(),
            identity.build_channel().as_str(),
            identity.source_dirty(),
            identity.maybe_release_tag().unwrap_or(UNAVAILABLE),
        )
    }

    pub fn parse_stamp(stamp: &str) -> Result<Self, BuildIdentityError> {
        if !stamp.is_ascii() {
            return Err(BuildIdentityError::new("provenance stamp must be ASCII"));
        }

        let mut fields = BTreeMap::new();
        for line in stamp.lines() {
            if line.is_empty() {
                return Err(BuildIdentityError::new(
                    "provenance stamp contains an empty line",
                ));
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(BuildIdentityError::new(
                    "provenance stamp line is missing '='",
                ));
            };
            if !is_stamp_key(key) {
                return Err(BuildIdentityError::new(format!(
                    "unknown provenance field {key}"
                )));
            }
            if value.contains('=') || value.is_empty() {
                return Err(BuildIdentityError::new(format!(
                    "invalid value for provenance field {key}"
                )));
            }
            if fields.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(BuildIdentityError::new(format!(
                    "duplicate provenance field {key}"
                )));
            }
        }

        let schema_version = take_field(&mut fields, "schema_version")?;
        if schema_version != BUILD_PROVENANCE_SCHEMA_VERSION.to_string() {
            return Err(BuildIdentityError::new(
                "unsupported provenance schema_version",
            ));
        }
        let semantic_version = take_field(&mut fields, "semantic_version")?;
        let source_commit = take_field(&mut fields, "source_commit")?;
        let short_commit = take_field(&mut fields, "short_commit")?;
        let build_label = take_field(&mut fields, "build_label")?;
        let build_channel = take_field(&mut fields, "build_channel")?;
        let source_dirty = parse_bool(&take_field(&mut fields, "source_dirty")?)?;
        let release_tag = take_field(&mut fields, "release_tag")?;
        let reference_commit = take_field(&mut fields, "reference_commit")?;
        if !fields.is_empty() {
            return Err(BuildIdentityError::new(
                "provenance stamp contains unconsumed fields",
            ));
        }

        let maybe_release_tag = if release_tag == UNAVAILABLE {
            None
        } else {
            Some(release_tag)
        };
        let provenance = Self::new(
            semantic_version,
            source_commit,
            source_dirty,
            maybe_release_tag,
            reference_commit,
        )?;
        let identity = provenance.build_identity();
        if short_commit != identity.short_commit()
            || build_label != identity.build_label()
            || build_channel != identity.build_channel().as_str()
        {
            return Err(BuildIdentityError::new(
                "provenance stamp contains contradictory derived fields",
            ));
        }

        Ok(provenance)
    }

    pub fn parse_workspace_status(status: &str) -> Result<Self, BuildIdentityError> {
        const EXPECTED_KEYS: [&str; 5] = [
            "STABLE_BITAXE_SOURCE_COMMIT",
            "STABLE_BITAXE_SOURCE_DIRTY",
            "STABLE_BITAXE_RELEASE_TAG",
            "STABLE_BITAXE_SEMANTIC_VERSION",
            "STABLE_BITAXE_REFERENCE_COMMIT",
        ];

        let mut fields = BTreeMap::new();
        for line in status.lines() {
            let mut parts = line.split_ascii_whitespace();
            let Some(key) = parts.next() else {
                continue;
            };
            if key.starts_with("STABLE_BITAXE_") && !EXPECTED_KEYS.contains(&key) {
                return Err(BuildIdentityError::new(format!(
                    "unknown Bitaxe workspace status key {key}"
                )));
            }
            if !EXPECTED_KEYS.contains(&key) {
                continue;
            }
            let Some(value) = parts.next() else {
                return Err(BuildIdentityError::new(format!(
                    "workspace status key {key} has no value"
                )));
            };
            if parts.next().is_some() {
                return Err(BuildIdentityError::new(format!(
                    "workspace status key {key} has multiple values"
                )));
            }
            if fields.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(BuildIdentityError::new(format!(
                    "duplicate Bitaxe workspace status key {key}"
                )));
            }
        }

        let source_commit = take_field(&mut fields, "STABLE_BITAXE_SOURCE_COMMIT")?;
        let source_dirty = parse_bool(&take_field(&mut fields, "STABLE_BITAXE_SOURCE_DIRTY")?)?;
        let release_tag = take_field(&mut fields, "STABLE_BITAXE_RELEASE_TAG")?;
        let maybe_release_tag = (release_tag != UNAVAILABLE).then_some(release_tag);
        let semantic_version = take_field(&mut fields, "STABLE_BITAXE_SEMANTIC_VERSION")?;
        let reference_commit = take_field(&mut fields, "STABLE_BITAXE_REFERENCE_COMMIT")?;

        Self::new(
            semantic_version,
            source_commit,
            source_dirty,
            maybe_release_tag,
            reference_commit,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildIdentityError {
    message: String,
}

impl BuildIdentityError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for BuildIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for BuildIdentityError {}

fn validate_commit(field: &str, value: &str) -> Result<(), BuildIdentityError> {
    if value.len() != FULL_COMMIT_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(BuildIdentityError::new(format!(
            "{field} must be 40 lowercase hexadecimal characters"
        )));
    }
    Ok(())
}

fn validate_release_tag(value: &str) -> Result<(), BuildIdentityError> {
    if value.len() > MAX_RELEASE_TAG_BYTES || !value.starts_with('v') {
        return Err(BuildIdentityError::new("invalid release_tag"));
    }
    let numeric = &value[1..];
    let segments: Vec<_> = numeric.split('.').collect();
    if !(segments.len() == 2 || segments.len() == 3)
        || segments
            .iter()
            .any(|segment| segment.is_empty() || !segment.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(BuildIdentityError::new("invalid release_tag"));
    }
    Ok(())
}

fn validate_semantic_version(value: &str) -> Result<(), BuildIdentityError> {
    if value.is_empty()
        || value.len() > MAX_SEMANTIC_VERSION_BYTES
        || !value.is_ascii()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+'))
        || !value.bytes().any(|byte| byte == b'.')
    {
        return Err(BuildIdentityError::new("invalid semantic_version"));
    }
    Ok(())
}

fn is_stamp_key(key: &str) -> bool {
    matches!(
        key,
        "schema_version"
            | "semantic_version"
            | "source_commit"
            | "short_commit"
            | "build_label"
            | "build_channel"
            | "source_dirty"
            | "release_tag"
            | "reference_commit"
    )
}

fn take_field(
    fields: &mut BTreeMap<String, String>,
    key: &str,
) -> Result<String, BuildIdentityError> {
    fields
        .remove(key)
        .ok_or_else(|| BuildIdentityError::new(format!("missing provenance field {key}")))
}

fn parse_bool(value: &str) -> Result<bool, BuildIdentityError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(BuildIdentityError::new(
            "source_dirty must be true or false",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{BuildChannel, BuildIdentity, BuildProvenance, BUILD_LABEL_MAX_BYTES};

    const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
    const REFERENCE_COMMIT: &str = "abcdef0123456789abcdef0123456789abcdef01";

    #[test]
    fn build_identity_derives_all_four_label_states() {
        // Arrange
        let cases = [
            (false, Some("v1.2"), "0123456789ab", BuildChannel::Release),
            (
                true,
                Some("v1.2.3"),
                "0123456789ab-dirty",
                BuildChannel::Release,
            ),
            (false, None, "0123456789ab-dev", BuildChannel::Dev),
            (true, None, "0123456789ab-dirty-dev", BuildChannel::Dev),
        ];

        for (dirty, maybe_tag, expected_label, expected_channel) in cases {
            // Act
            let identity =
                BuildIdentity::new(SOURCE_COMMIT, dirty, maybe_tag).expect("valid build identity");

            // Assert
            assert_eq!(identity.build_label(), expected_label);
            assert_eq!(identity.build_channel(), expected_channel);
            assert!(identity.build_label().len() <= BUILD_LABEL_MAX_BYTES);
        }
    }

    #[test]
    fn build_identity_rejects_malformed_commit_and_release_tags() {
        // Arrange
        let malformed_commit = "ABCDEF0123456789abcdef0123456789abcdef01";
        let malformed_tags = ["1.2", "v1", "v1.2.3.4", "v1.x"];

        // Act
        let commit_result = BuildIdentity::new(malformed_commit, false, Some("v1.2"));

        // Assert
        assert!(commit_result.is_err());
        for tag in malformed_tags {
            assert!(BuildIdentity::new(SOURCE_COMMIT, false, Some(tag)).is_err());
        }
    }

    #[test]
    fn provenance_stamp_round_trips_canonical_fields() {
        // Arrange
        let provenance =
            BuildProvenance::new("0.1.0", SOURCE_COMMIT, true, None::<&str>, REFERENCE_COMMIT)
                .expect("valid provenance");

        // Act
        let parsed = BuildProvenance::parse_stamp(&provenance.render_stamp())
            .expect("canonical stamp parses");

        // Assert
        assert_eq!(parsed, provenance);
    }

    #[test]
    fn provenance_renders_exact_retained_runtime_identity_record() {
        // Arrange
        let provenance =
            BuildProvenance::new("0.1.0", SOURCE_COMMIT, true, None::<&str>, REFERENCE_COMMIT)
                .expect("valid provenance");

        // Act
        let record = provenance.runtime_identity_record();

        // Assert
        assert_eq!(
            record,
            "runtime_build_identity semantic_version=0.1.0 label=0123456789ab-dirty-dev channel=dev source_dirty=true release_tag=unavailable redacted=true"
        );
        assert!(!record.contains(SOURCE_COMMIT));
        assert!(!record.contains(REFERENCE_COMMIT));
    }

    #[test]
    fn provenance_stamp_rejects_contradictory_label() {
        // Arrange
        let stamp = BuildProvenance::new(
            "0.1.0",
            SOURCE_COMMIT,
            false,
            None::<&str>,
            REFERENCE_COMMIT,
        )
        .expect("valid provenance")
        .render_stamp()
        .replace("build_label=0123456789ab-dev", "build_label=0123456789ab");

        // Act
        let result = BuildProvenance::parse_stamp(&stamp);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn provenance_stamp_rejects_missing_duplicate_unknown_and_non_ascii_fields() {
        // Arrange
        let stamp = BuildProvenance::new(
            "0.1.0",
            SOURCE_COMMIT,
            false,
            Some("v1.2"),
            REFERENCE_COMMIT,
        )
        .expect("valid provenance")
        .render_stamp();
        let missing = stamp.replace("semantic_version=0.1.0\n", "");
        let duplicate = format!("{stamp}source_dirty=false\n");
        let unknown = format!("{stamp}branch=main\n");
        let non_ascii = stamp.replace("semantic_version=0.1.0", "semantic_version=β");

        // Act / Assert
        assert!(BuildProvenance::parse_stamp(&missing).is_err());
        assert!(BuildProvenance::parse_stamp(&duplicate).is_err());
        assert!(BuildProvenance::parse_stamp(&unknown).is_err());
        assert!(BuildProvenance::parse_stamp(&non_ascii).is_err());
    }

    #[test]
    fn provenance_stamp_rejects_unsupported_schema_and_invalid_boolean() {
        // Arrange
        let stamp = BuildProvenance::new(
            "0.1.0",
            SOURCE_COMMIT,
            false,
            None::<&str>,
            REFERENCE_COMMIT,
        )
        .expect("valid provenance")
        .render_stamp();

        // Act
        let unsupported = stamp.replace("schema_version=1", "schema_version=2");
        let invalid_bool = stamp.replace("source_dirty=false", "source_dirty=0");

        // Assert
        assert!(BuildProvenance::parse_stamp(&unsupported).is_err());
        assert!(BuildProvenance::parse_stamp(&invalid_bool).is_err());
    }

    #[test]
    fn workspace_status_parses_only_the_canonical_stable_keys() {
        // Arrange
        let status = format!(
            "BUILD_USER local\nSTABLE_BITAXE_SOURCE_COMMIT {SOURCE_COMMIT}\nSTABLE_BITAXE_SOURCE_DIRTY false\nSTABLE_BITAXE_RELEASE_TAG unavailable\nSTABLE_BITAXE_SEMANTIC_VERSION 0.1.0\nSTABLE_BITAXE_REFERENCE_COMMIT {REFERENCE_COMMIT}\n"
        );

        // Act
        let provenance =
            BuildProvenance::parse_workspace_status(&status).expect("valid workspace status");

        // Assert
        assert_eq!(provenance.build_identity().source_commit(), SOURCE_COMMIT);
        assert_eq!(
            provenance.build_identity().build_label(),
            "0123456789ab-dev"
        );
        assert!(!provenance.build_identity().source_dirty());
    }
}
