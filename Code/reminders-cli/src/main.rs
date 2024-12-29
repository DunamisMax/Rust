use std::{
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone}; // <-- We import `TimeZone` here
use clap::{Parser, Subcommand};
use dirs::home_dir;
use serde::{Deserialize, Serialize};

/// The name of our JSON file. Will be stored in the user's home directory.
const REMINDERS_FILE: &str = ".reminders.json";

/// Each reminder we store in our JSON file.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Reminder {
    id: usize,                    // A unique ID
    title: String,                // The reminder text or title
    due: Option<DateTime<Local>>, // Optional due date/time
    completed: bool,              // Whether the reminder is completed
}

/// The main CLI arguments parser.
#[derive(Debug, Parser)]
#[command(name = "reminders")]
#[command(about = "A simple CLI Reminders application in Rust!", long_about = None)]
struct Cli {
    /// The subcommand to run (e.g., `add`, `list`, `done`, `remove`)
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Add a new reminder (e.g. `reminders add "Buy milk" --due "2024-12-29 10:00"`)
    Add {
        /// Title of the reminder
        title: String,
        /// Optional due date/time (in RFC3339 or "YYYY-MM-DD HH:MM" format, e.g. "2024-12-29 10:00")
        #[arg(short, long)]
        due: Option<String>,
    },
    /// Mark a reminder as completed by its ID
    Done {
        /// ID of the reminder to mark done
        id: usize,
    },
    /// Remove a reminder by its ID
    Remove {
        /// ID of the reminder to remove
        id: usize,
    },
    /// List reminders (by default, lists only outstanding ones; use `--all` to show completed too)
    List {
        /// Show all reminders, including completed
        #[arg(short, long)]
        all: bool,
    },
    /// Clear all completed reminders
    ClearCompleted,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. Load existing reminders from file (or create an empty file if none exists).
    let mut reminders = load_reminders()?;

    // 2. Handle subcommands.
    match cli.command {
        Commands::Add { title, due } => {
            // Try parsing due date if provided.
            let parsed_due =
                match due {
                    Some(due_str) => {
                        // We’ll support multiple common formats for convenience.
                        Some(parse_datetime(&due_str).with_context(|| {
                            format!("Failed to parse due date/time: '{}'", due_str)
                        })?)
                    }
                    None => None,
                };

            add_reminder(&mut reminders, title, parsed_due)?;
            println!("Reminder added successfully!");
        }
        Commands::Done { id } => {
            mark_done(&mut reminders, id)?;
        }
        Commands::Remove { id } => {
            remove_reminder(&mut reminders, id)?;
        }
        Commands::List { all } => {
            list_reminders(&reminders, all);
        }
        Commands::ClearCompleted => {
            clear_completed(&mut reminders);
            println!("All completed reminders cleared.");
        }
    }

    // 3. Persist updated reminders back to JSON file.
    save_reminders(&reminders)?;

    Ok(())
}

/// Parse a date/time string using various possible formats.
/// Returns a `DateTime<Local>` on success.
fn parse_datetime(input: &str) -> anyhow::Result<DateTime<Local>> {
    // 1) Try offset-aware parse (RFC 3339, e.g. "2024-12-29T10:00:00-05:00")
    if let Ok(dt_utc) = DateTime::parse_from_rfc3339(input) {
        // Convert to Local from the parsed UTC-based DateTime
        return Ok(dt_utc.with_timezone(&Local));
    }

    // 2) If that fails, treat the input as a naive datetime (no offset) and interpret it as Local
    let formats = &[
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
    ];

    for &fmt in formats {
        // parse_from_str returns a `NaiveDateTime` if successful
        if let Ok(naive) = NaiveDateTime::parse_from_str(input, fmt) {
            // Convert from a naive datetime to a local DateTime
            // `.single()` returns Some if this local time is unambiguous
            if let Some(local_dt) = Local.from_local_datetime(&naive).single() {
                return Ok(local_dt);
            } else {
                // It's either ambiguous or invalid due to DST changes, etc.
                // You could handle that scenario differently if needed.
            }
        }
    }

    anyhow::bail!("Could not parse date/time string: {}", input);
}

/// Load reminders from a JSON file in the user's home directory.
/// If no file exists, return an empty vector.
fn load_reminders() -> Result<Vec<Reminder>> {
    let file_path = get_reminders_file_path()?;

    if !file_path.exists() {
        // If file doesn't exist, we'll start with an empty list.
        return Ok(Vec::new());
    }

    let file =
        File::open(&file_path).with_context(|| format!("Unable to open file {:?}", file_path))?;
    let reader = BufReader::new(file);

    let reminders = serde_json::from_reader(reader)
        .with_context(|| format!("Failed to parse JSON from {:?}", file_path))?;
    Ok(reminders)
}

/// Save reminders to the JSON file, overwriting the old data.
fn save_reminders(reminders: &[Reminder]) -> Result<()> {
    let file_path = get_reminders_file_path()?;
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&file_path)
        .with_context(|| format!("Unable to open file for writing {:?}", file_path))?;

    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, reminders)
        .with_context(|| format!("Failed to write JSON to {:?}", file_path))?;
    Ok(())
}

/// Determine the JSON file path in the user's home directory.
fn get_reminders_file_path() -> Result<PathBuf> {
    let home = home_dir().context("Could not locate home directory")?;
    Ok(home.join(REMINDERS_FILE))
}

/// Add a new reminder to the list, assigning it a unique ID.
fn add_reminder(
    reminders: &mut Vec<Reminder>,
    title: String,
    due: Option<DateTime<Local>>,
) -> Result<()> {
    // We can auto-generate a unique ID by taking the max existing ID and adding 1.
    let new_id = reminders.iter().map(|r| r.id).max().unwrap_or(0) + 1;

    let reminder = Reminder {
        id: new_id,
        title,
        due,
        completed: false,
    };

    reminders.push(reminder);
    Ok(())
}

/// Mark a reminder as completed by its ID.
fn mark_done(reminders: &mut [Reminder], id: usize) -> Result<()> {
    let reminder = reminders
        .iter_mut()
        .find(|r| r.id == id)
        .with_context(|| format!("No reminder found with ID {}", id))?;

    reminder.completed = true;
    println!("Reminder '{}' marked as completed.", reminder.title);
    Ok(())
}

/// Remove a reminder entirely by its ID.
fn remove_reminder(reminders: &mut Vec<Reminder>, id: usize) -> Result<()> {
    let initial_len = reminders.len();
    reminders.retain(|r| r.id != id);

    if reminders.len() == initial_len {
        anyhow::bail!("No reminder found with ID {}", id);
    }

    println!("Reminder with ID {} has been removed.", id);
    Ok(())
}

/// List reminders. By default, we only list incomplete reminders unless `all` is true.
fn list_reminders(reminders: &[Reminder], show_all: bool) {
    let mut to_list: Vec<&Reminder> = if show_all {
        reminders.iter().collect()
    } else {
        reminders.iter().filter(|r| !r.completed).collect()
    };

    // Sort by due date (Option sort—`None` goes last).
    to_list.sort_by_key(|r| r.due.map(|dt| dt.timestamp()));

    if to_list.is_empty() {
        if show_all {
            println!("No reminders found.");
        } else {
            println!("No outstanding (incomplete) reminders.");
        }
        return;
    }

    println!(
        "{} reminders{}:",
        to_list.len(),
        if show_all { " (showing all)" } else { "" }
    );
    for reminder in to_list {
        let due_str = match reminder.due {
            Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
            None => "No due date".to_string(),
        };

        let status = if reminder.completed { "✔ DONE" } else { " " };
        println!(
            "[{status}] ID: {} | Title: {} | Due: {}",
            reminder.id, reminder.title, due_str
        );
    }
}

/// Remove all reminders that are marked completed.
fn clear_completed(reminders: &mut Vec<Reminder>) {
    reminders.retain(|r| !r.completed);
}
