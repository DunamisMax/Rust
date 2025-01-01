////////////////////////////////////////////////////////////////////////////////
// Imports
////////////////////////////////////////////////////////////////////////////////

use anyhow::{anyhow, Result};
use clap::Parser;

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
//
// If you actually want mouse capture, uncomment the following lines:
// use crossterm::event::EnableMouseCapture;
// use crossterm::event::DisableMouseCapture;
// use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal, Frame,
};

use ring::{aead, pbkdf2, rand as ring_rand};
use serde::{Deserialize, Serialize};

use std::{
    fs::OpenOptions,
    io::{self, Read, Write},
    num::NonZeroU32,
    path::Path,
    time::Duration,
};

////////////////////////////////////////////////////////////////////////////////
// Cross-Platform Line Endings
////////////////////////////////////////////////////////////////////////////////

#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";

#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";

////////////////////////////////////////////////////////////////////////////////
// Clap Arguments
////////////////////////////////////////////////////////////////////////////////

#[derive(Parser, Debug)]
#[command(author, version, about = "Secure Notes CLI", long_about = None)]
struct CliArgs {
    /// Optional path to the encrypted notes file
    #[arg(long, short, default_value = "secure_notes.json.enc")]
    file: String,
}

////////////////////////////////////////////////////////////////////////////////
// SALT & PBKDF2 CONFIG (Demo Purposes Only)
////////////////////////////////////////////////////////////////////////////////

/// In production, each user typically requires a unique salt and higher iteration count.
/// This static salt is only for demonstration.
const SALT: &[u8] = b"fixed-salt-demo";
const PBKDF2_ITERATIONS: u32 = 100_000;

////////////////////////////////////////////////////////////////////////////////
// Data Structures
////////////////////////////////////////////////////////////////////////////////

/// Generates a user-friendly 6-digit numeric ID.
fn generate_user_friendly_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let number: u32 = rng.gen_range(0..=999999);
    format!("{:06}", number)
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct Note {
    id: String,
    title: String,
    content: String,
}

/// Tracks which TUI screen we’re on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    PasswordPrompt,
    Menu,
    ViewNotes,
    CreateNote,
    EditNote,
    DeleteNote,
    OpenNote,
    DeleteAll,
    Exit,
}

/// For note editing, we keep track of which note ID we’re editing, plus the text buffer.
#[derive(Debug, Clone)]
struct EditState {
    note_id: Option<String>,
    buffer: String,
}

/// Main TUI app state.
struct App {
    password: String,      // Master password
    key: [u8; 32],         // Derived encryption key
    notes: Vec<Note>,      // All notes
    screen: Screen,        // Current screen
    input_buffer: String,  // Generic input (e.g. for prompts)
    edit_state: EditState, // State used during note create/edit
    error_message: String, // Display any errors to user
    file_path: String,     // Encrypted notes file path
}

////////////////////////////////////////////////////////////////////////////////
// Main (Tokio) Entry Point
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<()> {
    // 1) Parse CLI arguments
    let args = CliArgs::parse();

    // 2) Enable raw mode for TUI
    enable_raw_mode()?;

    // 3) Construct a CrosstermBackend
    //
    // If you do want to capture mouse events + alternate screen, uncomment below:
    /*
    let mut stdout = io::stdout();
    execute!(stdout, EnableMouseCapture, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    */
    // Otherwise, do:
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // 4) Clear the screen using TUI
    clear_screen(&mut terminal)?;

    // 5) Print a welcome banner once at app start (Paragraph-based)
    draw_welcome_banner(&mut terminal)?;

    // 6) Print a quick message with cross-platform line ending
    print!("CLI started successfully!{}", LINE_ENDING);

    // 7) Build initial app state
    let app = App {
        password: String::new(),
        key: [0u8; 32],
        notes: Vec::new(),
        screen: Screen::PasswordPrompt, // Start by prompting for password
        input_buffer: String::new(),
        edit_state: EditState {
            note_id: None,
            buffer: String::new(),
        },
        error_message: String::new(),
        file_path: args.file, // from CLI args
    };

    // 8) Run the main TUI loop
    let result = run_app(&mut terminal, app);

    // 9) Before exiting, restore terminal to normal mode
    disable_raw_mode()?;
    // If you used alternate screen + mouse capture, uncomment:
    /*
    execute!(
        terminal.backend_mut(),
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    */
    // Otherwise, just clear the entire screen and move cursor to (0,0):
    execute!(
        terminal.backend_mut(),
        Clear(ClearType::All),
        MoveTo(0, 0)
    )?;
    print!("Goodbye!{}", LINE_ENDING);

    // If the app returned an error, display it
    if let Err(e) = result {
        eprint!("Error: {:?}{}", e, LINE_ENDING);
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Utility Functions
////////////////////////////////////////////////////////////////////////////////

/// Clears the terminal screen for a clean start using tui.
fn clear_screen<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    terminal.clear()?;
    Ok(())
}

/// A single-time banner at app launch (separate from the “always-present” TUI banner).
fn draw_welcome_banner<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    // A minimal text banner for launch
    let banner = r#"
    Welcome to Secure Notes!
    ========================
    Your encrypted note management system
    "#;

    // Use a Paragraph to display the banner in the center
    let size = terminal.size()?;
    let block = Paragraph::new(banner)
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().fg(Color::Cyan));

    terminal.draw(|f| {
        f.render_widget(block, size);
    })?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Main TUI Loop
////////////////////////////////////////////////////////////////////////////////

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    loop {
        // Render the UI for the current state
        terminal.draw(|frame| draw_ui(frame, &app))?;

        // Poll for events (non-blocking)
        if crossterm::event::poll(Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(key_event) => {
                    handle_key_event(key_event, &mut app)?;
                }
                Event::Mouse(_) => {
                    // We won’t handle mouse in this minimal example
                }
                _ => {}
            }
        }

        // If user’s on the Exit screen, break out
        if app.screen == Screen::Exit {
            break;
        }
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// TUI Drawing
////////////////////////////////////////////////////////////////////////////////

/// Draws the entire TUI, selecting which “screen” to show.
fn draw_ui<B: Backend>(frame: &mut Frame<B>, app: &App) {
    let area = frame.size();

    // Always draw a top banner (ASCII art) at the top
    draw_banner(frame, area);

    // Dispatch to each screen below
    match app.screen {
        Screen::PasswordPrompt => draw_password_prompt(frame, app, area),
        Screen::Menu => draw_main_menu(frame, app, area),
        Screen::ViewNotes => draw_view_notes(frame, app, area),
        Screen::CreateNote | Screen::EditNote => draw_note_editor(frame, app, area),
        Screen::DeleteNote | Screen::OpenNote | Screen::DeleteAll => {
            draw_simple_input(frame, app, area)
        }
        Screen::Exit => {
            // Nothing special
        }
    }

    // Draw any error message at the bottom.
    if !app.error_message.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Error")
            .border_style(Style::default().fg(Color::Red));
        let paragraph = Paragraph::new(app.error_message.as_str())
            .block(block)
            .style(Style::default().fg(Color::Red));
        let rect = centered_rect(60, 3, area);
        frame.render_widget(paragraph, rect);
    }
}

/// Minimal banner with some ASCII art, using tui styles.
fn draw_banner<B: Backend>(frame: &mut Frame<B>, area: Rect) {
    let banner_text = r#"
 ___   ___   ___  _   _  _ __   ___   _ __    ___  | |_   ___  ___
/ __| / _ \ / __|| | | || '__| / _ \ | '_ \  / _ \ | __| / _ \/ __|
\__ \|  __/| (__ | |_| || |   |  __/ | | | || (_) || |_ |  __/\__ \
|___/ \___| \___| \__,_||_|    \___| |_| |_| \___/  \__| \___||___/
"#;

    let line1 = Line::from(Span::styled(
        "SECURE NOTES\n",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let line2 = Line::from(Span::raw(banner_text));

    let block = Block::default().borders(Borders::ALL).title(" Secure Notes ");
    let paragraph = Paragraph::new(vec![line1, line2]).block(block).style(
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    );

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(0)].as_ref())
        .split(area);

    frame.render_widget(paragraph, layout[0]);
}

/// Draw the password prompt screen.
fn draw_password_prompt<B: Backend>(frame: &mut Frame<B>, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Enter Master Password (Press ENTER to confirm, ESC to exit)");

    // In real usage, you might want to mask the input with '*'
    let paragraph = Paragraph::new(app.input_buffer.as_str())
        .block(block)
        .style(Style::default().fg(Color::Yellow));

    let rect = centered_rect(60, 3, area);
    frame.render_widget(paragraph, rect);
}

/// Draw the main menu.
fn draw_main_menu<B: Backend>(frame: &mut Frame<B>, _app: &App, area: Rect) {
    let items = vec![
        ListItem::new("1) View Notes"),
        ListItem::new("2) Create Note"),
        ListItem::new("3) Edit Note"),
        ListItem::new("4) Delete Note"),
        ListItem::new("5) Open Note"),
        ListItem::new("6) Delete ALL Notes"),
        ListItem::new("7) Exit"),
    ];
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Secure Notes Menu"),
        )
        .style(Style::default().fg(Color::Cyan));

    let rect = centered_rect(40, 15, area);
    frame.render_widget(list, rect);
}

/// Displays notes in truncated form.
fn draw_view_notes<B: Backend>(frame: &mut Frame<B>, app: &App, area: Rect) {
    if app.notes.is_empty() {
        let block = Block::default().borders(Borders::ALL).title("View Notes");
        let paragraph = Paragraph::new("No notes found.")
            .block(block)
            .style(Style::default().fg(Color::Yellow));
        let rect = centered_rect(60, 5, area);
        frame.render_widget(paragraph, rect);
        return;
    }

    let mut items = Vec::new();
    for note in &app.notes {
        let title_str = format!(
            "ID: {} | Title: {} | Content (truncated): {}",
            note.id,
            note.title,
            note.content.chars().take(30).collect::<String>()
        );
        items.push(ListItem::new(title_str));
    }
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("View Notes"))
        .style(Style::default().fg(Color::White));

    let rect = centered_rect(80, 20, area);
    frame.render_widget(list, rect);
}

/// Draws a note editor (for both create + edit).
fn draw_note_editor<B: Backend>(frame: &mut Frame<B>, app: &App, area: Rect) {
    let title = if app.screen == Screen::CreateNote {
        "Create Note (Esc=Save, F2=Discard)"
    } else {
        "Edit Note (Esc=Save, F2=Discard)"
    };

    let block = Block::default().borders(Borders::ALL).title(title);

    let paragraph = Paragraph::new(app.edit_state.buffer.as_str())
        .block(block)
        .style(Style::default().fg(Color::Green));

    let rect = centered_rect(60, 15, area);
    frame.render_widget(paragraph, rect);
}

/// Draws a simple input box (Delete note by ID, open note by ID, etc.).
fn draw_simple_input<B: Backend>(frame: &mut Frame<B>, app: &App, area: Rect) {
    let title = match app.screen {
        Screen::DeleteNote => "Enter note ID to delete (ENTER=confirm, ESC=cancel)",
        Screen::OpenNote => "Enter note ID to open (ENTER=confirm, ESC=cancel)",
        Screen::DeleteAll => "Are you sure? Type YES to confirm (ENTER=confirm, ESC=cancel)",
        _ => "",
    };

    let block = Block::default().borders(Borders::ALL).title(title);
    let paragraph = Paragraph::new(app.input_buffer.as_str())
        .block(block)
        .style(Style::default().fg(Color::Yellow));

    let rect = centered_rect(60, 3, area);
    frame.render_widget(paragraph, rect);
}

////////////////////////////////////////////////////////////////////////////////
// Input Handling
////////////////////////////////////////////////////////////////////////////////

fn handle_key_event(key_event: KeyEvent, app: &mut App) -> Result<()> {
    match app.screen {
        Screen::PasswordPrompt => match key_event.code {
            KeyCode::Enter => {
                // 1) Derive key
                app.password = app.input_buffer.clone();
                app.input_buffer.clear();
                app.key = derive_key_from_password(&app.password, SALT, PBKDF2_ITERATIONS)?;

                // 2) Try loading existing notes
                if let Ok(notes) = load_notes(&app.file_path, &app.key) {
                    app.notes = notes;
                }
                app.screen = Screen::Menu;
            }
            KeyCode::Char(c) => {
                // Normally you'd mask with '*'
                app.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Esc => {
                // If user presses Esc at password prompt, exit
                app.screen = Screen::Exit;
            }
            _ => {}
        },
        Screen::Menu => match key_event.code {
            KeyCode::Char('1') => app.screen = Screen::ViewNotes,
            KeyCode::Char('2') => {
                app.edit_state.buffer.clear();
                app.edit_state.note_id = None;
                app.screen = Screen::CreateNote;
            }
            KeyCode::Char('3') => {
                // Prompt for note ID first
                app.input_buffer.clear();
                app.edit_state.note_id = None;
                app.screen = Screen::EditNote;
            }
            KeyCode::Char('4') => {
                app.input_buffer.clear();
                app.screen = Screen::DeleteNote;
            }
            KeyCode::Char('5') => {
                app.input_buffer.clear();
                app.screen = Screen::OpenNote;
            }
            KeyCode::Char('6') => {
                app.input_buffer.clear();
                app.screen = Screen::DeleteAll;
            }
            KeyCode::Char('7') => {
                app.screen = Screen::Exit;
            }
            _ => {}
        },
        Screen::ViewNotes => {
            // On Enter/Esc, go back to main menu
            if matches!(key_event.code, KeyCode::Esc | KeyCode::Enter) {
                app.screen = Screen::Menu;
            }
        }
        Screen::CreateNote => match key_event.code {
            KeyCode::Esc => {
                // Save the new note
                let new_note = Note {
                    id: generate_user_friendly_id(),
                    title: "(Untitled)".to_string(),
                    content: app.edit_state.buffer.clone(),
                };
                app.notes.push(new_note);
                save_notes(&app.file_path, &app.notes, &app.key)?;
                app.screen = Screen::Menu;
            }
            KeyCode::F(n) if n == 2 => {
                // Discard
                app.screen = Screen::Menu;
            }
            KeyCode::Char(c) => {
                app.edit_state.buffer.push(c);
            }
            KeyCode::Backspace => {
                app.edit_state.buffer.pop();
            }
            _ => {}
        },
        Screen::EditNote => {
            if app.edit_state.note_id.is_none() {
                // Expecting a note ID
                match key_event.code {
                    KeyCode::Char(c) => {
                        app.input_buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input_buffer.pop();
                    }
                    KeyCode::Enter => {
                        let id = app.input_buffer.trim().to_string();
                        app.input_buffer.clear();
                        if let Some(note) = app.notes.iter().find(|n| n.id == id) {
                            app.edit_state.note_id = Some(note.id.clone());
                            app.edit_state.buffer = note.content.clone();
                        } else {
                            app.error_message = "Note ID not found.".to_string();
                        }
                    }
                    KeyCode::Esc => {
                        app.screen = Screen::Menu;
                    }
                    _ => {}
                }
            } else {
                // We are editing the note content
                match key_event.code {
                    KeyCode::Esc => {
                        // Save changes
                        if let Some(id) = &app.edit_state.note_id {
                            if let Some(n) = app.notes.iter_mut().find(|x| &x.id == id) {
                                n.content = app.edit_state.buffer.clone();
                            }
                            save_notes(&app.file_path, &app.notes, &app.key)?;
                        }
                        app.screen = Screen::Menu;
                    }
                    KeyCode::F(2) => {
                        // Discard changes
                        app.screen = Screen::Menu;
                    }
                    KeyCode::Char(c) => {
                        app.edit_state.buffer.push(c);
                    }
                    KeyCode::Backspace => {
                        app.edit_state.buffer.pop();
                    }
                    _ => {}
                }
            }
        }
        Screen::DeleteNote => match key_event.code {
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Enter => {
                let id = app.input_buffer.trim();
                let old_len = app.notes.len();
                app.notes.retain(|n| n.id != id);
                if app.notes.len() == old_len {
                    app.error_message = "No note found with that ID.".to_string();
                } else {
                    save_notes(&app.file_path, &app.notes, &app.key)?;
                }
                app.input_buffer.clear();
                app.screen = Screen::Menu;
            }
            KeyCode::Esc => {
                app.screen = Screen::Menu;
            }
            _ => {}
        },
        Screen::OpenNote => match key_event.code {
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Enter => {
                let id = app.input_buffer.trim();
                if let Some(n) = app.notes.iter().find(|x| x.id == id) {
                    // Show full note content in the error area (quick way to display it)
                    app.error_message = format!("Full Note: {}", n.content);
                } else {
                    app.error_message = "Note not found.".to_string();
                }
                app.input_buffer.clear();
                app.screen = Screen::Menu;
            }
            KeyCode::Esc => {
                app.screen = Screen::Menu;
            }
            _ => {}
        },
        Screen::DeleteAll => match key_event.code {
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Enter => {
                let confirm = app.input_buffer.trim();
                if confirm == "YES" {
                    app.notes.clear();
                    save_notes(&app.file_path, &app.notes, &app.key)?;
                } else {
                    app.error_message = "Canceled. Type YES to confirm next time.".to_string();
                }
                app.input_buffer.clear();
                app.screen = Screen::Menu;
            }
            KeyCode::Esc => {
                app.screen = Screen::Menu;
            }
            _ => {}
        },
        Screen::Exit => {}
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Layout Helpers
////////////////////////////////////////////////////////////////////////////////

fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - height) / 2),
                Constraint::Length(height),
                Constraint::Percentage((100 - height) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    let middle = popup_layout[1];
    let popup_width = middle.width * percent_x / 100;
    let margin_x = (middle.width.saturating_sub(popup_width)) / 2;

    Rect {
        x: middle.x + margin_x,
        y: middle.y,
        width: popup_width,
        height: middle.height,
    }
}

////////////////////////////////////////////////////////////////////////////////
// Encryption & Persistence
////////////////////////////////////////////////////////////////////////////////

fn derive_key_from_password(password: &str, salt: &[u8], iterations: u32) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        NonZeroU32::new(iterations).unwrap(),
        salt,
        password.as_bytes(),
        &mut key,
    );
    Ok(key)
}

fn encrypt_data(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    let sealing_key = aead::LessSafeKey::new(
        aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key)
            .map_err(|_| anyhow!("Failed to create encryption key"))?,
    );

    let rng = ring_rand::SystemRandom::new();
    let nonce_bytes = ring_rand::generate::<[u8; 12]>(&rng)
        .map_err(|_| anyhow!("Failed to generate nonce"))?
        .expose();
    let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

    let mut in_out = plaintext.to_vec();
    in_out.resize(in_out.len() + sealing_key.algorithm().tag_len(), 0);

    sealing_key
        .seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|_| anyhow!("Encryption failed"))?;

    let mut result = Vec::with_capacity(12 + in_out.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&in_out);
    Ok(result)
}

fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.len() < 12 {
        return Err(anyhow!("Ciphertext too short"));
    }
    let (nonce_bytes, encrypted) = ciphertext.split_at(12);
    let nonce =
        aead::Nonce::try_assume_unique_for_key(nonce_bytes).map_err(|_| anyhow!("Invalid nonce"))?;

    let opening_key = aead::LessSafeKey::new(
        aead::UnboundKey::new(&aead::CHACHA20_POLY1305, key)
            .map_err(|_| anyhow!("Failed to create decryption key"))?,
    );

    let mut in_out = encrypted.to_vec();
    let decrypted_data = opening_key
        .open_in_place(nonce, aead::Aad::empty(), &mut in_out)
        .map_err(|_| anyhow!("Decryption failed"))?;
    Ok(decrypted_data.to_vec())
}

fn load_notes<P: AsRef<Path>>(path: P, key: &[u8]) -> Result<Vec<Note>> {
    if !path.as_ref().exists() {
        // Not necessarily an error; just return empty list
        return Ok(Vec::new());
    }
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut ciphertext = Vec::new();
    file.read_to_end(&mut ciphertext)?;
    let decrypted_bytes = decrypt_data(&ciphertext, key)?;
    let notes: Vec<Note> = serde_json::from_slice(&decrypted_bytes)?;
    Ok(notes)
}

fn save_notes<P: AsRef<Path>>(path: P, notes: &[Note], key: &[u8]) -> Result<()> {
    let json_data = serde_json::to_vec(notes)?;
    let ciphertext = encrypt_data(&json_data, key)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    file.write_all(&ciphertext)?;
    file.flush()?;
    Ok(())
}
