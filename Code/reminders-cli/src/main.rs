use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
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

/// Asynchronous entry point using Tokio.
#[tokio::main]
async fn main() -> Result<()> {
    clear_screen()?;
    print_welcome_banner()?;
    prompt_and_greet()?;

    // Load existing reminders from file (or create an empty file if none exists).
    let mut reminders = load_reminders()?;

    // Start the interactive menu loop.
    menu_loop(&mut reminders)?;

    Ok(())
}

/// Clears the terminal screen for a clean start using crossterm.
fn clear_screen() -> Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    Ok(())
}

/// Prints a banner with ASCII art, then an initial welcome message.
fn print_welcome_banner() -> Result<()> {
    let banner = r#"
                        _             _                          _  _
                       (_)           | |                        | |(_)
 _ __   ___  _ __ ___   _  _ __    __| |  ___  _ __  ___    ___ | | _
| '__| / _ \| '_ ` _ \ | || '_ \  / _` | / _ \| '__|/ __|  / __|| || |
| |   |  __/| | | | | || || | | || (_| ||  __/| |   \__ \ | (__ | || |
|_|    \___||_| |_| |_||_||_| |_| \__,_| \___||_|   |___/  \___||_||_|
    "#;

    cprintln_banner(banner)?;
    cprintln_info("Welcome to the Reminders CLI!\r\n")?;
    Ok(())
}

/// Prompts the user for their name and greets them.
fn prompt_and_greet() -> Result<()> {
    cprintln_info("Before we begin, let's get acquainted!\r\n")?;

    // Prompt for user’s name
    cprint_info("What is your name?\r\n")?;
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .context("Failed to read input from stdin")?;

    let trimmed = name.trim();
    if trimmed.is_empty() {
        greet("Friend")?;
    } else {
        greet(trimmed)?;
    }
    Ok(())
}

/// Prints a simple greeting message.
fn greet(name: &str) -> Result<()> {
    let greeting = format!("Hello, {}! Enjoy the Reminders CLI!\r\n", name);
    cprintln_info(&greeting)?;
    Ok(())
}

/// The main loop that presents the user with menu options.
fn menu_loop(reminders: &mut Vec<Reminder>) -> Result<()> {
    loop {
        cprintln_info("===== MAIN MENU =====")?;
        cprintln_info("1) List all reminders")?;
        cprintln_info("2) Add a new reminder")?;
        cprintln_info("3) Mark a reminder as completed")?;
        cprintln_info("4) Remove a reminder")?;
        cprintln_info("5) Clear all completed reminders")?;
        cprintln_info("6) Quit")?;
        cprintln_info("=====================")?;

        // Prompt user for choice
        let choice = prompt_info("Enter a choice (1-6): ")?;

        match choice.trim() {
            "1" => list_reminders_interactive(reminders)?,
            "2" => add_reminder_interactive(reminders)?,
            "3" => mark_done_interactive(reminders)?,
            "4" => remove_reminder_interactive(reminders)?,
            "5" => {
                clear_completed(reminders);
                cprintln_success("All completed reminders cleared.\r\n")?;
                // Persist after clearing
                save_reminders(reminders)?;
            }
            "6" => {
                cprintln_info("Goodbye!\r\n")?;
                break;
            }
            _ => cprintln_warning("Invalid choice. Please enter a number between 1 and 6.\r\n")?,
        }
    }

    Ok(())
}

/// Presents all reminders (including completed) for the user to see.
fn list_reminders_interactive(reminders: &[Reminder]) -> Result<()> {
    if reminders.is_empty() {
        cprintln_warning("No reminders found.\r\n")?;
        return Ok(());
    }

    let mut to_list: Vec<&Reminder> = reminders.iter().collect();
    // Sort by due date (Option sort—`None` goes last).
    to_list.sort_by_key(|r| r.due.map(|dt| dt.timestamp()));

    let msg = format!("You have {} reminders in total:\r\n", to_list.len());
    cprintln_info(&msg)?;

    for reminder in to_list {
        let due_str = match reminder.due {
            Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
            None => "No due date".to_string(),
        };

        let status = if reminder.completed {
            "[✔ DONE]".to_owned()
        } else {
            "[ ]".to_owned()
        };

        let line = format!(
            "{status} ID: {} | Title: {} | Due: {}\r\n",
            reminder.id, reminder.title, due_str
        );
        cprintln_info(&line)?;
    }

    Ok(())
}

/// Interactive function to add a new reminder by prompting user for a title and optional due date.
fn add_reminder_interactive(reminders: &mut Vec<Reminder>) -> Result<()> {
    cprintln_info("\r\n--- Add a New Reminder ---\r\n")?;

    let title = prompt_info("Enter the reminder title: ")?;
    if title.trim().is_empty() {
        cprintln_warning("Title cannot be empty.\r\n")?;
        return Ok(());
    }

    let due_input = prompt_info("Enter a due date/time (optional, e.g. '2024-12-29 10:00'): ")?;

    // Attempt to parse the due date/time if provided.
    let parsed_due = if due_input.trim().is_empty() {
        None
    } else {
        match parse_datetime(due_input.trim()) {
            Ok(dt) => Some(dt),
            Err(e) => {
                let err_msg = format!("Invalid date/time: {}\r\n", e);
                cprintln_error(&err_msg)?;
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

    cprintln_success("Reminder added successfully!\r\n")?;
    Ok(())
}

/// Interactive function to mark a reminder as completed by ID.
fn mark_done_interactive(reminders: &mut [Reminder]) -> Result<()> {
    cprintln_info("\r\n--- Mark Reminder as Completed ---\r\n")?;

    let input = prompt_info("Enter the ID of the reminder to mark as completed: ")?;
    let id: usize = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            cprintln_warning("Invalid ID. Please enter a number.\r\n")?;
            return Ok(());
        }
    };

    // Mutably iterate to find the matching reminder
    if let Some(r) = reminders.iter_mut().find(|reminder| reminder.id == id) {
        r.completed = true;
        let msg = format!("Reminder '{}' marked as completed.\r\n", r.title);
        cprintln_success(&msg)?;
        // Persist changes
        save_reminders(reminders)?;
    } else {
        let err_msg = format!("No reminder found with ID {}\r\n", id);
        cprintln_warning(&err_msg)?;
    }

    Ok(())
}

/// Interactive function to remove a reminder by ID.
fn remove_reminder_interactive(reminders: &mut Vec<Reminder>) -> Result<()> {
    cprintln_info("\r\n--- Remove a Reminder ---\r\n")?;

    let input = prompt_info("Enter the ID of the reminder to remove: ")?;
    let id: usize = match input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            cprintln_warning("Invalid ID. Please enter a number.\r\n")?;
            return Ok(());
        }
    };

    let initial_len = reminders.len();
    reminders.retain(|r| r.id != id);

    if reminders.len() == initial_len {
        let err_msg = format!("No reminder found with ID {}\r\n", id);
        cprintln_warning(&err_msg)?;
    } else {
        let success_msg = format!("Reminder with ID {} has been removed.\r\n", id);
        cprintln_success(&success_msg)?;
        save_reminders(reminders)?;
    }

    Ok(())
}

/// Remove all reminders that are marked completed.
fn clear_completed(reminders: &mut Vec<Reminder>) {
    reminders.retain(|r| !r.completed);
}

/// Helper function to prompt the user for input and read it from stdin (standard info color).
fn prompt_info(message: &str) -> Result<String> {
    cprint_info(message)?;
    io::stdout().flush().ok(); // Ensure the prompt is displayed immediately

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;
    Ok(buffer)
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

//
// Below are helper functions to print text with crossterm + \r\n line endings,
// each using a specific color to maintain a consistent theme.
//

// Theming constants
const BANNER_COLOR: Color = Color::Cyan;
const INFO_COLOR: Color = Color::White;
const SUCCESS_COLOR: Color = Color::Green;
const WARNING_COLOR: Color = Color::Yellow;
const ERROR_COLOR: Color = Color::Red;

/// Print banner text in `BANNER_COLOR`.
fn cprintln_banner(text: &str) -> Result<()> {
    cprintln_color(text, BANNER_COLOR)
}

/// Print normal info text in `INFO_COLOR`.
fn cprintln_info(text: &str) -> Result<()> {
    cprintln_color(text, INFO_COLOR)
}

/// Print success text in `SUCCESS_COLOR`.
fn cprintln_success(text: &str) -> Result<()> {
    cprintln_color(text, SUCCESS_COLOR)
}

/// Print warning text in `WARNING_COLOR`.
fn cprintln_warning(text: &str) -> Result<()> {
    cprintln_color(text, WARNING_COLOR)
}

/// Print error text in `ERROR_COLOR`.
fn cprintln_error(text: &str) -> Result<()> {
    cprintln_color(text, ERROR_COLOR)
}

/// Helper to print text with a specific color + `\r\n`.
fn cprintln_color(text: &str, color: Color) -> Result<()> {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        SetForegroundColor(color),
        Print(text),
        ResetColor,
        Print("\r\n")
    )?;
    Ok(())
}

/// Similar to `cprintln_color` but doesn't add a newline automatically.
/// Used for prompts that continue on the same line.
fn cprint_info(text: &str) -> Result<()> {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        SetForegroundColor(INFO_COLOR),
        Print(text),
        ResetColor
    )?;
    Ok(())
}
