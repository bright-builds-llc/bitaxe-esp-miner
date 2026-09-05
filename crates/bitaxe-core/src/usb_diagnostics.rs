//! Closed boot diagnostic allowlist for the single Serial/JTAG writer.

/// Selects only exact closed memory/startup fields, never arbitrary retained log text.
#[must_use]
pub fn is_worker_diagnostic_retained_line(line: &str) -> bool {
    let mut fields = line.split(' ');
    match fields.next() {
        Some("usb_memory_checkpoint") => {
            matches!(
                fields.next(),
                Some(
                    "stage=worker_owner_prepare"
                        | "stage=usb_install"
                        | "stage=usb_installed"
                        | "stage=statistics_start"
                        | "stage=statistics_started"
                )
            ) && fields
                .next()
                .is_some_and(|field| decimal_field(field, "free_bytes="))
                && fields
                    .next()
                    .is_some_and(|field| decimal_field(field, "largest_block_bytes="))
                && fields
                    .next()
                    .is_some_and(|field| decimal_field(field, "reserve_bytes="))
                && fields.next() == Some("redacted=true")
                && fields.next().is_none()
        }
        Some("bwg_worker_start_failure") => {
            fields.next() == Some("category=startup_failed")
                && matches!(
                    fields.next(),
                    Some("detail=owner_spawn" | "detail=usb_install" | "detail=control_owner")
                )
                && fields.next() == Some("redacted=true")
                && fields.next().is_none()
        }
        _ => false,
    }
}

fn decimal_field(field: &str, prefix: &str) -> bool {
    field.strip_prefix(prefix).is_some_and(|value| {
        !value.is_empty() && value.len() <= 10 && value.bytes().all(|byte| byte.is_ascii_digit())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extra_or_sensitive_fields_never_become_diagnostics() {
        let valid = "usb_memory_checkpoint stage=usb_install free_bytes=100 largest_block_bytes=90 reserve_bytes=98304 redacted=true";
        assert!(is_worker_diagnostic_retained_line(valid));
        assert!(!is_worker_diagnostic_retained_line(&format!(
            "{valid} secret=value"
        )));
        assert!(!is_worker_diagnostic_retained_line(
            "bwg_worker_start_failure category=startup_failed detail=private-url redacted=true"
        ));
    }
}
