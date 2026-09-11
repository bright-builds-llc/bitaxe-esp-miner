use std::{
    env, fs,
    path::PathBuf,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-env-changed=HOST_STALL_TRIAL");
    println!("cargo:rerun-if-env-changed=HOST_STALL_MARKER_ROOT");
    let nonce = env::var("HOST_STALL_TRIAL")?;
    let package = env::var("CARGO_PKG_NAME")?;
    let root = PathBuf::from(env::var("HOST_STALL_MARKER_ROOT")?);
    fs::write(
        root.join(format!("{nonce}-{package}.entered")),
        timestamp()?,
    )?;
    thread::sleep(Duration::from_millis(500));
    fs::write(root.join(format!("{nonce}-{package}.exited")), timestamp()?)?;
    Ok(())
}

fn timestamp() -> Result<String, std::time::SystemTimeError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .to_string())
}
