//! Pure native-USB profile and operation planning.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbProfile {
    WorkerRuntime,
    SerialJtagRuntime,
    RomDownloader,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbIntent {
    Inspect,
    Flash,
    Observe,
    Recover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbOperationPlan {
    InspectOnly,
    ObserveOnly,
    DirectEspflash,
    HandoffThenEspflash,
    RejectUnknownProfile,
}

#[must_use]
pub const fn plan_usb_operation(intent: UsbIntent, profile: UsbProfile) -> UsbOperationPlan {
    match (intent, profile) {
        (UsbIntent::Inspect, UsbProfile::Unknown) => UsbOperationPlan::RejectUnknownProfile,
        (UsbIntent::Inspect, _) => UsbOperationPlan::InspectOnly,
        (UsbIntent::Observe, UsbProfile::WorkerRuntime | UsbProfile::SerialJtagRuntime) => {
            UsbOperationPlan::ObserveOnly
        }
        (UsbIntent::Flash | UsbIntent::Recover, UsbProfile::WorkerRuntime) => {
            UsbOperationPlan::HandoffThenEspflash
        }
        (
            UsbIntent::Flash | UsbIntent::Recover,
            UsbProfile::SerialJtagRuntime | UsbProfile::RomDownloader,
        ) => UsbOperationPlan::DirectEspflash,
        _ => UsbOperationPlan::RejectUnknownProfile,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worker_runtime_flash_requires_handoff_before_espflash() {
        // Arrange / Act
        let plan = plan_usb_operation(UsbIntent::Flash, UsbProfile::WorkerRuntime);

        // Assert
        assert_eq!(plan, UsbOperationPlan::HandoffThenEspflash);
    }

    #[test]
    fn monitoring_never_arms_handoff() {
        // Arrange / Act
        let worker = plan_usb_operation(UsbIntent::Observe, UsbProfile::WorkerRuntime);
        let serial = plan_usb_operation(UsbIntent::Observe, UsbProfile::SerialJtagRuntime);

        // Assert
        assert_eq!(worker, UsbOperationPlan::ObserveOnly);
        assert_eq!(serial, UsbOperationPlan::ObserveOnly);
    }
}
