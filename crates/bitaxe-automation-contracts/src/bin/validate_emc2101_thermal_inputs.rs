use std::env;
use std::fs::File;

use bitaxe_automation_contracts::{
    validate_emc2101_thermal_inputs, Emc2101ThermalSnapshotInput, Emc2101ThermalWebSocketInput,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let http_path = args.next().ok_or("missing HTTP snapshot path")?;
    let websocket_path = args.next().ok_or("missing WebSocket snapshot path")?;
    if args.next().is_some() {
        return Err("unexpected thermal input validator argument".into());
    }
    let http: Emc2101ThermalSnapshotInput = serde_json::from_reader(File::open(http_path)?)?;
    let websocket: Emc2101ThermalWebSocketInput =
        serde_json::from_reader(File::open(websocket_path)?)?;
    validate_emc2101_thermal_inputs(&http, &websocket).map_err(Into::into)
}
