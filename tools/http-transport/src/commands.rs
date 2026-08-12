use super::*;

impl StrictHttpClient {
    pub fn post_pause_once(&self, deadline: Instant) -> Result<ExchangeObservation> {
        self.exchange_until(
            "POST",
            "/api/system/pause",
            deadline,
            DEFAULT_CONNECT_TIMEOUT,
        )
    }

    pub fn post_resume_once(&self, deadline: Instant) -> Result<ExchangeObservation> {
        self.exchange_until(
            "POST",
            "/api/system/resume",
            deadline,
            DEFAULT_CONNECT_TIMEOUT,
        )
    }

    pub fn post_identify_once(&self, deadline: Instant) -> Result<ExchangeObservation> {
        self.exchange_until(
            "POST",
            "/api/system/identify",
            deadline,
            DEFAULT_CONNECT_TIMEOUT,
        )
    }

    pub fn post_block_found_dismiss_once(&self, deadline: Instant) -> Result<ExchangeObservation> {
        self.exchange_until(
            "POST",
            "/api/system/blockFound/dismiss",
            deadline,
            DEFAULT_CONNECT_TIMEOUT,
        )
    }
}
