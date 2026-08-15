/// Highest boot-scoped generations observed through one independent log channel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct CommandTransitionWitness {
    pub(super) pause_generation: u64,
    pub(super) resume_generation: u64,
    pub(super) identify_generation: u64,
    pub(super) dismiss_generation: u64,
    pub(super) display_identify_generation: u64,
    pub(super) display_non_identify_generation: u64,
}

impl CommandTransitionWitness {
    pub(super) fn observe(&mut self, command: &str, generation: u64, outcome: &str) -> bool {
        let target = match (command, outcome) {
            ("pause", "applied") => &mut self.pause_generation,
            ("resume", "applied") => &mut self.resume_generation,
            ("identify_enable", "applied") => &mut self.identify_generation,
            ("block_found_dismiss", "applied") => &mut self.dismiss_generation,
            ("display_identify", "rendered") => &mut self.display_identify_generation,
            ("display_non_identify", "rendered") => &mut self.display_non_identify_generation,
            ("restart", "applied")
            | ("identify_disable", "applied")
            | ("display_availability", "available" | "unavailable") => return true,
            _ => return false,
        };
        *target = (*target).max(generation);
        true
    }
}
