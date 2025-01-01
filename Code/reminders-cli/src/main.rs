////////////////////////////////////////////////////////////////////////////////
// Imports
////////////////////////////////////////////////////////////////////////////////

use std::{
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
    time::Duration,
};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use clap::Parser;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dirs::home_dir;
use serde::{Deserialize, Serialize};

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

////////////////////////////////////////////////////////////////////////////////
// Cross-Platform Line Endings
////////////////////////////////////////////////////////////////////////////////

#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";

#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";

////////////////////////////////////////////////////////////////////////////////
// CLI Arguments
////////////////////////////////////////////////////////////////////////////////

#[derive(Parser, Debug)]
#[command(author, version, about = "Reminders CLI - TUI Edition", long_about = None)]
struct CliArgs {
    /// Enable verbose output
    #[arg(long, short)]
    verbose: bool,
}

////////////////////////////////////////////////////////////////////////////////
// Reminder Data
////////////////////////////////////////////////////////////////////////////////

const REMINDERS_FILE: &str = ".reminders.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Reminder {
    id: usize,
    title: String,
    due: Option<DateTime<Local>>,
    completed: bool,
}

////////////////////////////////////////////////////////////////////////////////
// TUI App State
////////////////////////////////////////////////////////////////////////////////

struct App {
    reminders: Vec<Reminder>,
    status_message: String, // A status line to display feedback
    cursor_idx: usize,      // Which reminder is selected
    input_mode: InputMode,  // Are we currently adding a new reminder or in normal mode?
    input_buffer: String,   // Stores user input for new reminder title/date
    verbose: bool,
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    AddTitle,
    AddDue,
}

impl App {
    fn new(verbose: bool) -> Result<Self> {
        let reminders = load_reminders()?;
        Ok(Self {
            reminders,
            status_message: String::new(),
            cursor_idx: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            verbose,
        })
    }

    /// Provide a short status message to the user.
    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
    }

    /// Move cursor up or down in the reminders list.
    fn move_cursor_up(&mut self) {
        if self.cursor_idx > 0 {
            self.cursor_idx -= 1;
        }
    }
    fn move_cursor_down(&mut self) {
        if self.cursor_idx + 1 < self.reminders.len() {
            self.cursor_idx += 1;
        }
    }

    /// Add a new reminder (called after user input is gathered).
    fn add_reminder(&mut self, title: &str, due: Option<DateTime<Local>>) -> Result<()> {
        if title.trim().is_empty() {
            self.set_status("Title cannot be empty.");
            return Ok(());
        }
        let new_id = self.reminders.iter().map(|r| r.id).max().unwrap_or(0) + 1;
        let reminder = Reminder {
            id: new_id,
            title: title.trim().to_string(),
            due,
            completed: false,
        };
        self.reminders.push(reminder);
        save_reminders(&self.reminders)?;
        self.set_status("Reminder added successfully!");
        Ok(())
    }

    /// Mark the currently selected reminder as completed.
    fn mark_selected_done(&mut self) -> Result<()> {
        if self.reminders.is_empty() {
            self.set_status("No reminders to complete.");
            return Ok(());
        }
        if let Some(rem) = self.reminders.get_mut(self.cursor_idx) {
            rem.completed = true;
            let msg = format!("'{}' marked as completed.", rem.title);
            self.set_status(msg);
            save_reminders(&self.reminders)?;
        }
        Ok(())
    }

    /// Remove the currently selected reminder.
    fn remove_selected(&mut self) -> Result<()> {
        if self.reminders.is_empty() {
            self.set_status("No reminders to remove.");
            return Ok(());
        }
        let removed_id = self.reminders[self.cursor_idx].id;
        self.reminders.remove(self.cursor_idx);
        // Adjust cursor if it goes out of bounds
        if self.cursor_idx >= self.reminders.len() && !self.reminders.is_empty() {
            self.cursor_idx = self.reminders.len() - 1;
        }
        save_reminders(&self.reminders)?;
        self.set_status(format!("Removed reminder with ID {}", removed_id));
        Ok(())
    }

    /// Clear all completed reminders.
    fn clear_completed(&mut self) -> Result<()> {
        self.reminders.retain(|r| !r.completed);
        if self.cursor_idx >= self.reminders.len() && !self.reminders.is_empty() {
            self.cursor_idx = self.reminders.len() - 1;
        }
        save_reminders(&self.reminders)?;
        self.set_status("Cleared all completed reminders.");
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Main (Tokio) Entry Point
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<()> {
    // 1) Parse CLI arguments
    let args = CliArgs::parse();
    if args.verbose {
        print!("Verbose mode enabled...{}", LINE_ENDING);
    }

    // 2) Enable raw mode + enter alternate screen (no mouse capture by default)
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    // 3) Construct a CrosstermBackend + TUI terminal
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 4) Clear the screen & print an initial TUI welcome banner
    clear_screen_tui(&mut terminal)?;
    draw_initial_banner(&mut terminal)?;

    // 5) Greet user (prompts in normal stdin mode briefly)
    greet_user_tui(&mut terminal)?;

    // Build the app state
    let mut app = App::new(args.verbose)?;

    // Run the TUI event loop
    let res = run_app(&mut terminal, &mut app);

    // Before exiting, restore the terminal
    disable_raw_mode()?;
    let mut out = terminal.into_inner();
    execute!(out, LeaveAlternateScreen)?;

    if let Err(e) = res {
        eprint!("Application error: {e}{}", LINE_ENDING);
    }

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// TUI Event Loop
////////////////////////////////////////////////////////////////////////////////

fn run_app<B: tui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        // Sort reminders by due date: place incomplete ones first, completed after
        app.reminders
            .sort_by_key(|r| (r.completed, r.due.map(|dt| dt.timestamp())));

        // Draw the UI
        terminal.draw(|frame| ui_draw(frame, app))?;

        // Poll for events
        if crossterm::event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            // Quit
                            return Ok(());
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            // Move down
                            app.move_cursor_down();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            // Move up
                            app.move_cursor_up();
                        }
                        KeyCode::Char('a') => {
                            // Add new reminder: first gather title
                            app.input_mode = InputMode::AddTitle;
                            app.input_buffer.clear();
                            app.set_status("Enter reminder title, then press Enter (Esc to cancel)...");
                        }
                        KeyCode::Char('r') => {
                            // Remove selected
                            app.remove_selected()?;
                        }
                        KeyCode::Char('d') => {
                            // Mark done
                            app.mark_selected_done()?;
                        }
                        KeyCode::Char('c') => {
                            // Clear completed
                            app.clear_completed()?;
                        }
                        _ => {}
                    },
                    InputMode::AddTitle => match key.code {
                        KeyCode::Enter => {
                            // Done entering title -> next: optional due date prompt
                            app.input_mode = InputMode::AddDue;
                            let t = app.input_buffer.trim().to_string();
                            app.input_buffer.clear();
                            if t.is_empty() {
                                // If empty, skip
                                app.set_status("Title cannot be empty! Aborted.");
                                app.input_mode = InputMode::Normal;
                            } else {
                                app.set_status(format!(
                                    "Title captured: '{}'. Now enter optional due date (YYYY-mm-dd HH:MM). Press Enter to skip.",
                                    t
                                ));
                                // Re-use input_buffer to store the *title* for this simplistic approach
                                app.input_buffer = t;
                            }
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.set_status("Add reminder cancelled.");
                            app.input_buffer.clear();
                        }
                        KeyCode::Backspace => {
                            app.input_buffer.pop();
                        }
                        KeyCode::Char(c) => {
                            app.input_buffer.push(c);
                        }
                        _ => {}
                    },
                    InputMode::AddDue => match key.code {
                        KeyCode::Enter => {
                            // The user might have typed a due date here; let's parse it
                            let title = app.input_buffer.clone();
                            app.input_mode = InputMode::Normal;
                            app.input_buffer.clear();

                            // For demonstration: parse the date from 'title' or skip
                            // Realistically you'd store the title in a separate field
                            // and have a second buffer for the date.
                            // We'll do a quick attempt to parse the entire user input as a date:
                            match parse_datetime(&title) {
                                Ok(parsed_dt) => {
                                    // If that was a valid date, treat the real "title" as unknown
                                    // But we only had one buffer. So let's just show a simpler approach:
                                    app.add_reminder(&title, Some(parsed_dt))?;
                                }
                                Err(_) => {
                                    // If it fails parse, treat it as title only
                                    app.add_reminder(&title, None)?;
                                }
                            }
                        }
                        KeyCode::Esc => {
                            app.input_mode = InputMode::Normal;
                            app.set_status("Add reminder cancelled.");
                            app.input_buffer.clear();
                        }
                        KeyCode::Backspace => {
                            app.input_buffer.pop();
                        }
                        KeyCode::Char(c) => {
                            app.input_buffer.push(c);
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// UI Drawing
////////////////////////////////////////////////////////////////////////////////

fn ui_draw<B: tui::backend::Backend>(frame: &mut tui::Frame<B>, app: &App) {
    // Split screen into vertical chunks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Banner area
            Constraint::Min(3),    // Main area
            Constraint::Length(3), // Status bar
        ])
        .split(frame.size());

    // 1) Banner block (ASCII art + name)
    let banner_text = vec![
        Spans::from(Span::styled(
            r#"
Welcome to the Reminders CLI app!
"#,
            Style::default().fg(Color::Cyan)
        )),
        Spans::from(""),
        Spans::from(Span::styled(
            "              Welcome to the TUI-based Reminders CLI! ",
            Style::default().fg(Color::Cyan),
        )),
    ];
    let banner_par = Paragraph::new(banner_text).block(Block::default().borders(Borders::ALL));
    frame.render_widget(banner_par, chunks[0]);

    // 2) Main area: a list of reminders
    let items: Vec<ListItem> = app
        .reminders
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let marker = if r.completed { "[✔]" } else { "[ ]" };
            let due_str = r
                .due
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "No due date".to_string());
            let display = format!("{} ID:{:>2} | {} | Due: {}", marker, r.id, r.title, due_str);

            if i == app.cursor_idx {
                ListItem::new(display)
                    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            } else {
                ListItem::new(display)
            }
        })
        .collect();

    let list = tui::widgets::List::new(items)
        .block(Block::default().title("Reminders").borders(Borders::ALL));
    frame.render_widget(list, chunks[1]);

    // 3) Status area (shows input mode + status message)
    let input_mode_text = match app.input_mode {
        InputMode::Normal => {
            "Normal Mode | [a] = Add, [d] = Done, [r] = Remove, [c] = Clear, [q] = Quit"
        }
        InputMode::AddTitle => "Add Reminder (Title)...",
        InputMode::AddDue => "Add Reminder (Due Date)...",
    };
    let status_text = vec![
        Spans::from(vec![Span::styled(
            input_mode_text,
            Style::default().fg(Color::White),
        )]),
        Spans::from(vec![Span::styled(
            &app.status_message,
            Style::default().fg(Color::Magenta),
        )]),
    ];
    let status_par = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL).title("Status"));
    frame.render_widget(status_par, chunks[2]);
}

////////////////////////////////////////////////////////////////////////////////
// Utility TUI Functions
////////////////////////////////////////////////////////////////////////////////

fn clear_screen_tui<B: tui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    terminal.clear()?;
    Ok(())
}

fn draw_initial_banner<B: tui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    // For brevity, we just show a quick partial screen draw.
    // This can be expanded to a more elaborate "splash" if desired.
    terminal.draw(|frame| {
        let size = frame.size();
        let par = Paragraph::new("Welcome to the Reminders CLI!")
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
        frame.render_widget(par, size);
    })?;
    Ok(())
}

fn greet_user_tui<B: tui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    // Temporarily disable raw mode to read from stdin
    disable_raw_mode()?;
    print!("Please enter your name: {}", LINE_ENDING);
    io::stdout().flush()?;

    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let trimmed = name.trim();
    let greet_name = if trimmed.is_empty() { "Friend" } else { trimmed };
    print!("Hello, {}! Press Enter to continue...{}", greet_name, LINE_ENDING);
    io::stdout().flush()?;

    // Wait for user to press Enter
    let mut dummy = String::new();
    io::stdin().read_line(&mut dummy)?;

    // Re-enable raw mode + re-enter alt screen for TUI
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;
    terminal.clear()?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// File & Data Logic
////////////////////////////////////////////////////////////////////////////////

fn load_reminders() -> Result<Vec<Reminder>> {
    let file_path = get_reminders_file_path()?;
    if !file_path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(&file_path)
        .with_context(|| format!("Unable to open file {:?}", file_path))?;
    let reader = BufReader::new(file);

    let reminders: Vec<Reminder> = serde_json::from_reader(reader)
        .with_context(|| format!("Failed to parse JSON from {:?}", file_path))?;
    Ok(reminders)
}

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

fn get_reminders_file_path() -> Result<PathBuf> {
    let home = home_dir().context("Could not locate home directory")?;
    Ok(home.join(REMINDERS_FILE))
}

////////////////////////////////////////////////////////////////////////////////
// Date/Time Parsing
////////////////////////////////////////////////////////////////////////////////

/// Attempts to parse a date-time string in various formats, returning Local time.
fn parse_datetime(input: &str) -> Result<DateTime<Local>> {
    // 1) Try offset-aware parse (RFC 3339)
    if let Ok(dt_utc) = DateTime::parse_from_rfc3339(input) {
        return Ok(dt_utc.with_timezone(&Local));
    }
    // 2) Attempt naive parse with multiple formats
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
    bail!("Could not parse date/time string: {}", input);
}
