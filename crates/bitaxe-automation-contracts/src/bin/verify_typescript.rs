use std::env;
use std::fs;
use std::io::Write;

use bitaxe_automation_contracts::{
    input_uat_typescript_contracts, typescript_contracts, ui_workflow_typescript_contracts,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let checked_copy = args.next().ok_or("missing checked TypeScript copy")?;
    let checked_ui_workflow_copy = args
        .next()
        .ok_or("missing checked UI workflow TypeScript copy")?;
    let checked_input_uat_copy = args
        .next()
        .ok_or("missing checked input UAT TypeScript copy")?;
    let stamp = args.next().ok_or("missing verification stamp path")?;
    if args.next().is_some() {
        return Err("unexpected verification argument".into());
    }
    let checked = fs::read_to_string(checked_copy)?;
    if checked != typescript_contracts() {
        return Err("checked TypeScript automation contracts are stale".into());
    }
    let checked_ui_workflow = fs::read_to_string(checked_ui_workflow_copy)?;
    if checked_ui_workflow != ui_workflow_typescript_contracts() {
        return Err("checked UI workflow TypeScript automation contracts are stale".into());
    }
    let checked_input_uat = fs::read_to_string(checked_input_uat_copy)?;
    if checked_input_uat != input_uat_typescript_contracts() {
        return Err("checked input UAT TypeScript automation contracts are stale".into());
    }
    let mut file = fs::File::create(stamp)?;
    file.write_all(b"bitaxe-command-contract-v1\n")?;
    Ok(())
}
