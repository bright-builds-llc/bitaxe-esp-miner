use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bitaxe_worker_control::{serial::serial_manifest_sha256, WorkLeaseAuthorityTrust};
use ed25519_dalek::{Signature, VerifyingKey};
use serde_json::Value;

const TRUST: &str = include_str!("../../../firmware/bitaxe/bwg/deployment-trust.json");
const CAPABILITY: &str = include_str!("../../../firmware/bitaxe/bwg/ultra205-capability.json");

#[test]
fn compiled_runtime_trust_capability_and_manifest_agree() {
    // Arrange: exercise the exact public files compiled into firmware, not conformance keys.
    let trust: Value = serde_json::from_str(TRUST).expect("compiled deployment trust JSON");
    let capability: Value = serde_json::from_str(CAPABILITY).expect("compiled capability JSON");
    let claims = &capability["attestation"]["claims"];

    // Act
    WorkLeaseAuthorityTrust::from_deployment_json(TRUST)
        .expect("compiled lease authority must admit Controller 0.4");
    let compact = capability["attestation"]["compactJws"]
        .as_str()
        .expect("signed capability");
    let parts: Vec<_> = compact.split('.').collect();
    assert_eq!(parts.len(), 3);
    let header: Value = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(parts[0])
            .expect("protected header encoding"),
    )
    .expect("protected header JSON");
    let signed: Value = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(parts[1])
            .expect("signed claims encoding"),
    )
    .expect("signed claims JSON");
    let key = trust["updateAuthority"]["keys"]
        .as_array()
        .expect("update role keys")
        .iter()
        .find(|key| key["kid"] == header["kid"])
        .expect("capability must use an installed Update Authority");
    let key_bytes: [u8; 32] = URL_SAFE_NO_PAD
        .decode(key["x"].as_str().expect("public key"))
        .expect("public key encoding")
        .try_into()
        .expect("Ed25519 key length");
    let key = VerifyingKey::from_bytes(&key_bytes).expect("Update Authority key");
    let signature = Signature::from_slice(
        &URL_SAFE_NO_PAD
            .decode(parts[2])
            .expect("signature encoding"),
    )
    .expect("Ed25519 signature length");
    key.verify_strict(format!("{}.{}", parts[0], parts[1]).as_bytes(), &signature)
        .expect("compiled capability signature");

    // Assert: signed and served claims describe the exact fixed transport.
    assert_eq!(header["alg"], "Ed25519");
    assert_eq!(header["typ"], "bwg-worker-capability+jws");
    assert_eq!(&signed, claims);
    assert_eq!(claims["profile"], "bwg-reference-firmware-capability/0.2");
    assert_eq!(claims["profile"], trust["updateAuthority"]["audience"]);
    assert_eq!(trust["updateAuthority"]["role"], "update_authority");
    assert_eq!(capability["protocolVersion"], "bwg-worker-controller/0.4");
    assert_eq!(capability["transportProfile"], "bwg-worker-serial/0.1");
    assert_eq!(
        claims["serialManifestSha256"],
        serial_manifest_sha256().expect("serial manifest digest")
    );
    for field in [
        "protocolVersion",
        "firmware",
        "compatibility",
        "transportProfile",
    ] {
        assert_eq!(
            claims[field], capability[field],
            "signed capability field {field}"
        );
    }
    assert_eq!(claims["board"]["model"], capability["board"]["model"]);
    assert_eq!(claims["board"]["revision"], capability["board"]["revision"]);
    assert_eq!(capability["board"]["usbTransport"], "web_serial");
    assert!(claims.get("applicationDescriptorSha256").is_none());
}
