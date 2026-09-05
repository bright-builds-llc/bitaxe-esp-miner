use bitaxe_worker_control::{
    load_or_generate_device_identity, DeviceIdentitySeedGenerator, DeviceIdentitySeedStore,
    IdentityLoadError, PossessionRequest,
};

#[derive(Default)]
struct MemorySeedStore {
    maybe_seed: Option<Vec<u8>>,
    writes: usize,
}

impl DeviceIdentitySeedStore for MemorySeedStore {
    fn load_seed(&self) -> Result<Option<Vec<u8>>, IdentityLoadError> {
        Ok(self.maybe_seed.clone())
    }

    fn store_seed_atomic(&mut self, seed: &[u8; 32]) -> Result<(), IdentityLoadError> {
        self.maybe_seed = Some(seed.to_vec());
        self.writes += 1;
        Ok(())
    }
}

struct FixedGenerator([u8; 32]);

impl DeviceIdentitySeedGenerator for FixedGenerator {
    fn fill_seed(&mut self, seed: &mut [u8; 32]) -> Result<(), IdentityLoadError> {
        *seed = self.0;
        Ok(())
    }
}

#[test]
fn first_load_persists_once_and_later_loads_preserve_identity() {
    // Arrange
    let mut store = MemorySeedStore::default();
    let mut first_generator = FixedGenerator([7_u8; 32]);

    // Act
    let first = load_or_generate_device_identity(&mut store, &mut first_generator)
        .expect("absent identity should generate");
    let mut unused_generator = FixedGenerator([9_u8; 32]);
    let loaded = load_or_generate_device_identity(&mut store, &mut unused_generator)
        .expect("persisted identity should load");

    // Assert
    assert_eq!(store.writes, 1);
    assert_eq!(proof(&first), proof(&loaded));
}

#[test]
fn corrupt_seed_fails_closed_without_silent_rotation() {
    // Arrange
    let mut store = MemorySeedStore {
        maybe_seed: Some(vec![7_u8; 31]),
        writes: 0,
    };
    let mut generator = FixedGenerator([9_u8; 32]);

    // Act
    let result = load_or_generate_device_identity(&mut store, &mut generator);

    // Assert
    assert_eq!(
        result.expect_err("corrupt seed must fail").category(),
        "identity_corrupt"
    );
    assert_eq!(store.writes, 0);
}

fn proof(identity: &bitaxe_worker_control::DeviceIdentity) -> String {
    let request = PossessionRequest::from_frame(
        concat!(
            "{\"profile\":\"bwg-worker-possession/0.2\",",
            "\"requestId\":\"pos_identity_01\",",
            "\"command\":\"prove_possession\",",
            "\"payload\":{",
            "\"purpose\":\"initial_admission\",",
            "\"possessionNonce\":\"BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB\",",
            "\"challengeBindingSha256\":\"CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC\",",
            "\"controllerCapabilitySha256\":\"JFWsyueHvXS9M9GlDlK6yEOwUzO8oPXtloalyTRxFvE\",",
            "\"sessionId\":\"AAAAAAAAAAAAAAAAAAAAAA\",",
            "\"hostNonce\":\"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\",",
            "\"deviceNonce\":\"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\",",
            "\"serialManifestSha256\":\"rOKO_7whZfy0ntMKM9RIeZNAA3x97tt3rWMAm_QshVA\"",
            "}}\n"
        )
        .as_bytes(),
    )
    .expect("identity test request should parse");
    identity
        .prove(
            &request,
            &bitaxe_worker_control::FirmwareSourceCommit::parse(&"a".repeat(40))
                .expect("fixture source commit should parse"),
            &"b".repeat(64),
        )
        .expect("identity should sign")
        .compact_jws()
        .to_owned()
}
