impl RomExitEnvironment for FakeFlashEnvironment {
    fn execute_rom_exit(
        &self,
        _esptool: &Utf8Path,
        _observation_seconds: u64,
    ) -> Result<RomExitHardwareCapture> {
        Ok(RomExitHardwareCapture {
            force_download_bit_set: true,
            transport: UsbProfile::SerialJtagRuntime,
            reenumerated: true,
            monitor: MonitorOutput {
                bytes: Vec::new(),
                interrupted_by: None,
                reenumerated: true,
            },
        })
    }
}

fn fake_usb_command_diagnostic(
    terminal_category: UsbTerminalCategory,
    device_effect_state: UsbDeviceEffectState,
) -> UsbCommandDiagnostic {
    UsbCommandDiagnostic {
        schema_version: "esp-usb-command-diagnostic-v1".to_owned(),
        terminal_category,
        device_effect_state,
        termination: UsbCommandTermination::ExitedSuccess,
        attempt_count: 1,
        connection_signature: UsbConnectionSignature::NotApplicable,
        stdout_bytes: 0,
        stderr_bytes: 0,
        stdout_sha256: sha256_bytes(&[]),
        stderr_sha256: sha256_bytes(&[]),
        transfer_started: device_effect_state != UsbDeviceEffectState::None,
        transfer_completed: device_effect_state == UsbDeviceEffectState::Completed,
        raw_output_included: false,
    }
}
