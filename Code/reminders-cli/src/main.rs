use std::{
    fs::{File, OpenOptions},
    io::{stdin, stdout, BufReader, BufWriter, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use colored::*;
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

fn main() -> Result<()> {
    // Print a welcome banner with some ASCII styling.
    print_banner();

    // Load existing reminders from file (or create an empty file if none exists).
    let mut reminders = load_reminders()?;

    // Start the interactive menu loop.
    menu_loop(&mut reminders)?;

    Ok(())
}

/// The main loop that presents the user with menu options.
fn menu_loop(reminders: &mut Vec<Reminder>) -> Result<()> {
    loop {
        println!("{}", "===== MAIN MENU =====".bright_white().bold());
        println!("1) List all reminders");
        println!("2) Add a new reminder");
        println!("3) Mark a reminder as completed");
        println!("4) Remove a reminder");
        println!("5) Clear all completed reminders");
        println!("6) Quit");
        println!("{}", "=====================".bright_white().bold());

        // Prompt user for choice
        let choice = prompt("Enter a choice (1-6): ")?;

        match choice.trim() {
            "1" => list_reminders_interactive(reminders)?,
            "2" => add_reminder_interactive(reminders)?,
            "3" => mark_done_interactive(reminders)?,
            "4" => remove_reminder_interactive(reminders)?,
            "5" => {
                clear_completed(reminders);
                println!("{}", "All completed reminders cleared.".green().bold());
                // Persist after clearing
                save_reminders(reminders)?;
            }
            "6" => {
                println!("{}", "Goodbye!".bright_blue().bold());
                break;
            }
            _ => println!(
                "{}",
                "Invalid choice. Please enter a number between 1 and 6.".red()
            ),
        }
    }

    Ok(())
}

/// Helper function to print an ASCII-art banner to liven up the CLI.
fn print_banner() {
    let banner = r#"
                        _             _                          _  _
                       (_)           | |                        | |(_)
 _ __   ___  _ __ ___   _  _ __    __| |  ___  _ __  ___    ___ | | _
| '__| / _ \| '_ ` _ \ | || '_ \  / _` | / _ \| '__|/ __|  / __|| || |
| |   |  __/| | | | | || || | | || (_| ||  __/| |   \__ \ | (__ | || |
|_|    \___||_| |_| |_||_||_| |_| \__,_| \___||_|   |___/  \___||_||_|

"#;

    println!("{}", banner.bright_yellow().bold());
}

/// Helper function to prompt the user for input and read it from stdin.
fn prompt(message: &str) -> Result<String> {
    print!("{}", message);
    stdout().flush().ok(); // Ensure the prompt is displayed immediately

    let mut buffer = String::new();
    stdin().read_line(&mut buffer)?;
    Ok(buffer)
}

/// Presents all reminders (including completed) for the user to see.
fn list_reminders_interactive(reminders: &[Reminder]) -> Result<()> {
    if reminders.is_empty() {
        println!("{}", "No reminders found.".yellow().bold());
        return Ok(());
    }

    let mut to_list: Vec<&Reminder> = reminders.iter().collect();
    // Sort by due date (Option sort—`None` goes last).
    to_list.sort_by_key(|r| r.due.map(|dt| dt.timestamp()));

    println!();
    println!(
        "{}",
        format!("You have {} reminders in total:", to_list.len())
            .bright_white()
            .bold()
    );

    for reminder in to_list {
        let due_str = match reminder.due {
            Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
            None => "No due date".to_string(),
        };

        let status = if reminder.completed {
            format!("{}", "✔ DONE".green().bold())
        } else {
            format!("{}", "[ ]".yellow().bold())
        };

        println!(
            "[{status}] {} | {} | {}",
            format!("ID: {}", reminder.id).magenta().bold(),
            format!("Title: {}", reminder.title).cyan().bold(),
            format!("Due: {due_str}").blue()
        );
    }
    println!();

    Ok(())
}

/// Interactive function to add a new reminder by prompting user for a title and optional due date.
fn add_reminder_interactive(reminders: &mut Vec<Reminder>) -> Result<()> {
    println!("{}", "\n--- Add a New Reminder ---".bright_white().bold());

    let title = prompt("Enter the reminder title: ")?;
    if title.trim().is_empty() {
        println!("{}", "Title cannot be empty.".red());
        return Ok(());
    }

    let due_input = prompt("Enter a due date/time (optional, e.g. '2024-12-29 10:00'): ")?;

    // Attempt to parse the due date/time if provided.
    let parsed_due = if due_input.trim().is_empty() {
        None
    } else {
        match parse_datetime(due_input.trim()) {
            Ok(dt) => Some(dt),
            Err(e) => {
                println!("{}", format!("Invalid date/time: {}", e).red());
                return Ok(());
            }
        }
    };

    // Generate a new ID
    let new_id = reminders.iter().map(|r| r.id).max().unwrap_or(0) + 1;
    let reminder = Reminder {
        id: new_id,
        title: title.trim().to_string(),
        due: parsed_due,
        completed: false,
    };

    reminders.push(reminder);
    save_reminders(reminders)?; // Persist the addition immediately

    println!("{}", "Reminder added successfully!\n".green().bold());
    Ok(())
}

/// Interactive function to mark a reminder as completed by ID.
fn mark_done_interactive(reminders: &mut [Reminder]) -> Result<()> {
    println!(
        "{}",
        "\n--- Mark Reminder as Completed ---".bright_white().bold()
    );

    let input = prompt("Enter the ID of the reminder to mark as completed: ")?;
    let id: usize = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("{}", "Invalid ID. Please enter a number.".red());
            return Ok(());
        }
    };

    // Mutably iterate to find the matching reminder
    if let Some(r) = reminders.iter_mut().find(|reminder| reminder.id == id) {
        r.completed = true;
        println!(
            "{}",
            format!("Reminder '{}' marked as completed.", r.title)
                .green()
                .bold()
        );
        // We can still save, because `&mut [Reminder]` can be passed to a function expecting `&[Reminder]`.
    } else {
        println!("{}", format!("No reminder found with ID {}", id).red());
    }

    Ok(())
}

/// Interactive function to remove a reminder by ID.
fn remove_reminder_interactive(reminders: &mut Vec<Reminder>) -> Result<()> {
    println!("{}", "\n--- Remove a Reminder ---".bright_white().bold());

    let input = prompt("Enter the ID of the reminder to remove: ")?;
    let id: usize = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("{}", "Invalid ID. Please enter a number.".red());
            return Ok(());
        }
    };

    let initial_len = reminders.len();
    reminders.retain(|r| r.id != id);

    if reminders.len() == initial_len {
        println!("{}", format!("No reminder found with ID {}", id).red());
    } else {
        println!(
            "{}",
            format!("Reminder with ID {} has been removed.", id)
                .red()
                .bold()
        );
        save_reminders(reminders)?;
    }

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
        if let Ok(naive) = NaiveDateTime::parse_from_str(input, fmt) {
            // `.single()` returns Some if this local time is unambiguous
            if let Some(local_dt) = Local.from_local_datetime(&naive).single() {
                return Ok(local_dt);
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

    let reminders: Vec<Reminder> = serde_json::from_reader(reader)
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

/// Remove all reminders that are marked completed.
fn clear_completed(reminders: &mut Vec<Reminder>) {
    reminders.retain(|r| !r.completed);
}
