use super::UsbProfile;

pub(super) const MAX_PROFILE_OBSERVATION_SAMPLES: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProfileObservationCategory {
    Absent,
    SameWorker,
    SameSerialJtag,
    SameUnknown,
    PhysicalMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct ProfileObservationTrace {
    schema_version: &'static str,
    expected_profile: &'static str,
    samples: Vec<ProfileObservationCategory>,
    overflowed: bool,
}

impl ProfileObservationTrace {
    pub(crate) fn new(expected_profile: UsbProfile) -> Self {
        Self {
            schema_version: "bitaxe-usb-profile-observation-v1",
            expected_profile: usb_profile_label(expected_profile),
            samples: Vec::new(),
            overflowed: false,
        }
    }

    pub(crate) fn observe(&mut self, category: ProfileObservationCategory) {
        if self.samples.len() >= MAX_PROFILE_OBSERVATION_SAMPLES {
            self.overflowed = true;
            return;
        }
        self.samples.push(category);
    }

    #[cfg(test)]
    pub(crate) fn samples(&self) -> &[ProfileObservationCategory] {
        &self.samples
    }

    #[cfg(test)]
    pub(crate) const fn overflowed(&self) -> bool {
        self.overflowed
    }
}

pub(crate) const fn profile_observation_category(
    maybe_profile: Option<UsbProfile>,
    physical_identity_matches: bool,
) -> ProfileObservationCategory {
    if !physical_identity_matches {
        return ProfileObservationCategory::PhysicalMismatch;
    }
    match maybe_profile {
        None => ProfileObservationCategory::Absent,
        Some(UsbProfile::WorkerRuntime) => ProfileObservationCategory::SameWorker,
        Some(UsbProfile::SerialJtagRuntime | UsbProfile::RomDownloader) => {
            ProfileObservationCategory::SameSerialJtag
        }
        Some(UsbProfile::Unknown) => ProfileObservationCategory::SameUnknown,
    }
}

const fn usb_profile_label(profile: UsbProfile) -> &'static str {
    match profile {
        UsbProfile::WorkerRuntime => "worker_runtime",
        UsbProfile::SerialJtagRuntime => "serial_jtag_runtime",
        UsbProfile::RomDownloader => "rom_downloader",
        UsbProfile::Unknown => "unknown",
    }
}
