doctor:
    ./scripts/esp-doctor.sh

bootstrap-esp *args:
    ./scripts/bootstrap-esp.sh {{ args }}

detect-ultra205:
    ./scripts/detect-ultra205.sh

diagnose-ultra205-session *args:
    ./scripts/diagnose-ultra205-session.sh {{ args }}

diagnose-ultra205-late-attach *args:
    ./scripts/diagnose-ultra205-late-attach.sh {{ args }}

diagnose-ultra205-uart-capture *args:
    ./scripts/diagnose-ultra205-uart-capture.sh {{ args }}

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

verify-reference:
    bazel run //scripts:verify_reference_clean

parity:
    bazel run //tools/parity:report -- report --checklist docs/parity/checklist.md --fail-on-invalid-verified

phase23-evidence *args:
    bazel run //scripts:phase23_redacted_operator_evidence -- {{ args }}

phase25-evidence *args:
    bazel run //scripts:phase25_live_stratum_evidence -- {{ args }}

phase27-evidence *args:
    bazel run //scripts:phase27_live_hardware_bridge_evidence -- {{ args }}

phase27-package:
    scripts/phase27-live-hardware-bridge-package.sh
