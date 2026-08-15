use std::env;
use std::fs::File;

use bitaxe_automation_contracts::{
    validate_adc_observation_inputs, AdcObservationSnapshotInput, AdcObservationWebSocketInput,
    SystemInfoEvidence,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let http_path = args.next().ok_or("missing HTTP snapshot path")?;
    let websocket_path = args.next().ok_or("missing WebSocket snapshot path")?;
    let source_path = args.next().ok_or("missing system info evidence path")?;
    if args.next().is_some() {
        return Err("unexpected ADC input validator argument".into());
    }
    let http: AdcObservationSnapshotInput = serde_json::from_reader(File::open(http_path)?)?;
    let websocket: AdcObservationWebSocketInput =
        serde_json::from_reader(File::open(websocket_path)?)?;
    let source: SystemInfoEvidence = serde_json::from_reader(File::open(source_path)?)?;
    validate_adc_observation_inputs(&http, &websocket, &source).map_err(Into::into)
}
