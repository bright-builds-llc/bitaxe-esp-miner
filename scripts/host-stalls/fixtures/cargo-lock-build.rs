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
    let root = PathBuf::from(env::var("HOST_STALL_MARKER_ROOT")?);
    fs::write(root.join(format!("{nonce}.entered")), timestamp()?)?;
    thread::sleep(Duration::from_secs(5));
    fs::write(root.join(format!("{nonce}.exited")), timestamp()?)?;
    Ok(())
}

fn timestamp() -> Result<String, std::time::SystemTimeError> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .to_string())
}
