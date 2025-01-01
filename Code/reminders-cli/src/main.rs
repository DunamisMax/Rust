////////////////////////////////////////////////////////////////////////////////
// reminders-cli - A Ratatui-based TUI for managing reminders
////////////////////////////////////////////////////////////////////////////////

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
    cursor::MoveTo,
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use dirs::home_dir;
use serde::{Deserialize, Serialize};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
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
#[command(author, version, about = "Reminders CLI - Ratatui Edition", long_about = None)]
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
// RAII Guard for Raw Mode
////////////////////////////////////////////////////////////////////////////////

struct RawModeGuard {
    active: bool,
}

impl RawModeGuard {
    fn new() -> Result<Self> {
        enable_raw_mode().context("Unable to enable raw mode")?;
        Ok(Self { active: true })
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = disable_raw_mode();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// TUI App State
////////////////////////////////////////////////////////////////////////////////

struct App {
    reminders: Vec<Reminder>,
    status_message: String, // A status line to display feedback
    cursor_idx: usize,      // Which reminder is selected
    input_mode: InputMode,  // Are we adding a new reminder or in normal mode?
    input_buffer: String,   // Stores user input for new reminder
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

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = msg.into();
    }

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

    fn remove_selected(&mut self) -> Result<()> {
        if self.reminders.is_empty() {
            self.set_status("No reminders to remove.");
            return Ok(());
        }
        let removed_id = self.reminders[self.cursor_idx].id;
        self.reminders.remove(self.cursor_idx);
        if self.cursor_idx >= self.reminders.len() && !self.reminders.is_empty() {
            self.cursor_idx = self.reminders.len() - 1;
        }
        save_reminders(&self.reminders)?;
        self.set_status(format!("Removed reminder with ID {}", removed_id));
        Ok(())
    }

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
// Main (Tokio) Entry
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<()> {
    // 1) Parse CLI arguments
    let args = CliArgs::parse();
    if args.verbose {
        print!("Verbose mode enabled...{}", LINE_ENDING);
    }

    // 2) Enable raw mode via RAII guard
    let _raw_guard = RawModeGuard::new().context("Failed to enable raw mode")?;

    // 3) Create Terminal & clear screen (enter alt screen)
    let mut terminal = setup_terminal().context("Failed to create terminal")?;
    clear_screen(&mut terminal)?;

    // 4) Draw the welcome TUI
    draw_welcome_screen(&mut terminal)?;

    // 5) Temporarily drop raw mode to prompt for name
    drop(_raw_guard);
    greet_user_cli()?;

    // 6) Re-enable raw mode to run the main TUI loop
    let _raw_guard = RawModeGuard::new().context("Failed to re-enable raw mode")?;
    let mut terminal = setup_terminal().context("Failed to create terminal")?;
    clear_screen(&mut terminal)?;

    // Create the app state
    let mut app = App::new(args.verbose)?;

    // 7) Run TUI event loop
    if let Err(e) = run_app(&mut terminal, &mut app) {
        disable_raw_mode()?; // explicitly exit raw mode on error
        eprintln!("Application error: {e}");
    }

    // 8) Drop raw mode so user can press Enter to exit
    drop(_raw_guard);

    println!("{}", LINE_ENDING);
    println!("{}", LINE_ENDING);
    print!("Press Enter to exit...{}", LINE_ENDING);
    io::stdout().flush()?;
    let mut exit_buf = String::new();
    io::stdin().read_line(&mut exit_buf)?;

    // 9) Cleanup: clear screen & say goodbye
    execute!(terminal.backend_mut(), Clear(ClearType::All), MoveTo(0, 0))?;
    print!("Goodbye!{}", LINE_ENDING);

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Setup Terminal & Clear
////////////////////////////////////////////////////////////////////////////////

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn clear_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    terminal.clear()?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Draw a small "Welcome" TUI (banner area + minimal text)
////////////////////////////////////////////////////////////////////////////////

fn draw_welcome_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    terminal.draw(|frame| {
        let size = frame.area();

        // Simple banner
        let banner_lines = vec![
            Line::from(Span::styled(
                "Reminders CLI",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Manage your tasks effortlessly!",
                Style::default().fg(Color::Yellow),
            )),
        ];
        let paragraph = Paragraph::new(banner_lines)
            .block(Block::default().borders(Borders::ALL).title(" Welcome "));

        frame.render_widget(paragraph, size);
    })?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Prompt for user's name outside of raw mode
////////////////////////////////////////////////////////////////////////////////

fn greet_user_cli() -> Result<()> {
    disable_raw_mode()?; // Just to be sure
    println!("{}", LINE_ENDING);
    println!("Please enter your name:{}", LINE_ENDING);
    print!("> ");
    io::stdout().flush()?;

    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let trimmed = name.trim();

    println!(
        "{}Hello, {}! Press Enter to continue...{}",
        LINE_ENDING,
        if trimmed.is_empty() { "Friend" } else { trimmed },
        LINE_ENDING
    );
    io::stdout().flush()?;

    let mut dummy = String::new();
    io::stdin().read_line(&mut dummy)?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// TUI Event Loop
////////////////////////////////////////////////////////////////////////////////

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        // Sort reminders by (completed, due)
        app.reminders
            .sort_by_key(|r| (r.completed, r.due.map(|dt| dt.timestamp())));

        // Draw the UI
        terminal.draw(|frame| draw_main_ui(frame, app))?;

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
                            // Add new reminder title
                            app.input_mode = InputMode::AddTitle;
                            app.input_buffer.clear();
                            app.set_status("Enter title, then press Enter (Esc to cancel)...");
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
                            // Title done -> ask for optional due date
                            let t = app.input_buffer.trim().to_string();
                            app.input_buffer.clear();
                            if t.is_empty() {
                                app.set_status("Title cannot be empty! Aborted.");
                                app.input_mode = InputMode::Normal;
                            } else {
                                app.set_status(format!(
                                    "Got title: '{}'. Now enter optional due date (YYYY-mm-dd HH:MM). Press Enter to skip.",
                                    t
                                ));
                                // Re-use input_buffer for the title
                                app.input_buffer = t;
                                app.input_mode = InputMode::AddDue;
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
                            // Try parse date
                            let title = app.input_buffer.clone();
                            app.input_mode = InputMode::Normal;
                            app.input_buffer.clear();
                            match parse_datetime(&title) {
                                Ok(parsed_dt) => {
                                    // If user input is actually a date, treat the entire buffer as a date
                                    // In a more advanced approach, you'd store the title + date separately
                                    app.add_reminder(&title, Some(parsed_dt))?;
                                }
                                Err(_) => {
                                    // If parse fails, treat it as a title-only
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
// Main UI Drawing
////////////////////////////////////////////////////////////////////////////////

fn draw_main_ui<B: ratatui::backend::Backend>(frame: &mut Frame<B>, app: &App) {
    // Split screen into banner, main list, status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Banner area
            Constraint::Min(3),    // Main area
            Constraint::Length(3), // Status bar
        ])
        .split(frame.area());

    // Banner
    let banner = Paragraph::new(Line::from(Span::styled(
        "Reminders CLI - [j/k: navigate] [a: add] [d: done] [r: remove] [c: clear] [q: quit]",
        Style::default().fg(Color::Cyan),
    )))
    .block(Block::default().borders(Borders::ALL).title(" Banner "));
    frame.render_widget(banner, chunks[0]);

    // Reminders list
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
            let text = format!("{} ID:{:>2} | {} | Due: {}", marker, r.id, r.title, due_str);

            if i == app.cursor_idx {
                ListItem::new(text).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(text)
            }
        })
        .collect();

    let reminders_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Reminders "));
    frame.render_widget(reminders_list, chunks[1]);

    // Status bar
    let mode_text = match app.input_mode {
        InputMode::Normal => "Mode: Normal",
        InputMode::AddTitle => "Mode: Adding Title",
        InputMode::AddDue => "Mode: Adding Due Date",
    };

    let status_lines = vec![
        Line::from(Span::raw(mode_text)),
        Line::from(Span::styled(
            &app.status_message,
            Style::default().fg(Color::Magenta),
        )),
    ];

    let status_par = Paragraph::new(status_lines)
        .block(Block::default().borders(Borders::ALL).title(" Status "));
    frame.render_widget(status_par, chunks[2]);
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
    let reminders: Vec<Reminder> =
        serde_json::from_reader(reader).with_context(|| "Failed to parse JSON")?;
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
    serde_json::to_writer_pretty(writer, reminders).with_context(|| "Failed to write JSON")?;
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

    bail!("Could not parse date/time string: {}", input)
}
