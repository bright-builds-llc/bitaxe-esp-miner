doctor:
    bazel run //tools/automation:doctor

bootstrap-esp *args:
    bazel run //tools/automation:bootstrap_esp -- {{ args }}

detect-ultra205 *args:
    bazel run //tools/flash:flash -- detect {{ args }}

observe-serial *args:
    bazel run //tools/automation:observe_serial -- {{ args }}

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

mining-campaign *args:
    bazel run //tools/flash:flash -- mining-campaign {{ args }}

verify-flash-durability *args:
    bazel run //tools/automation:verify_flash_durability -- {{ args }}

verify-reference:
    bazel run //tools/automation:verify_reference

parity:
    bazel run //tools/parity:report -- report --checklist docs/parity/checklist.md --fail-on-invalid-verified

parity-progress *args:
    bazel run //tools/parity:report -- progress {{ args }}

verify-redaction *args:
    bazel run //tools/automation:verify_redaction -- {{ args }}

verify-production-session:
    bazel run //tools/automation:verify_production_session

capture-operator-evidence *args:
    bazel run //tools/automation:capture_operator_evidence -- {{ args }}

verify-settings-durability *args:
    bazel run //tools/automation:verify_settings_durability -- {{ args }}

verify-theme-durability *args:
    bazel run //tools/automation:verify_theme_durability -- {{ args }}

capture-correlated-runtime-evidence *args:
    bazel build //firmware/bitaxe:firmware_image
    bazel run //tools/automation:capture_correlated_runtime_evidence -- {{ args }}

capture-version-evidence *args:
    bazel run //tools/automation:capture_version_evidence -- {{ args }}

capture-operator-snapshot-evidence *args:
    bazel run //tools/automation:capture_operator_snapshot_evidence -- {{ args }}

capture-runtime-health-evidence *args:
    bazel run //tools/automation:capture_runtime_health_evidence -- {{ args }}

capture-system-info-evidence *args:
    bazel run //tools/automation:capture_system_info_evidence -- {{ args }}

capture-ultra205-defaults-evidence *args:
    bazel run //tools/automation:capture_ultra205_defaults_evidence -- {{ args }}

capture-settings-patch-evidence *args:
    bazel run //tools/automation:capture_settings_patch_evidence -- {{ args }}

capture-log-buffer-evidence *args:
    bazel run //tools/automation:capture_log_buffer_evidence -- {{ args }}

capture-partition-layout-evidence *args:
    bazel run //tools/automation:capture_partition_layout_evidence -- {{ args }}

capture-sdkconfig-rollback-evidence *args:
    bazel run //tools/automation:capture_sdkconfig_rollback_evidence -- {{ args }}
