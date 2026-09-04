use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlashReadDisposition {
    RetainRom,
    ReturnToApplication,
}

pub(crate) struct ManagedFlashRead {
    program: Utf8PathBuf,
    address: u32,
    size: u32,
    output: Utf8PathBuf,
    disposition: FlashReadDisposition,
}

impl ManagedFlashRead {
    pub(crate) fn new(
        program: Utf8PathBuf,
        address: u32,
        size: u32,
        output: Utf8PathBuf,
        disposition: FlashReadDisposition,
    ) -> Result<Self> {
        if size == 0 || size > 0x400000 {
            bail!("flash_transfer=blocked reason=range");
        }
        Ok(Self {
            program,
            address,
            size,
            output,
            disposition,
        })
    }

    pub(crate) fn program(&self) -> &Utf8Path {
        &self.program
    }
    pub(crate) const fn size(&self) -> u32 {
        self.size
    }
    pub(crate) fn output(&self) -> &Utf8Path {
        &self.output
    }

    pub(crate) fn args(&self, port: &str) -> Vec<String> {
        let after = match self.disposition {
            FlashReadDisposition::RetainRom => "no_reset",
            FlashReadDisposition::ReturnToApplication => "hard_reset",
        };
        [
            "--chip".to_owned(),
            "esp32s3".to_owned(),
            "--port".to_owned(),
            port.to_owned(),
            "--before".to_owned(),
            "no_reset".to_owned(),
            "--after".to_owned(),
            after.to_owned(),
            "--no-stub".to_owned(),
            "read_flash".to_owned(),
            format!("0x{:x}", self.address),
            format!("0x{:x}", self.size),
            self.output.as_str().to_owned(),
            "--flash_size".to_owned(),
            "16MB".to_owned(),
        ]
        .to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_read_uses_rom_only_adapter_and_explicit_geometry() {
        // Arrange
        let read = ManagedFlashRead::new(
            Utf8PathBuf::from("esptool"),
            0x9000,
            0x6000,
            Utf8PathBuf::from("private.bin"),
            FlashReadDisposition::ReturnToApplication,
        )
        .expect("read request");

        // Act
        let args = read.args("admitted");

        // Assert
        assert!(args
            .windows(2)
            .any(|pair| pair == ["--after", "hard_reset"]));
        assert!(args.iter().any(|arg| arg == "--no-stub"));
        assert!(args.windows(2).any(|pair| pair == ["--flash_size", "16MB"]));
        assert_eq!(
            args.iter()
                .filter(|arg| arg.as_str() == "read_flash")
                .count(),
            1
        );
    }
}
