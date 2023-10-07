use chrono::{DateTime, Datelike, Local, Timelike, Utc};

mod console_transport;
mod file_transport;
mod filter_transport;

pub use console_transport::*;
pub use file_transport::*;
pub use filter_transport::*;

fn format_timestamp(timestamp: DateTime<Utc>) -> String {
    let local = timestamp.with_timezone(&Local);

    let year = local.year();
    let month = local.month();
    let day = local.day();

    let hour = local.hour();
    let minute = local.minute();
    let second = local.second();
    let millisecond = local.timestamp_subsec_millis();

    let zone_offset = local.offset().local_minus_utc();
    let offset_in_hours = zone_offset.abs() / 3600;
    let offset_in_minutes = (zone_offset.abs() % 3600) / 60;

    // ISO 8601
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}{}{:02}:{:02}",
        year,
        month,
        day,
        hour,
        minute,
        second,
        millisecond,
        if zone_offset < 0 { "-" } else { "+" },
        offset_in_hours,
        offset_in_minutes
    )
}
