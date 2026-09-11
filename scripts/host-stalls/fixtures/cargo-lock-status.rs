use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "entered_main {}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()
    );
    Ok(())
}
