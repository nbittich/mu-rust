use std::error::Error;

use chrono::Local;

use serde::{Deserialize, Serialize};
use time::{macros::format_description, UtcOffset};
use tracing::Level;
use tracing_subscriber::{fmt::time::OffsetTime, EnvFilter, FmtSubscriber};

pub const SPARQL_RESULT_CONTENT_TYPE: &str = "application/sparql-results+json";
pub const HEADER_MU_AUTH_SUDO: &str = "mu-auth-sudo";
pub const HEADER_MU_CALL_ID: &str = "mu-auth-sudo";
pub const HEADER_MU_SESSION_ID: &str = "mu-call-id";
pub const SPARQL_ENDPOINT: &str = "SPARQL_ENDPOINT";
// response
pub const HEADER_MU_AUTH_ALLOWED_GROUPS: &str = "mu-auth-allowed-groups";
pub const HEADER_MU_AUTH_USED_GROUPS: &str = "mu-auth-used-groups";

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SessionQueryHeaders {
    pub call_id: Option<String>,
    pub session_id: Option<String>,
}

pub fn setup_tracing() -> Result<(), Box<dyn Error>> {
    let offset_hours = {
        let now = Local::now();
        let offset_seconds = now.offset().local_minus_utc();
        let hours = offset_seconds / 3600;
        hours as i8
    };
    let offset = UtcOffset::from_hms(offset_hours, 0, 0)?;

    let timer = OffsetTime::new(
        offset,
        format_description!("[day]-[month]-[year] [hour]:[minute]:[second]"),
    );

    let subscriber = FmtSubscriber::builder()
        .with_timer(timer)
        .with_max_level(Level::TRACE)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
