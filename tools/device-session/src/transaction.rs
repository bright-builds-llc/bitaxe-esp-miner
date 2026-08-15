use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    run_admitted_live_session, run_admitted_ota_session, OtaIntent, RebootIntent, SessionArtifacts,
    TerminalCategory,
};

pub const TRANSACTION_INTENT_SCHEMA: &str = "bitaxe-device-transaction-intent-v1";

/// One high-level verification goal admitted by the shared device transaction.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(
    tag = "transaction_kind",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum TransactionGoal {
    CommandEffects { reboot: RebootIntent },
    SettingsDurability { reboot: RebootIntent },
    Restart { reboot: RebootIntent },
    OtaTransition { ota: OtaIntent },
}

/// Private, versioned intent for one admitted live device transaction.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceTransactionIntent {
    pub schema_version: String,
    pub goal: TransactionGoal,
}

impl DeviceTransactionIntent {
    #[must_use]
    pub fn restart(reboot: RebootIntent) -> Self {
        Self {
            schema_version: TRANSACTION_INTENT_SCHEMA.to_owned(),
            goal: TransactionGoal::Restart { reboot },
        }
    }

    #[must_use]
    pub fn ota_transition(ota: OtaIntent) -> Self {
        Self {
            schema_version: TRANSACTION_INTENT_SCHEMA.to_owned(),
            goal: TransactionGoal::OtaTransition { ota },
        }
    }

    #[must_use]
    pub fn schema_is_valid(&self) -> bool {
        if self.schema_version != TRANSACTION_INTENT_SCHEMA {
            return false;
        }
        match &self.goal {
            TransactionGoal::CommandEffects { reboot }
            | TransactionGoal::SettingsDurability { reboot }
            | TransactionGoal::Restart { reboot } => reboot.schema_is_valid(),
            TransactionGoal::OtaTransition { ota } => ota.schema_is_valid(),
        }
    }

    #[must_use]
    pub const fn requires_ota_image(&self) -> bool {
        matches!(self.goal, TransactionGoal::OtaTransition { .. })
    }
}

/// Runs one admitted high-level goal through the authoritative session implementation.
pub fn run_admitted_transaction(
    intent: DeviceTransactionIntent,
    admitted_port: String,
    maybe_ota_image: Option<Vec<u8>>,
    artifacts: SessionArtifacts,
    timeout: Duration,
) -> Result<TerminalCategory> {
    if !intent.schema_is_valid() {
        anyhow::bail!("device transaction intent schema is invalid");
    }
    match intent.goal {
        TransactionGoal::CommandEffects { reboot }
        | TransactionGoal::SettingsDurability { reboot }
        | TransactionGoal::Restart { reboot } => {
            if maybe_ota_image.is_some() {
                anyhow::bail!("reboot transaction must not carry an OTA image");
            }
            run_admitted_live_session(reboot, admitted_port, artifacts, timeout)
        }
        TransactionGoal::OtaTransition { ota } => {
            let Some(ota_image) = maybe_ota_image else {
                anyhow::bail!("OTA transaction requires an image");
            };
            run_admitted_ota_session(ota, admitted_port, ota_image, artifacts, timeout)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BaselineApplication, ExpectedPostcondition, REBOOT_INTENT_SCHEMA};

    fn digest(character: char) -> String {
        std::iter::repeat_n(character, 64).collect()
    }

    fn reboot() -> RebootIntent {
        RebootIntent {
            schema_version: REBOOT_INTENT_SCHEMA.to_owned(),
            board_category: "205".to_owned(),
            trusted_origin: "http://private-device".to_owned(),
            baseline: BaselineApplication {
                boot_session: "private-session".to_owned(),
                boot_ordinal: 7,
                source_commit: "source".to_owned(),
                reference_commit: "reference".to_owned(),
                app_elf_sha256: digest('a'),
                running_partition: None,
            },
            expected_postcondition: ExpectedPostcondition {
                hostname_sha256: digest('b'),
                app_elf_sha256: None,
                running_partition: None,
            },
        }
    }

    #[test]
    fn every_reboot_goal_uses_the_same_validated_nested_intent() {
        // Arrange
        let goals = [
            TransactionGoal::CommandEffects { reboot: reboot() },
            TransactionGoal::SettingsDurability { reboot: reboot() },
            TransactionGoal::Restart { reboot: reboot() },
        ];

        // Act and assert
        for goal in goals {
            let intent = DeviceTransactionIntent {
                schema_version: TRANSACTION_INTENT_SCHEMA.to_owned(),
                goal,
            };
            assert!(intent.schema_is_valid());
            assert!(!intent.requires_ota_image());
        }
    }

    #[test]
    fn unknown_fields_and_unversioned_intents_fail_parsing_or_validation() {
        // Arrange
        let value = serde_json::json!({
            "schema_version": "wrong",
            "goal": {
                "transaction_kind": "restart",
                "reboot": reboot(),
            },
        });

        // Act
        let intent: DeviceTransactionIntent =
            serde_json::from_value(value).expect("shape is parseable");

        // Assert
        assert!(!intent.schema_is_valid());
    }
}
