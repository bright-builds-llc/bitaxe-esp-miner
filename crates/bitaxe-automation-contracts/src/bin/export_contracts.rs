use bitaxe_automation_contracts::{
    contract_bundle, input_uat_typescript_contracts, typescript_contracts,
    ui_workflow_typescript_contracts,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--typescript") {
        print!("{}", typescript_contracts());
        return Ok(());
    }
    if std::env::args().nth(1).as_deref() == Some("--ui-workflow-typescript") {
        print!("{}", ui_workflow_typescript_contracts());
        return Ok(());
    }
    if std::env::args().nth(1).as_deref() == Some("--input-uat-typescript") {
        print!("{}", input_uat_typescript_contracts());
        return Ok(());
    }
    serde_json::to_writer_pretty(std::io::stdout(), &contract_bundle())?;
    println!();
    Ok(())
}
