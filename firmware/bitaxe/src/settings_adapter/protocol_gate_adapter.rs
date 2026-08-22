use esp_idf_svc::nvs::{EspNvs, NvsDefault};

use super::protocol_gate::{ProductionProtocolGateDecision, ProtocolSelectorObservation};
use super::{default_nvs_partition, NVS_NAMESPACE, SETTINGS_TRANSACTION_LOCK};

pub(super) fn read() -> ProductionProtocolGateDecision {
    match read_inputs() {
        Ok((primary, fallback, _)) => {
            ProductionProtocolGateDecision::from_selectors(primary, fallback)
        }
        Err(decision) => decision,
    }
}

pub(super) fn read_plan(
) -> Result<super::protocol_gate::ConfiguredProtocolPlan, ProductionProtocolGateDecision> {
    let (primary, fallback, prefer_fallback) = read_inputs()?;
    super::protocol_gate::ConfiguredProtocolPlan::from_selectors(primary, fallback, prefer_fallback)
}

fn read_inputs() -> Result<
    (
        ProtocolSelectorObservation,
        ProtocolSelectorObservation,
        bool,
    ),
    ProductionProtocolGateDecision,
> {
    let Ok(_transaction_guard) = SETTINGS_TRANSACTION_LOCK.lock() else {
        return Err(ProductionProtocolGateDecision::TransactionUnavailable);
    };
    let partition = match default_nvs_partition() {
        Ok(partition) => partition,
        Err(_) => return Err(ProductionProtocolGateDecision::PartitionOwnerUnavailable),
    };
    let nvs = match EspNvs::new(partition, NVS_NAMESPACE, false) {
        Ok(nvs) => nvs,
        Err(_) => return Err(ProductionProtocolGateDecision::NamespaceUnavailable),
    };
    let prefer_fallback = nvs
        .get_u16("usefbstartum")
        .map_err(|_| ProductionProtocolGateDecision::NamespaceUnavailable)?
        .unwrap_or(0)
        == 1;
    Ok((
        read_selector(&nvs, "stratumprot"),
        read_selector(&nvs, "fbstratumprot"),
        prefer_fallback,
    ))
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
        Ok(Some("SV2")) => ProtocolSelectorObservation::V2,
        Ok(Some(_)) => ProtocolSelectorObservation::Unsupported,
        Ok(None) => ProtocolSelectorObservation::Missing,
        Err(_) => ProtocolSelectorObservation::Invalid,
    }
}
