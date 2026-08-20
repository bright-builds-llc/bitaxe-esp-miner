use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
    process::Command,
    thread,
};

fn run_git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("git should launch");
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn commit_all(root: &Path) {
    run_git(root, &["add", "."]);
    run_git(
        root,
        &[
            "-c",
            "user.name=Pool Readiness Test",
            "-c",
            "user.email=pool-readiness@example.invalid",
            "commit",
            "-m",
            "fixture",
        ],
    );
}

fn initialize_workspace(root: &Path) {
    run_git(root, &["init", "--quiet"]);
    fs::write(root.join(".gitignore"), "scratch/\npool-credentials.json\n")
        .expect("ignore file should write");
    fs::write(root.join("MODULE.bazel"), "module(name = \"fixture\")\n")
        .expect("module file should write");
    let reference = root.join("reference/esp-miner");
    fs::create_dir_all(&reference).expect("reference directory should create");
    run_git(&reference, &["init", "--quiet"]);
    fs::write(reference.join("reference.txt"), "pinned\n").expect("reference fixture should write");
    commit_all(&reference);
    commit_all(root);
}

fn spawn_pool() -> (u16, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("loopback pool should bind");
    let port = listener
        .local_addr()
        .expect("loopback address should resolve")
        .port();
    let handle = thread::spawn(move || {
        for _ in 0..3 {
            let (mut stream, _) = listener.accept().expect("loopback pool should accept");
            let reader_stream = stream.try_clone().expect("loopback stream should clone");
            let mut reader = BufReader::new(reader_stream);
            for _ in 0..3 {
                let mut line = String::new();
                reader
                    .read_line(&mut line)
                    .expect("loopback request should read");
                let request: serde_json::Value =
                    serde_json::from_str(&line).expect("request should be JSON");
                let method = request["method"]
                    .as_str()
                    .expect("request method should be text");
                assert_ne!(method, "mining.submit");
                let response = match method {
                    "mining.configure" => {
                        "{\"id\":1,\"result\":{\"version-rolling\":true,\"version-rolling.mask\":\"1fffe000\"},\"error\":null}\n"
                    }
                    "mining.subscribe" => {
                        "{\"id\":2,\"result\":[[],\"01020304\",4],\"error\":null}\n"
                    }
                    "mining.authorize" => {
                        "{\"id\":3,\"result\":true,\"error\":null}\n"
                    }
                    _ => panic!("unexpected request method"),
                };
                stream
                    .write_all(response.as_bytes())
                    .expect("loopback response should write");
            }
        }
    });
    (port, handle)
}

#[test]
fn built_cli_proves_three_sessions_without_exposing_credentials() {
    // Arrange
    let temporary = tempfile::tempdir().expect("temporary workspace should create");
    let root = temporary.path();
    initialize_workspace(root);
    fs::create_dir_all(root.join("scratch/stat003-scoreboard"))
        .expect("ignored scratch parent should create");
    let credentials = root.join("pool-credentials.json");
    let (port, pool) = spawn_pool();
    fs::write(
        &credentials,
        format!(
            "{{\"poolURL\":\"127.0.0.1\",\"poolPort\":{port},\"poolUser\":\"private-owner.worker\",\"poolPassword\":\"private-password\"}}\n"
        ),
    )
    .expect("private credentials should write");
    fs::set_permissions(&credentials, fs::Permissions::from_mode(0o600))
        .expect("private credential mode should set");

    // Act
    let output = Command::new(env!("CARGO_BIN_EXE_pool-readiness"))
        .args([
            "--private-root",
            "scratch/stat003-scoreboard/readiness-005",
            "--pool-credentials",
            "pool-credentials.json",
            "--attempt-ordinal",
            "5",
            "--samples",
            "3",
            "--sample-timeout-seconds",
            "15",
            "--sample-delay-seconds",
            "2",
        ])
        .env("BUILD_WORKSPACE_DIRECTORY", root)
        .output()
        .expect("pool readiness CLI should launch");
    pool.join().expect("loopback pool should finish");

    // Assert
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let public = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(public.contains("pool_readiness=ready samples=3"));
    assert!(!public.contains("private-owner"));
    assert!(!public.contains("private-password"));
    assert!(!public.contains("127.0.0.1"));
    let private_root = root.join("scratch/stat003-scoreboard/readiness-005");
    assert_eq!(
        fs::metadata(&private_root)
            .expect("private root metadata should read")
            .mode()
            & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(private_root.join("readiness-result.json"))
            .expect("private result metadata should read")
            .mode()
            & 0o777,
        0o600
    );
}
