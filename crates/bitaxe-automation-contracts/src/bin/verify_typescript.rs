use std::env;
use std::fs;
use std::io::Write;

use bitaxe_automation_contracts::typescript_contracts;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1);
    let checked_copy = args.next().ok_or("missing checked TypeScript copy")?;
    let stamp = args.next().ok_or("missing verification stamp path")?;
    if args.next().is_some() {
        return Err("unexpected verification argument".into());
    }
    let checked = fs::read_to_string(checked_copy)?;
    if checked != typescript_contracts() {
        return Err("checked TypeScript automation contracts are stale".into());
    }
    let mut file = fs::File::create(stamp)?;
    file.write_all(b"bitaxe-command-contract-v1\n")?;
    Ok(())
}
