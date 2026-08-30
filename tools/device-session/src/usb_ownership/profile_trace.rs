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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct ProfileObservationCounts {
    pub absent: u16,
    pub same_worker: u16,
    pub same_serial_jtag: u16,
    pub same_unknown: u16,
    pub physical_mismatch: u16,
}

impl ProfileObservationCounts {
    pub fn merge(self, other: Self) -> Self {
        Self {
            absent: self.absent.saturating_add(other.absent),
            same_worker: self.same_worker.saturating_add(other.same_worker),
            same_serial_jtag: self.same_serial_jtag.saturating_add(other.same_serial_jtag),
            same_unknown: self.same_unknown.saturating_add(other.same_unknown),
            physical_mismatch: self
                .physical_mismatch
                .saturating_add(other.physical_mismatch),
        }
    }
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

    pub(crate) fn counts(&self) -> ProfileObservationCounts {
        let mut counts = ProfileObservationCounts::default();
        for category in &self.samples {
            let target = match category {
                ProfileObservationCategory::Absent => &mut counts.absent,
                ProfileObservationCategory::SameWorker => &mut counts.same_worker,
                ProfileObservationCategory::SameSerialJtag => &mut counts.same_serial_jtag,
                ProfileObservationCategory::SameUnknown => &mut counts.same_unknown,
                ProfileObservationCategory::PhysicalMismatch => &mut counts.physical_mismatch,
            };
            *target = target.saturating_add(1);
        }
        counts
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
