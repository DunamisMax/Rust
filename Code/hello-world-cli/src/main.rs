////////////////////////////////////////////////////////////////////////////////
// main.rs - A Multilingual TUI w/ Ratatui banner
////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
// Imports
////////////////////////////////////////////////////////////////////////////////

use anyhow::{Context, Result};
use clap::Parser;
use std::io::{self, Write};
use std::time::Duration;

use crossterm::{
    cursor::MoveTo,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

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
// RAII Guard for Raw Mode
////////////////////////////////////////////////////////////////////////////////

/// A simple guard that enables raw mode on creation and disables it on drop.
/// This ensures raw mode is properly cleaned up even if an error occurs.
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
// App Data Model
////////////////////////////////////////////////////////////////////////////////

/// Tracks the current user input (for name) and the generated greeting.
struct App {
    input: String,
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
// Main (Tokio) Entry
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<()> {
    // 1) Parse CLI arguments
    let args = CliArgs::parse();
    if args.verbose {
        print!("Verbose mode enabled...{}", LINE_ENDING);
    }

    // 2) Enable raw mode (via our guard)
    let _raw_guard = RawModeGuard::new().context("Failed to enable raw mode")?;

    // 3) Switch to an alternate screen, enable mouse capture, and clear
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        Clear(ClearType::All),
        MoveTo(0, 0)
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // 4) Draw an initial welcome TUI (Ratátui-style banner)
    draw_welcome_screen(&mut terminal).context("Failed to draw welcome screen")?;

    // 5) Run the TUI-driven event loop (user types a name and sees a random greeting)
    run_app(&mut terminal).context("Error in TUI event loop")?;

    // 6) Drop the Terminal to free resources, then restore normal terminal state
    drop(terminal);
    drop(_raw_guard); // This also disables raw mode

    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;

    // 7) Print a friendly exit message on the standard buffer
    println!("{}", LINE_ENDING); // Extra blank line
    println!("Goodbye!{}", LINE_ENDING);

    // Pause so the user can see the message
    print!("Press Enter to exit...{}", LINE_ENDING);
    io::stdout().flush()?;
    let mut exit_buf = String::new();
    io::stdin().read_line(&mut exit_buf)?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// TUI Event Loop
////////////////////////////////////////////////////////////////////////////////

/// Runs the main TUI loop until the user presses Esc or Ctrl+C.
/// The user can type a name and press Enter to generate a random greeting.
fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();

    loop {
        // 1) Draw the UI with the current state
        terminal.draw(|frame| {
            draw_main_ui(frame, &app);
        })?;

        // 2) Poll for key events (~100ms)
        if event::poll(Duration::from_millis(100))? {
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
                    // Enter => generate random greeting
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

fn draw_welcome_screen(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    terminal.draw(|frame| {
        let screen = frame.area();

        // 1) Split into a small banner area and the rest
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
            .split(screen);

        // 2) Draw top banner
        draw_banner(frame, chunks[0]);

        // 3) In the main area, a centered paragraph with basic instructions
        let main_area = centered_rect(60, 50, chunks[1]);
        let welcome_para = Paragraph::new(vec![
            Line::from(Span::styled(
                "Welcome to the multilingual hello-world-cli!",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Type your name once you enter the main app, then press Enter to see a random greeting in many languages!",
                Style::default().fg(Color::Green),
            )),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Get Ready! ")
                .border_style(Style::default().fg(Color::Magenta)),
        );

        frame.render_widget(welcome_para, main_area);
    })?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Draw the Banner
////////////////////////////////////////////////////////////////////////////////

/// Notice that `Frame` no longer takes a backend type parameter in 0.29+.
fn draw_banner(frame: &mut Frame, area: Rect) {
    let line1 = Line::from(Span::styled(
        "HELLO-WORLD-CLI",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let line2 = Line::from("A minimal TUI demonstration in multiple languages");

    let paragraph = Paragraph::new(vec![line1, line2])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Welcome ")
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

////////////////////////////////////////////////////////////////////////////////
// Draw the Main UI
////////////////////////////////////////////////////////////////////////////////

/// Likewise, `draw_main_ui` takes `&mut Frame` rather than `&mut Frame<B>`.
fn draw_main_ui(frame: &mut Frame, app: &App) {
    let screen = frame.area();

    // Outer border
    let main_block = Block::default()
        .title(" Multilingual Greeter ")
        .borders(Borders::ALL);
    frame.render_widget(main_block, screen);

    // Inner layout
    let inner = centered_rect(80, 60, screen);

    // Vertical chunks: instructions, input, greeting
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
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(instructions, chunks[0]);

    // 2) Current input
    let input_text = format!("Name: {}", app.input);
    let input_para = Paragraph::new(input_text)
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Left);
    frame.render_widget(input_para, chunks[1]);

    // 3) Greeting (with random color)
    let greeting_para = Paragraph::new(app.greeting.as_str())
        .alignment(Alignment::Center)
        .style(Style::default().fg(random_ratatui_color()));
    frame.render_widget(greeting_para, chunks[2]);
}

////////////////////////////////////////////////////////////////////////////////
// Helper: Center a smaller box within a given area
////////////////////////////////////////////////////////////////////////////////

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    let middle = layout[1];
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

fn random_ratatui_color() -> Color {
    use Color::*;
    let colors = [Red, Green, Yellow, Blue, Magenta, Cyan, White, Gray];
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..colors.len());
    colors[idx]
}
