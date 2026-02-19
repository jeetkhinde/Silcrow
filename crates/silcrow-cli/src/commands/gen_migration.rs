use crate::util::{find_project_root, write_if_new};
use std::time::SystemTime;

pub fn run(name: &str) -> Result<(), String> {
    let root = find_project_root()?;

    // Generate timestamp: YYYYMMDDHHMMSS
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("System time error: {e}"))?;
    let secs = now.as_secs();

    // Convert to date components (simplified UTC calculation)
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Days since epoch to year/month/day
    let (year, month, day) = days_to_ymd(days);

    let timestamp = format!(
        "{:04}{:02}{:02}{:02}{:02}{:02}",
        year, month, day, hours, minutes, seconds
    );

    let filename = format!("{}_{}.sql", timestamp, name);
    let path = root.join("migrations").join(&filename);

    println!("Generating migration: {name}");

    write_if_new(
        &path,
        "-- Add migration script here\n",
    )?;

    println!("\nMigration created.");
    println!("  file: migrations/{filename}");
    Ok(())
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    // Simplified Gregorian calendar calculation
    let mut year = 1970;

    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let months_days: [u64; 12] = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for &md in &months_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }

    (year, month, days + 1)
}

fn is_leap(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
