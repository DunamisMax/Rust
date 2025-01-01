////////////////////////////////////////////////////////////////////////////////
// hello-world-cli - A Multilingual TUI w/ random greetings & ASCII art banner
////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
// Imports
////////////////////////////////////////////////////////////////////////////////

use anyhow::{Context, Result};
use clap::Parser;
use std::io::{self};
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use figlet_rs::FIGfont;
use rand::{seq::SliceRandom, Rng};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
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
#[command(author, version, about = "Multilingual Hello-World Ratatui App", long_about = None)]
struct CliArgs {
    /// Optional flag for demonstration
    #[arg(long, short, help = "Enable verbose mode")]
    verbose: bool,
}

////////////////////////////////////////////////////////////////////////////////
// App State
////////////////////////////////////////////////////////////////////////////////

/// Tracks the current user input and the generated greeting.
struct App {
    /// The text the user has typed (for the name).
    input: String,
    /// The current greeting displayed.
    greeting: String,
}

impl App {
    fn new() -> Self {
        Self {
            input: String::new(),
            greeting: String::new(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// RAII Guard for Raw Mode
////////////////////////////////////////////////////////////////////////////////

/// A simple guard that enables raw mode on creation, and disables it on drop.
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
// Main (Tokio) Entry
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<()> {
    // 1) Parse CLI arguments
    let args = CliArgs::parse();
    if args.verbose {
        print!("Verbose mode enabled...{}", LINE_ENDING);
    }

    // 2) Enable raw mode
    let _raw_guard = RawModeGuard::new().context("Failed to enable raw mode")?;

    // 3) Switch to an alternate screen buffer, enable mouse capture, and clear
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        Clear(ClearType::All)
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // 4) Draw an initial welcome TUI (with a FIGlet ASCII banner)
    draw_welcome_screen(&mut terminal)?;

    // 5) Run the TUI-driven event loop (user can type a name and see greetings)
    run_app(&mut terminal).context("Error in TUI event loop")?;

    // 6) Restore terminal state:
    //    - Drop the `Terminal` to release its resources.
    //    - Switch back from the alternate screen, disable mouse capture.
    drop(terminal); // drop TUI
    drop(_raw_guard); // drop raw mode

    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;

    // 7) Print a friendly exit message on the standard buffer
    print!("Goodbye!{}", LINE_ENDING);

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// TUI App Loop
////////////////////////////////////////////////////////////////////////////////

/// Runs the main TUI event loop. The user can type in a name and press Enter,
/// receiving a random greeting. Press Esc or Ctrl+C to exit.
fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();

    loop {
        // 1) Draw the UI with the current state
        terminal.draw(|frame| {
            draw_main_ui(frame, &app);
        })?;

        // 2) Check for key events (poll ~100ms)
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    // Typing characters
                    KeyCode::Char(c) if key_event.modifiers.is_empty() => {
                        app.input.push(c);
                    }
                    // Backspace
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    // Enter => generate a random greeting
                    KeyCode::Enter => {
                        let name = if app.input.trim().is_empty() {
                            "World"
                        } else {
                            app.input.trim()
                        };
                        app.greeting = pick_random_greeting(name);
                        app.input.clear();
                    }
                    // Esc or Ctrl+C => exit
                    KeyCode::Esc | KeyCode::Char('c')
                        if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Draw the Welcome Screen
////////////////////////////////////////////////////////////////////////////////

/// Draws an initial “Welcome” TUI with a FIGlet-based ASCII art banner.
fn draw_welcome_screen(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    terminal.draw(|frame| {
        let size = frame.area();

        // Draw a simple block as background
        let block = Block::default()
            .title(" Welcome to the Multilingual Ratatui CLI! ")
            .borders(Borders::ALL);
        frame.render_widget(block, size);

        // Create a FIGlet ASCII banner
        let fig_font = FIGfont::standard().expect("Failed to load standard FIGfont");
        let figure = fig_font
            .convert("Hello, Ratatui!")
            .expect("Failed to render FIGlet text");
        let banner_str = figure.to_string();

        // We'll place the ASCII banner text in the center
        let text_area = centered_rect(80, 60, size);
        let paragraph = Paragraph::new(
            banner_str
                .lines()
                .map(|line| {
                    Line::from(Span::styled(
                        line.to_string(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ))
                })
                .collect::<Vec<_>>(),
        )
        .alignment(Alignment::Center);

        frame.render_widget(paragraph, text_area);
    })?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Draw the Main UI
////////////////////////////////////////////////////////////////////////////////

/// Draws the core TUI: instructions, current input, and random greeting display.
/// Note: Must specify `'_` lifetime for `Frame`.
fn draw_main_ui(frame: &mut Frame<'_, CrosstermBackend<io::Stdout>>, app: &App) {
    let screen = frame.area();

    // Outer border
    let main_block = Block::default()
        .title(" Multilingual Greeter ")
        .borders(Borders::ALL);
    frame.render_widget(main_block, screen);

    // Inside, we define an inner region
    let inner = centered_rect(80, 60, screen);

    // Create vertical chunks: instructions, input, greeting
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // instructions
            Constraint::Length(3), // typed input
            Constraint::Length(3), // greeting
        ])
        .split(inner);

    // 1) Instructions
    let instructions = Paragraph::new("Type a name & press Enter. Press Esc or Ctrl+C to exit.")
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center);
    frame.render_widget(instructions, chunks[0]);

    // 2) Current input
    let input_label = format!("Name: {}", app.input);
    let input_para = Paragraph::new(input_label)
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Left);
    frame.render_widget(input_para, chunks[1]);

    // 3) Greeting (with random color)
    let greeting_para = Paragraph::new(app.greeting.as_str())
        .style(Style::default().fg(random_ratatui_color()))
        .alignment(Alignment::Center);
    frame.render_widget(greeting_para, chunks[2]);
}

////////////////////////////////////////////////////////////////////////////////
// Helper: Centered Rect
////////////////////////////////////////////////////////////////////////////////

/// Returns a sub-rectangle of `area`, with the given `percent_x` and `percent_y`
/// sized portion centered in the parent rectangle.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    // Split vertically: top blank space, center chunk, bottom blank space
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(area);

    // The middle region
    let middle = layout[1];
    // Compute the actual width for x-centering
    let box_width = middle.width * percent_x / 100;
    let x_offset = middle.x + (middle.width.saturating_sub(box_width)) / 2;

    Rect {
        x: x_offset,
        y: middle.y,
        width: box_width,
        height: middle.height,
    }
}

////////////////////////////////////////////////////////////////////////////////
// Greeting Logic
////////////////////////////////////////////////////////////////////////////////

/// Picks a random greeting from a large set of languages, and returns
/// a formatted greeting with the user’s `name`.
fn pick_random_greeting(name: &str) -> String {
    let greetings = [
        "Mandarin Chinese: 你好 (Nǐ hǎo)",
        "Spanish: Hola",
        "English: Hello",
        "Hindi: नमस्ते (Namaste)",
        "Arabic: مرحبا (Marḥaban)",
        "Bengali: নমস্কার (Nomoshkar)",
        "Portuguese: Olá",
        "Russian: Привет (Privet)",
        "Japanese: こんにちは (Konnichiwa)",
        "Punjabi: ਸਤ ਸ੍ਰੀ ਅਕਾਲ (Sat Srī Akāl)",
        "German: Hallo",
        "Malay: Hai",
        "Telugu: నమస్కారం (Namaskāraṁ)",
        "Vietnamese: Xin chào",
        "Korean: 안녕하세요 (Annyeonghaseyo)",
        "French: Bonjour",
        "Tamil: வணக்கம் (Vaṇakkam)",
        "Marathi: नमस्कार (Namaskār)",
        "Urdu: اسلام علیکم (As-salāmu ʿalaykum)",
        "Turkish: Merhaba",
        "Italian: Ciao",
        "Thai: สวัสดี (S̄wạs̄dī)",
        "Gujarati: નમસ્તે (Namaste)",
        "Persian (Farsi): سلام (Salām)",
        "Polish: Cześć",
        "Pashto: السلام علیکم (As-salāmu ʿalaykum)",
        "Kannada: ನಮಸ್ಕಾರ (Namaskāra)",
        "Ukrainian: Привіт (Pryvit)",
        "Swahili: Jambo",
        "Zulu: Sawubona",
        "Greek: Γεια σου (Geia sou)",
        "Dutch: Hallo",
        "Tagalog: Kamusta",
        "Hungarian: Szia",
        "Czech: Ahoj",
        "Romanian: Bună",
        "Bulgarian: Здравей (Zdravey)",
        "Catalan: Hola",
        "Finnish: Hei",
        "Norwegian: Hei",
        "Swedish: Hej",
        "Danish: Hej",
        "Slovak: Ahoj",
        "Malayalam: നമസ്കാരം (Namaskāram)",
        "Burmese: မင်္ဂလာပါ (Mingalaba)",
        "Georgian: გამარჯობა (Gamarjoba)",
        "Bosnian: Zdravo",
        "Croatian: Bok",
        "Serbian: Zdravo",
        "Slovene: Živijo",
        "Indonesian: Halo",
        "Afrikaans: Hallo",
    ];

    let mut rng = rand::thread_rng();
    let greeting = greetings.choose(&mut rng).unwrap_or(&"English: Hello");
    format!("{} — {}!", greeting, name)
}

////////////////////////////////////////////////////////////////////////////////
// Random Ratatui Color
////////////////////////////////////////////////////////////////////////////////

/// Selects a random `ratatui::style::Color` for the greeting text.
fn random_ratatui_color() -> Color {
    use Color::*;
    let colors = [Red, Green, Yellow, Blue, Magenta, Cyan, White, Gray];
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..colors.len());
    colors[idx]
}
