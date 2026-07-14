pub mod effects;
pub mod evidence;
pub mod fault;
pub mod mining_preconditions;
pub mod observation;
pub mod power;
pub mod self_test;
pub mod sensor_acquisition;
pub mod status;
pub mod thermal;
pub mod watchdog;

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

#[cfg(test)]
mod safety_module_graph {
    use super::{fault, power, self_test, thermal, watchdog};

    const BOUNDARY_SOURCES: [(&str, &str); 5] = [
        ("power", include_str!("power.rs")),
        ("thermal", include_str!("thermal.rs")),
        ("fault", include_str!("fault.rs")),
        ("self_test", include_str!("self_test.rs")),
        ("watchdog", include_str!("watchdog.rs")),
    ];

    #[test]
    fn safety_module_graph_public_modules_are_importable() {
        // Arrange
        let module_names = [
            power::MODULE_NAME,
            thermal::MODULE_NAME,
            fault::MODULE_NAME,
            self_test::MODULE_NAME,
            watchdog::MODULE_NAME,
        ];

        // Act
        let import_count = module_names.len();

        // Assert
        assert_eq!(import_count, 5);
    }

    #[test]
    fn safety_module_graph_boundaries_are_breadcrumbed() {
        // Arrange
        let breadcrumb_cases: [(&str, &[&str]); 5] = [
            ("power", &["DS4432U", "INA260"]),
            ("thermal", &["thermal.c", "PID.c", "fan_controller_task"]),
            ("fault", &["power_management_task", "fan_controller_task"]),
            ("self_test", &["self_test.c", "power_management_task"]),
            ("watchdog", &["power_management_task", "self_test.c"]),
        ];

        // Act / Assert
        for (module_name, expected_breadcrumbs) in breadcrumb_cases {
            let source = boundary_source(module_name);
            for &expected_breadcrumb in expected_breadcrumbs {
                assert!(
                    source.contains(expected_breadcrumb),
                    "{module_name} boundary should mention {expected_breadcrumb}"
                );
            }
        }
    }

    #[test]
    fn safety_module_graph_boundaries_remain_pure() {
        // Arrange
        let forbidden_terms = forbidden_boundary_terms();

        // Act / Assert
        for (module_name, source) in BOUNDARY_SOURCES {
            for forbidden_term in &forbidden_terms {
                assert!(
                    !source.contains(forbidden_term),
                    "{module_name} boundary should not contain {forbidden_term}"
                );
            }
        }
    }

    fn boundary_source(module_name: &str) -> &'static str {
        BOUNDARY_SOURCES
            .iter()
            .find_map(|(name, source)| (*name == module_name).then_some(*source))
            .expect("boundary source should exist")
    }

    fn forbidden_boundary_terms() -> Vec<String> {
        vec![
            ["esp", "_", "idf"].concat(),
            ["I2c", "Driver"].concat(),
            ["Pin", "Driver"].concat(),
            ["Uart", "Driver"].concat(),
            ["std::", "thread", "::", "sleep"].concat(),
            ["Tcp", "Stream"].concat(),
            ["Udp", "Socket"].concat(),
        ]
    }
}
