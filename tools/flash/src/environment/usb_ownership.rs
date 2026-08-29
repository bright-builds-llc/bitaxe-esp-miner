use super::*;

const ESPFLASH_ADMITTED_ROM_BEFORE: &str = "no-reset";
const ESPTOOL_ADMITTED_ROM_BEFORE: &str = "no_reset";

impl LocalFlashEnvironment {
    pub(super) fn ensure_bootloader(&self) -> Result<()> {
        let mut session_slot = self.usb_session.borrow_mut();
        let Some(session) = session_slot.as_mut() else {
            bail!("cleanup_failed: bootloader admission attempted without a repository session");
        };
        let inspection = inspect_usb_profile(session.port())?;
        let intent = if session.operation() == UsbOperation::Recover {
            UsbIntent::Recover
        } else {
            UsbIntent::Flash
        };
        let transitioned = match plan_usb_operation(intent, inspection.profile) {
            UsbOperationPlan::HandoffThenEspflash => {
                handoff_worker_to_rom(session).map_err(|error| anyhow::anyhow!(error))?;
                true
            }
            UsbOperationPlan::DirectEspflash => false,
            UsbOperationPlan::RejectUnknownProfile
            | UsbOperationPlan::InspectOnly
            | UsbOperationPlan::ObserveOnly => bail!("runtime_profile_unknown"),
        };
        let before = if transitioned {
            ESPFLASH_ADMITTED_ROM_BEFORE
        } else {
            "usb-reset"
        };
        let args = [
            "board-info".to_owned(),
            "--chip".to_owned(),
            "esp32s3".to_owned(),
            "--port".to_owned(),
            session.port().to_owned(),
            "--non-interactive".to_owned(),
            "--before".to_owned(),
            before.to_owned(),
            "--after".to_owned(),
            "no-reset".to_owned(),
        ];
        let output = session
            .run_espflash_probe(
                self.espflash_bin.as_std_path(),
                &args,
                Duration::from_secs(30),
            )
            .map_err(|error| anyhow::anyhow!(error))?;
        let mut board_info = output.stdout;
        board_info.extend_from_slice(&output.stderr);
        let post_handoff_inspection = inspect_usb_profile(session.port())?;
        let rom = admit_rom_downloader(post_handoff_inspection, &board_info)
            .map_err(|error| anyhow::anyhow!(error))?;
        if rom.physical_identity_digest != inspection.physical_identity_digest {
            bail!("physical_identity_drift");
        }
        emit_line("usb_profile", "rom_downloader")?;
        Ok(())
    }

    pub(super) fn ensure_observable_runtime(&self, port: &str) -> Result<()> {
        let profile = inspect_usb_profile(port)?.profile;
        match plan_usb_operation(UsbIntent::Observe, profile) {
            UsbOperationPlan::ObserveOnly => Ok(()),
            UsbOperationPlan::RejectUnknownProfile
            | UsbOperationPlan::InspectOnly
            | UsbOperationPlan::DirectEspflash
            | UsbOperationPlan::HandoffThenEspflash => bail!("runtime_profile_unknown"),
        }
    }
}

pub(super) fn route_espflash_to_admitted_rom(
    command_spec: &CommandSpec,
    port: &str,
) -> Result<Vec<String>> {
    let mut args = command_with_port(command_spec, port)?;
    replace_option_value(&mut args, "--before", ESPFLASH_ADMITTED_ROM_BEFORE)?;
    Ok(args)
}

pub(super) fn route_esptool_to_admitted_rom(args: &[String], port: &str) -> Result<Vec<String>> {
    let mut args = args.to_vec();
    replace_option_value(&mut args, "--port", port)?;
    replace_option_value(&mut args, "--before", ESPTOOL_ADMITTED_ROM_BEFORE)?;
    Ok(args)
}

fn replace_option_value(args: &mut [String], option: &str, value: &str) -> Result<()> {
    let Some(index) = args.iter().position(|argument| argument == option) else {
        bail!("USB-owned command is missing {option}");
    };
    let Some(argument) = args.get_mut(index.saturating_add(1)) else {
        bail!("USB-owned command has an incomplete {option} argument");
    };
    *argument = value.to_owned();
    Ok(())
}

pub(super) fn requires_rom_downloader(command_spec: &CommandSpec) -> bool {
    matches!(
        command_spec.args.first().map(String::as_str),
        Some("write-bin" | "erase-flash" | "flash")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admitted_espflash_route_replaces_port_and_preserves_rom_without_reset() {
        // Arrange
        let command = CommandSpec::new(
            "espflash",
            ["write-bin", "--port", "worker", "--before", "usb-reset"],
        );

        // Act
        let args = route_espflash_to_admitted_rom(&command, "rom").expect("admitted route");

        // Assert
        assert_eq!(args, ["write-bin", "--port", "rom", "--before", "no-reset"]);
    }

    #[test]
    fn admitted_esptool_route_uses_its_native_no_reset_spelling() {
        // Arrange
        let args = ["--port", "worker", "--before", "usb_reset"].map(str::to_owned);

        // Act
        let routed = route_esptool_to_admitted_rom(&args, "rom").expect("admitted route");

        // Assert
        assert_eq!(routed, ["--port", "rom", "--before", "no_reset"]);
    }

    #[test]
    fn only_write_commands_require_rom_admission() {
        // Arrange
        let write = CommandSpec::new("espflash", ["write-bin"]);
        let board_info = CommandSpec::new("espflash", ["board-info"]);
        let monitor = CommandSpec::new("bitaxe-receive-only", ["observe"]);

        // Act / Assert
        assert!(requires_rom_downloader(&write));
        assert!(!requires_rom_downloader(&board_info));
        assert!(!requires_rom_downloader(&monitor));
    }
}
