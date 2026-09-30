#[path = "snapshot.rs"]
mod snapshot;
fn main() {
    let root =
        std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest"))
            .parent()
            .and_then(std::path::Path::parent)
            .expect("workspace root")
            .to_owned();
    for path in snapshot::ROOTS.iter().chain(snapshot::FILES.iter()) {
        println!("cargo:rerun-if-changed={}", root.join(path).display());
    }
    let inputs = snapshot::snapshot(&root).expect("compiler source snapshot");
    println!("cargo:rustc-env=BITAXE_VIRTUAL_COMPILED_INPUTS_JSON={inputs}");
}
