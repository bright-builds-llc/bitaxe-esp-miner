use std::fmt;
use std::io;

use camino::Utf8PathBuf;

mod filesystem;
mod ownership;
mod phase35;
mod phase36;
#[cfg(test)]
mod tests;

pub(crate) use phase35::{
    publish_phase35_generation, Phase35GenerationDocuments, Phase35PublicationFailurePoint,
    Phase35PublicationOptions,
};
use phase36::Phase36PublicationFailurePoint;
pub(crate) use phase36::{
    publish_phase36_generation, read_phase36_public_checklist, Phase36GenerationDocuments,
    Phase36PublicationOptions,
};
#[derive(Debug)]
pub(crate) enum GenerationError {
    InvalidInput(String),
    Io {
        action: String,
        source: io::Error,
    },
    Validation(Vec<String>),
    Phase35Injected(Phase35PublicationFailurePoint),
    Phase36Injected(Phase36PublicationFailurePoint),
    RecoveryRequired {
        destination: Utf8PathBuf,
        retained_old_generation: Utf8PathBuf,
        detail: String,
    },
}

impl fmt::Display for GenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(message) => formatter.write_str(message),
            Self::Io { action, source } => write!(formatter, "{action}: {source}"),
            Self::Validation(errors) => {
                write!(formatter, "generated operator evidence failed validation: {}", errors.join("; "))
            }
            Self::Phase35Injected(point) => {
                write!(formatter, "injected Phase 35 publication failure at {point:?}")
            }
            Self::Phase36Injected(point) => {
                write!(formatter, "injected Phase 36 publication failure at {point:?}")
            }
            Self::RecoveryRequired {
                destination,
                retained_old_generation,
                detail,
            } => write!(
                formatter,
                "phase28 promotion needs recovery; destination={destination}; retained_old_generation={retained_old_generation}; {detail}"
            ),
        }
    }
}

impl std::error::Error for GenerationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

type GenerationResult<T> = Result<T, GenerationError>;
