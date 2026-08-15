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

confirm-api-command-identify *args:
    bazel run //tools/flash:flash -- signal-identify {{ args }}

signal-api-command-identify *args:
    bazel run //tools/flash:flash -- signal-identify {{ args }}

api-command-effects-campaign *args:
    bazel build //firmware/bitaxe:firmware_image
    bazel run //tools/automation:bitaxe_automation -- api-command-effects-campaign {{ args }}

# Runs only the bounded machine portion. The rendered/cleared confirmations are
# durable private files and therefore have no chat-response deadline.
api-command-display-uat *args:
    bazel run //tools/device-session:device-session -- display-uat-live {{ args }}

finalize-api-command-display-uat *args:
    bazel run //tools/device-session:device-session -- display-uat-finalize {{ args }}

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

validate-mining-criteria-evidence projection:
    #!/usr/bin/env bash
    set -euo pipefail
    projection={{ quote(projection) }}
    test -f "$projection"
    projection_path="$(/bin/realpath "$projection")"
    bazel run //crates/bitaxe-automation-contracts:validate_mining_criteria_evidence -- "$projection_path"

validate-ina260-evidence projection:
    #!/usr/bin/env bash
    set -euo pipefail
    projection={{ quote(projection) }}
    test -f "$projection"
    projection_path="$(/bin/realpath "$projection")"
    bazel run //crates/bitaxe-automation-contracts:validate_ina260_evidence -- "$projection_path"

validate-emc2101-thermal-evidence projection:
    #!/usr/bin/env bash
    set -euo pipefail
    projection={{ quote(projection) }}
    test -f "$projection"
    projection_path="$(/bin/realpath "$projection")"
    bazel run //crates/bitaxe-automation-contracts:validate_emc2101_thermal_evidence -- "$projection_path"

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

capture-adc-observation-evidence *args:
    bazel run //tools/automation:capture_adc_observation_evidence -- {{ args }}

validate-adc-observation-evidence projection:
    #!/usr/bin/env bash
    set -euo pipefail
    projection={{ quote(projection) }}
    test -f "$projection"
    projection_path="$(/bin/realpath "$projection")"
    bazel run //crates/bitaxe-automation-contracts:validate_adc_observation_evidence -- "$projection_path"

capture-emc2101-thermal-evidence *args:
    bazel run //tools/automation:capture_emc2101_thermal_evidence -- {{ args }}

capture-emc2101-thermal-fault-evidence *args:
    bazel run //tools/automation:capture_emc2101_thermal_fault_evidence -- {{ args }}

validate-emc2101-thermal-fault-evidence projection:
    #!/usr/bin/env bash
    set -euo pipefail
    projection={{ quote(projection) }}
    test -f "$projection"
    projection_path="$(/bin/realpath "$projection")"
    bazel run //crates/bitaxe-automation-contracts:validate_emc2101_thermal_fault_evidence -- "$projection_path"

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

capture-network-reconnect-evidence *args:
    bazel run //tools/automation:capture_network_reconnect_evidence -- {{ args }}

capture-network-scan-evidence *args:
    bazel run //tools/automation:capture_network_scan_evidence -- {{ args }}

project-asic-initialization-evidence *args:
    bazel run //tools/automation:project_asic_initialization_evidence -- {{ args }}

project-asic-power-initialization-evidence *args:
    bazel run //tools/automation:project_asic_power_initialization_evidence -- {{ args }}

project-core-voltage-control-evidence *args:
    bazel run //tools/automation:project_core_voltage_control_evidence -- {{ args }}

project-ina260-evidence *args:
    bazel run //tools/automation:project_ina260_evidence -- {{ args }}

project-asic-reset-evidence *args:
    bazel run //tools/automation:project_asic_reset_evidence -- {{ args }}

project-asic-work-send-evidence *args:
    bazel run //tools/automation:project_asic_work_send_evidence -- {{ args }}

project-asic-result-parsing-evidence *args:
    bazel run //tools/automation:project_asic_result_parsing_evidence -- {{ args }}

capture-provisioning-network-evidence *args:
    bazel run //tools/automation:capture_provisioning_network_evidence -- {{ args }}

project-ui-workflow-evidence *args:
    bazel run //tools/automation:project_ui_workflow_evidence -- {{ args }}

validate-ui-workflow-evidence projection:
    #!/usr/bin/env bash
    set -euo pipefail
    projection={{ quote(projection) }}
    test -f "$projection"
    projection_path="$(/bin/realpath "$projection")"
    bazel run //crates/bitaxe-automation-contracts:validate_ui_workflow_evidence -- "$projection_path"
