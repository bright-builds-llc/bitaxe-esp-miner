use esp_idf_svc::nvs::{EspNvs, NvsDefault};

use super::protocol_gate::{ProductionProtocolGateDecision, ProtocolSelectorObservation};
use super::{default_nvs_partition, NVS_NAMESPACE, SETTINGS_TRANSACTION_LOCK};

pub(super) fn read() -> ProductionProtocolGateDecision {
    let Ok(_transaction_guard) = SETTINGS_TRANSACTION_LOCK.lock() else {
        return ProductionProtocolGateDecision::TransactionUnavailable;
    };
    let partition = match default_nvs_partition() {
        Ok(partition) => partition,
        Err(_) => return ProductionProtocolGateDecision::PartitionOwnerUnavailable,
    };
    let nvs = match EspNvs::new(partition, NVS_NAMESPACE, false) {
        Ok(nvs) => nvs,
        Err(_) => return ProductionProtocolGateDecision::NamespaceUnavailable,
    };
    ProductionProtocolGateDecision::from_selectors(
        read_selector(&nvs, "stratumprot"),
        read_selector(&nvs, "fbstratumprot"),
    )
}

fn read_selector(nvs: &EspNvs<NvsDefault>, key: &str) -> ProtocolSelectorObservation {
    let maybe_len = match nvs.str_len(key) {
        Ok(maybe_len) => maybe_len,
        Err(_) => return ProtocolSelectorObservation::Invalid,
    };
    let Some(len) = maybe_len else {
        return ProtocolSelectorObservation::Missing;
    };
    let mut buffer = vec![0; len];
    match nvs.get_str(key, &mut buffer) {
        Ok(Some("SV1")) => ProtocolSelectorObservation::V1,
        Ok(Some(_)) => ProtocolSelectorObservation::Unsupported,
        Ok(None) => ProtocolSelectorObservation::Missing,
        Err(_) => ProtocolSelectorObservation::Invalid,
    }
}
