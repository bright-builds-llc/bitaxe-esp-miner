//! The board a package targets (ADR-0041): its file names and the metadata its manifest records.

use crate::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoardId {
    Ultra205,
    Gamma601,
}

impl FromStr for BoardId {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "205" => Ok(Self::Ultra205),
            "601" => Ok(Self::Gamma601),
            other => Err(format!("unsupported board {other}; boards are 205 and 601")),
        }
    }
}

impl fmt::Display for BoardId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Ultra205 => "205",
            Self::Gamma601 => "601",
        })
    }
}

impl BoardId {
    pub(crate) const fn default_elf_name(self) -> &'static str {
        match self {
            Self::Ultra205 => DEFAULT_ELF_NAME,
            Self::Gamma601 => "bitaxe-gamma601.elf",
        }
    }

    pub(crate) const fn factory_image_name(self) -> &'static str {
        match self {
            Self::Ultra205 => FACTORY_IMAGE_NAME,
            Self::Gamma601 => "bitaxe-gamma601-factory.bin",
        }
    }

    pub(crate) const fn device_model(self) -> &'static str {
        match self {
            Self::Ultra205 => "Ultra 205",
            Self::Gamma601 => "Gamma 601",
        }
    }

    pub(crate) const fn asic(self) -> &'static str {
        match self {
            Self::Ultra205 => "BM1366",
            Self::Gamma601 => "BM1370",
        }
    }

    pub(crate) const fn install_summary(self) -> &'static str {
        match self {
            Self::Ultra205 => "Ultra 205 release operator guide",
            Self::Gamma601 => "Gamma 601 safe-boot development notes",
        }
    }
}

pub(crate) fn parse_board(value: &str) -> std::result::Result<BoardId, String> {
    value.parse()
}
