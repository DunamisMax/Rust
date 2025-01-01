////////////////////////////////////////////////////////////////////////////////
// secure-notes - Encrypted Notes Manager with Ratatui + Tokio + Crossterm
////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
// Imports
////////////////////////////////////////////////////////////////////////////////

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use ring::{aead, pbkdf2, rand as ring_rand};
use serde::{Deserialize, Serialize};

use std::{
    fs::OpenOptions,
    io::{self, Read, Write},
    num::NonZeroU32,
    path::Path,
    time::Duration,
};

use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};

// IMPORTANT: In ratatui 0.29, we just import `Frame` (with no generic param)
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use rand::Rng;

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
#[command(author, version, about = "Secure Notes CLI", long_about = None)]
struct CliArgs {
    /// Optional path to the encrypted notes file
    #[arg(long, short, default_value = "secure_notes.json.enc")]
    file: String,
}

////////////////////////////////////////////////////////////////////////////////
// SALT & PBKDF2 CONFIG (Demo Purposes Only)
// In production, use a unique salt & high iteration count per user.
////////////////////////////////////////////////////////////////////////////////

const SALT: &[u8] = b"fixed-salt-demo";
const PBKDF2_ITERATIONS: u32 = 100_000;

////////////////////////////////////////////////////////////////////////////////
// Data Structures
////////////////////////////////////////////////////////////////////////////////

/// Generates a user-friendly 6-digit numeric ID.
fn generate_user_friendly_id() -> String {
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

/// The different TUI screens.
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

/// For note editing, track which note ID we’re editing and the text buffer.
#[derive(Debug, Clone)]
struct EditState {
    note_id: Option<String>,
    buffer: String,
}

/// Main TUI App State.
struct App {
    password: String,      // Master password
    key: [u8; 32],         // Derived encryption key
    notes: Vec<Note>,      // All notes
    screen: Screen,        // Current screen
    input_buffer: String,  // Generic input buffer (prompt usage, etc.)
    edit_state: EditState, // For note creation & editing
    error_message: String, // Displayable error message
    file_path: String,     // The file path where notes are stored
}

////////////////////////////////////////////////////////////////////////////////
// RAII guard for raw mode. Ensures raw mode is disabled even on panic.
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
// Main (Tokio) Entry Point
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<()> {
    // 1) Parse CLI arguments
    let args = CliArgs::parse();

    // 2) Enable raw mode via our RAII guard
    let _raw_guard = RawModeGuard::new().context("Failed to enable raw mode")?;

    // 3) Create the Ratatui Terminal
    let mut terminal = setup_terminal().context("Failed to create terminal")?;

    // 4) Clear the screen
    clear_screen(&mut terminal).context("Failed to clear terminal")?;

    // 5) Print a quick message with cross-platform line ending
    print!("   CLI started successfully...{}", LINE_ENDING);

    // 6) Build initial App state
    let app = App {
        password: String::new(),
        key: [0u8; 32],
        notes: Vec::new(),
        screen: Screen::PasswordPrompt,
        input_buffer: String::new(),
        edit_state: EditState {
            note_id: None,
            buffer: String::new(),
        },
        error_message: String::new(),
        file_path: args.file,
    };

    // 7) Launch the main TUI loop
    if let Err(e) = run_app(&mut terminal, app) {
        // If the app errored, restore terminal and show the error
        finalize_terminal(&mut terminal)?;
        eprintln!("Error: {e}");
        return Err(e);
    }

    // 8) If everything is OK, finalize
    finalize_terminal(&mut terminal)?;
    print!("Goodbye!{}", LINE_ENDING);

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Setup Terminal
////////////////////////////////////////////////////////////////////////////////

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Clears the terminal screen
////////////////////////////////////////////////////////////////////////////////

fn clear_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    terminal.clear()?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Finalize Terminal (disable raw mode, clear screen, move cursor)
////////////////////////////////////////////////////////////////////////////////

fn finalize_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    // The RawModeGuard will automatically disable raw mode upon drop.
    // Manually clear the screen + move the cursor home:
    execute!(terminal.backend_mut(), Clear(ClearType::All), MoveTo(0, 0))?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Main TUI Loop
////////////////////////////////////////////////////////////////////////////////

fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, mut app: App) -> Result<()> {
    loop {
        // 1) Draw current TUI
        terminal.draw(|frame| draw_ui(frame, &app))?;

        // 2) Check for input events with a short timeout
        if crossterm::event::poll(Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(key_event) => {
                    handle_key_event(key_event, &mut app)?;
                }
                Event::Mouse(_) => {
                    // Not handling mouse input in this example
                }
                _ => {}
            }
        }

        // 3) If user is ready to exit, break out
        if app.screen == Screen::Exit {
            break;
        }
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// TUI Rendering
////////////////////////////////////////////////////////////////////////////////

/// The top-level function that draws each screen.
/// Notice there's **no** `<B: Backend>` or `Frame<'_, B>`—just `Frame`.
fn draw_ui(frame: &mut Frame, app: &App) {
    let size = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // For the ASCII banner
            Constraint::Min(0),    // The main screen
            Constraint::Length(3), // For an optional error/footer
        ])
        .split(size);

    // 1) Banner (always displayed)
    draw_banner(frame, chunks[0]);

    // 2) Main screen
    match app.screen {
        Screen::PasswordPrompt => draw_password_prompt(frame, app, chunks[1]),
        Screen::Menu => draw_main_menu(frame, chunks[1]),
        Screen::ViewNotes => draw_view_notes(frame, app, chunks[1]),
        Screen::CreateNote | Screen::EditNote => draw_note_editor(frame, app, chunks[1]),
        Screen::DeleteNote | Screen::OpenNote | Screen::DeleteAll => {
            draw_simple_input(frame, app, chunks[1])
        }
        Screen::Exit => {
            // Nothing
        }
    }

    // 3) Error message or status line
    if !app.error_message.is_empty() {
        let block = Block::default()
            .title("Error")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red));
        let paragraph = Paragraph::new(Text::raw(&app.error_message))
            .block(block)
            .style(Style::default().fg(Color::Red));
        frame.render_widget(paragraph, chunks[2]);
    }
}

/// Minimal banner at the top.
fn draw_banner(frame: &mut Frame, area: Rect) {
    // We’ll do a two-line banner:
    //  1) A colorful “SECURE NOTES”
    //  2) A short tagline or divider

    // First line: "SECURE NOTES" in bright magenta, bold
    let line1 = Line::from(Span::styled(
        " SECURE NOTES ",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));

    // Second line: could be a simple divider, subtitle, or tagline
    let line2 = Line::from(Span::styled(
        " An Encrypted TUI App ",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::ITALIC),
    ));

    // Combine lines in a Paragraph
    let paragraph = Paragraph::new(vec![line1, line2])
        .block(Block::default().borders(Borders::ALL).title(" Welcome "))
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn draw_password_prompt(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title("Enter Master Password (ENTER=confirm, ESC=exit)")
        .borders(Borders::ALL);
    // For real usage, you might mask the password with '*'
    let paragraph = Paragraph::new(app.input_buffer.as_str())
        .block(block)
        .style(Style::default().fg(Color::Yellow));

    frame.render_widget(paragraph, area);
}

fn draw_main_menu(frame: &mut Frame, area: Rect) {
    let options = vec![
        "1) View Notes",
        "2) Create Note",
        "3) Edit Note",
        "4) Delete Note",
        "5) Open Note",
        "6) Delete ALL Notes",
        "7) Exit",
    ];
    let items: Vec<ListItem> = options
        .into_iter()
        .map(|opt| ListItem::new(Span::raw(opt)))
        .collect();

    let block = Block::default()
        .title(" Secure Notes Menu ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(list, area);
}

fn draw_view_notes(frame: &mut Frame, app: &App, area: Rect) {
    if app.notes.is_empty() {
        let block = Block::default()
            .title("View Notes")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));
        let paragraph = Paragraph::new("No notes found.")
            .block(block)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    }

    let mut items = Vec::new();
    for note in &app.notes {
        let text = format!(
            "ID: {} | Title: {} | Content (truncated): {}",
            note.id,
            note.title,
            note.content.chars().take(30).collect::<String>()
        );
        items.push(ListItem::new(Span::raw(text)));
    }

    let block = Block::default().title("View Notes").borders(Borders::ALL);
    let list = List::new(items)
        .block(block)
        .style(Style::default().fg(Color::White));

    frame.render_widget(list, area);
}

fn draw_note_editor(frame: &mut Frame, app: &App, area: Rect) {
    let title = if app.screen == Screen::CreateNote {
        "Create Note (Esc=save, F2=discard)"
    } else {
        "Edit Note (Esc=save, F2=discard)"
    };
    let block = Block::default().title(title).borders(Borders::ALL);
    let paragraph = Paragraph::new(app.edit_state.buffer.as_str())
        .block(block)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::Green));

    frame.render_widget(paragraph, area);
}

fn draw_simple_input(frame: &mut Frame, app: &App, area: Rect) {
    let title = match app.screen {
        Screen::DeleteNote => "Enter Note ID to delete (ENTER=confirm, ESC=cancel)",
        Screen::OpenNote => "Enter Note ID to open (ENTER=confirm, ESC=cancel)",
        Screen::DeleteAll => "Type YES to confirm (ENTER=confirm, ESC=cancel)",
        _ => "",
    };

    let block = Block::default().title(title).borders(Borders::ALL);
    let paragraph = Paragraph::new(app.input_buffer.as_str())
        .block(block)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, area);
}

////////////////////////////////////////////////////////////////////////////////
// Input Handling
////////////////////////////////////////////////////////////////////////////////

fn handle_key_event(key_event: KeyEvent, app: &mut App) -> Result<()> {
    match app.screen {
        Screen::PasswordPrompt => match key_event.code {
            KeyCode::Enter => {
                // Derive key
                app.password = app.input_buffer.clone();
                app.input_buffer.clear();
                app.key = derive_key_from_password(&app.password, SALT, PBKDF2_ITERATIONS)?;

                // Try loading notes
                if let Ok(notes) = load_notes(&app.file_path, &app.key) {
                    app.notes = notes;
                }
                app.screen = Screen::Menu;
            }
            KeyCode::Esc => {
                // Exit
                app.screen = Screen::Exit;
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                // For real usage, consider masking with '*'
                app.input_buffer.push(c);
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
                // Prompt for note ID
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
            KeyCode::Char('7') => app.screen = Screen::Exit,
            _ => {}
        },
        Screen::ViewNotes => {
            // On Enter/Esc, go back to menu
            if matches!(key_event.code, KeyCode::Enter | KeyCode::Esc) {
                app.screen = Screen::Menu;
            }
        }
        Screen::CreateNote => match key_event.code {
            KeyCode::Esc => {
                // Save
                let new_note = Note {
                    id: generate_user_friendly_id(),
                    title: "(Untitled)".to_string(),
                    content: app.edit_state.buffer.clone(),
                };
                app.notes.push(new_note);
                save_notes(&app.file_path, &app.notes, &app.key)?;
                app.screen = Screen::Menu;
            }
            KeyCode::F(2) => {
                // Discard
                app.screen = Screen::Menu;
            }
            KeyCode::Backspace => {
                app.edit_state.buffer.pop();
            }
            KeyCode::Char(c) => {
                app.edit_state.buffer.push(c);
            }
            _ => {}
        },
        Screen::EditNote => {
            // If we don't yet have a note_id, we're prompting for it
            if app.edit_state.note_id.is_none() {
                match key_event.code {
                    KeyCode::Enter => {
                        let id = app.input_buffer.trim().to_string();
                        app.input_buffer.clear();
                        if let Some(note) = app.notes.iter().find(|n| n.id == id) {
                            app.edit_state.note_id = Some(note.id.clone());
                            app.edit_state.buffer = note.content.clone();
                        } else {
                            app.error_message = format!("Note ID {id} not found.");
                        }
                    }
                    KeyCode::Esc => {
                        app.screen = Screen::Menu;
                    }
                    KeyCode::Backspace => {
                        app.input_buffer.pop();
                    }
                    KeyCode::Char(c) => {
                        app.input_buffer.push(c);
                    }
                    _ => {}
                }
            } else {
                // We have note_id, so we're editing the content
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
                    KeyCode::Backspace => {
                        app.edit_state.buffer.pop();
                    }
                    KeyCode::Char(c) => {
                        app.edit_state.buffer.push(c);
                    }
                    _ => {}
                }
            }
        }
        Screen::DeleteNote => match key_event.code {
            KeyCode::Enter => {
                let id = app.input_buffer.trim();
                let old_len = app.notes.len();
                app.notes.retain(|n| n.id != id);
                if app.notes.len() == old_len {
                    app.error_message = format!("No note found with ID {id}.");
                } else {
                    save_notes(&app.file_path, &app.notes, &app.key)?;
                }
                app.input_buffer.clear();
                app.screen = Screen::Menu;
            }
            KeyCode::Esc => {
                app.screen = Screen::Menu;
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
            }
            _ => {}
        },
        Screen::OpenNote => match key_event.code {
            KeyCode::Enter => {
                let id = app.input_buffer.trim();
                if let Some(n) = app.notes.iter().find(|x| x.id == id) {
                    // Place the full content in error_message as a quick display
                    app.error_message = format!("Full Note: {}", n.content);
                } else {
                    app.error_message = format!("No note found with ID {id}.");
                }
                app.input_buffer.clear();
                app.screen = Screen::Menu;
            }
            KeyCode::Esc => {
                app.screen = Screen::Menu;
            }
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
            }
            _ => {}
        },
        Screen::DeleteAll => match key_event.code {
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
            KeyCode::Backspace => {
                app.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                app.input_buffer.push(c);
            }
            _ => {}
        },
        Screen::Exit => {}
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Encryption + Persistence
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
    let nonce = aead::Nonce::try_assume_unique_for_key(nonce_bytes)
        .map_err(|_| anyhow!("Invalid nonce"))?;

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
        return Ok(Vec::new()); // No file yet -> empty list
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
