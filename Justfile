doctor:
    ./scripts/esp-doctor.sh

bootstrap-esp *args:
    ./scripts/bootstrap-esp.sh {{ args }}

detect-ultra205 *args:
    bazel run //tools/flash:flash -- detect {{ args }}

diagnose-ultra205-session *args:
    ./scripts/diagnose-ultra205-session.sh {{ args }}

diagnose-ultra205-late-attach *args:
    bash scripts/phase28.1.1-terminal-closure-guard.sh

diagnose-ultra205-uart-capture *args:
    bash scripts/phase28.1.1-terminal-closure-guard.sh

build:
    bazel build //firmware/bitaxe:firmware

test:
    bazel test //...

package:
    bazel build //firmware/bitaxe:firmware_image

flash *args:
    bazel run //tools/flash:flash -- flash {{ args }}

monitor *args:
    bazel run //tools/flash:flash -- monitor {{ args }}

flash-monitor *args:
    bazel run //tools/flash:flash -- flash-monitor {{ args }}

verify-flash-durability *args:
    ./scripts/verify-flash-durability.sh {{ args }}

verify-reference:
    bazel run //scripts:verify_reference_clean

parity:
    bazel run //tools/parity:report -- report --checklist docs/parity/checklist.md --fail-on-invalid-verified

verify-redaction *args:
    bazel run //scripts:verify_redaction -- {{ args }}

verify-production-session:
    bazel test //crates/bitaxe-stratum:tests //crates/bitaxe-api:tests //crates/bitaxe-config:tests //scripts:verify_production_session_source_test
    bazel run //scripts:verify_production_session_source
    bazel build //firmware/bitaxe:firmware

phase23-evidence *args:
    bazel run //scripts:phase23_redacted_operator_evidence -- {{ args }}

phase33-settings-durability *args:
    ./scripts/phase33-confirmed-settings-durability.sh {{ args }}

phase35-evidence *args:
    bazel build //firmware/bitaxe:firmware_image
    bazel run //scripts:phase35_correlated_evidence -- {{ args }}

phase36-substantive-evidence *args:
    bazel run //scripts:phase36_substantive_evidence -- {{ args }}
