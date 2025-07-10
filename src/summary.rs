use crate::{config::Config, entry::get_log_file_path_for_date};
use chrono::{Datelike, Duration, Local, Weekday};
use std::{fs, io::Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn parse_weekday(day_str: &str) -> Option<Weekday> {
    match day_str.to_lowercase().as_str() {
        "monday" | "mon" => Some(Weekday::Mon),
        "tuesday" | "tue" => Some(Weekday::Tue),
        "wednesday" | "wed" => Some(Weekday::Wed),
        "thursday" | "thu" => Some(Weekday::Thu),
        "friday" | "fri" => Some(Weekday::Fri),
        "saturday" | "sat" => Some(Weekday::Sat),
        "sunday" | "sun" => Some(Weekday::Sun),
        _ => None,
    }
}

pub fn summarize_logs(log_dir: &str, days: u32, config: &Config) -> anyhow::Result<()> {
    let today = Local::now().date_naive();
    let mut total_entries = 0;
    let mut entries_by_day = Vec::new();
    let mut total_eligible_days = 0;

    // Parse configured days into weekdays
    let allowed_weekdays: Vec<Weekday> = config
        .summary_days
        .iter()
        .filter_map(|day| parse_weekday(day))
        .collect();

    // Print header
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
    writeln!(stdout, "=== Log Summary for Past {} Days ===", days)?;
    stdout.reset()?;

    // Collect entries for each day
    for i in 0..days {
        let date = today - Duration::days(i as i64);
        let weekday = date.weekday();

        // Check if this day is in our allowed days
        if allowed_weekdays.contains(&weekday) {
            total_eligible_days += 1;
            let log_path = get_log_file_path_for_date(log_dir, date);

            if log_path.exists() {
                let content = fs::read_to_string(&log_path)?;
                if !content.trim().is_empty() {
                    total_entries += 1;
                    entries_by_day.push((date, content));
                }
            }
        }
    }

    if entries_by_day.is_empty() {
        println!(
            "No log entries found for the past {} days on configured days.",
            days
        );
        return Ok(());
    }

    // Print summary statistics
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
    writeln!(stdout, "\nSummary Statistics:")?;
    stdout.reset()?;
    println!("- Total days with entries: {}", total_entries);
    println!(
        "- Logging consistency: {:.1}% ({}/{} days)",
        (total_entries as f64 / total_eligible_days as f64) * 100.0,
        total_entries,
        total_eligible_days
    );

    // Show entries by day (most recent first)
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
    writeln!(stdout, "\nDaily Entries:")?;
    stdout.reset()?;

    for (date, content) in entries_by_day {
        // Print date header
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)).set_bold(true))?;
        writeln!(stdout, "\n--- {} ---", date.format("%Y-%m-%d (%A)"),)?;
        stdout.reset()?;

        // Extract and show titles/headers from the content
        let titles = extract_entry_titles(&content);
        if !titles.is_empty() {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
            for title in titles {
                println!("  - {}", title);
            }
            stdout.reset()?;
        } else {
            // If no clear titles, show first line or two
            let lines: Vec<&str> = content.lines().take(2).collect();
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
            for line in lines {
                if !line.trim().is_empty() {
                    println!("  {}", line.trim());
                    break;
                }
            }
            stdout.reset()?;
        }
    }

    // Print footer
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
    writeln!(stdout, "\n=== End of Summary ===")?;
    stdout.reset()?;

    Ok(())
}

fn extract_entry_titles(content: &str) -> Vec<String> {
    let mut titles = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        // Look for markdown headers (## timestamp - title format)
        if trimmed.starts_with("## ") && trimmed.contains(" - ") {
            if let Some(title_part) = trimmed.split(" - ").nth(1) {
                titles.push(title_part.to_string());
            }
        }
        // Also look for other markdown headers
        else if trimmed.starts_with("# ") || trimmed.starts_with("### ") {
            titles.push(trimmed.trim_start_matches('#').trim().to_string());
        }
    }

    titles
}