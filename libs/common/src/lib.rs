use std::error::Error;

use chrono::Local;

use time::{macros::format_description, UtcOffset};
use tracing::Level;
use tracing_subscriber::{fmt::time::OffsetTime, EnvFilter, FmtSubscriber};

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
