pub mod effects;
pub mod evidence;
pub mod status;

#[must_use]
pub const fn phase06_contract_name() -> &'static str {
    "safety-controllers-and-self-test"
}

#[cfg(test)]
mod tests {
    use super::{effects, evidence, phase06_contract_name, status};

    #[test]
    fn safety_contract_name_matches_phase_slug() {
        // Arrange

        // Act
        let contract_name = phase06_contract_name();

        // Assert
        assert_eq!(contract_name, "safety-controllers-and-self-test");
    }

    #[test]
    fn safety_contract_modules_are_publicly_importable() {
        // Arrange
        let module_names = [
            effects::MODULE_NAME,
            evidence::MODULE_NAME,
            status::MODULE_NAME,
        ];

        // Act
        let import_count = module_names.len();

        // Assert
        assert_eq!(import_count, 3);
    }
}
