use bitaxe_automation_contracts::{contract_bundle, typescript_contracts};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--typescript") {
        print!("{}", typescript_contracts());
        return Ok(());
    }
    serde_json::to_writer_pretty(std::io::stdout(), &contract_bundle())?;
    println!();
    Ok(())
}
