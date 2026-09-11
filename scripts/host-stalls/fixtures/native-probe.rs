use std::io::Write;

fn main() -> std::io::Result<()> {
    writeln!(std::io::stderr(), "host_stall_probe_entered_main")?;
    Ok(())
}
