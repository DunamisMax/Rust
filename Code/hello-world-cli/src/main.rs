////////////////////////////////////////////////////////////////////////////////
// Imports
////////////////////////////////////////////////////////////////////////////////

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::{seq::SliceRandom, Rng};
use std::{io, time::Duration};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
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
#[command(author, version, about = "Multilingual Hello-World TUI", long_about = None)]
struct CliArgs {
    /// Optional flag for demonstration
    #[arg(long, short, help = "Enable verbose mode")]
    verbose: bool,
}

////////////////////////////////////////////////////////////////////////////////
// Application State
////////////////////////////////////////////////////////////////////////////////

struct App {
    /// Current text input for "name"
    input: String,
    /// Current greeting displayed
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
// Main (Tokio) Entry Point
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<()> {
    // 1) Parse CLI arguments
    let args = CliArgs::parse();
    if args.verbose {
        print!("Verbose mode enabled...{}", LINE_ENDING);
    }

    // 2) Set up terminal in raw mode + alternate screen for TUI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3) Run our TUI-driven event loop
    let res = run_app(&mut terminal);

    // 4) Restore terminal state
    disable_raw_mode()?;

    // Drop the Terminal so we can safely get back to the raw stdout
    let mut stdout = terminal.into_inner();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;

    // Return any errors
    if let Err(err) = res {
        eprintln!("Error: {err}");
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// TUI Event Loop
////////////////////////////////////////////////////////////////////////////////

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();

    // We’ll keep polling user keystrokes until Esc or Ctrl+C is pressed.
    loop {
        // 1) Draw the current state of the UI
        terminal.draw(|frame| {
            draw_ui(frame, &app);
        })?;

        // 2) Non-blocking poll for events. We'll wait ~100ms to reduce CPU usage
        if crossterm::event::poll(Duration::from_millis(100))? {
            // If there's an event, read it
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    //-------------------------------------------
                    // Typing characters (with no modifiers)
                    //-------------------------------------------
                    KeyCode::Char(c) if key_event.modifiers.is_empty() => {
                        app.input.push(c);
                    }

                    //-------------------------------------------
                    // Ctrl+C => exit
                    //-------------------------------------------
                    KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        break;
                    }

                    //-------------------------------------------
                    // Backspace
                    //-------------------------------------------
                    KeyCode::Backspace => {
                        app.input.pop();
                    }

                    //-------------------------------------------
                    // Enter => generate a new greeting
                    //-------------------------------------------
                    KeyCode::Enter => {
                        if app.input.trim().is_empty() {
                            app.greeting = pick_random_greeting("World");
                        } else {
                            app.greeting = pick_random_greeting(app.input.trim());
                        }
                        // Clear the input for a new name
                        app.input.clear();
                    }

                    //-------------------------------------------
                    // ESC => exit
                    //-------------------------------------------
                    KeyCode::Esc => {
                        break;
                    }

                    //-------------------------------------------
                    // Ignore other keys
                    //-------------------------------------------
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// UI Drawing
////////////////////////////////////////////////////////////////////////////////

fn draw_ui(frame: &mut tui::Frame<CrosstermBackend<io::Stdout>>, app: &App) {
    let screen = frame.size();

    // A single "centered window" block with a border
    let main_block = Block::default()
        .title(" Multilingual Greeter ")
        .borders(Borders::ALL);
    frame.render_widget(main_block, screen);

    // Carve out a smaller inner rect
    let inner = inner_rect(screen);

    // Inside that block, we split into 3 vertical sections:
    //   1) instructions
    //   2) input
    //   3) greeting
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // instructions
            Constraint::Length(3), // input
            Constraint::Length(3), // greeting
        ])
        .split(inner);

    // 1) Instructions
    let instructions = "Type a name & press Enter. Press Esc or Ctrl+C to exit.";
    let instructions_para = Paragraph::new(instructions)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(instructions_para, chunks[0]);

    // 2) Current input
    let input_label = format!("Name: {}", app.input);
    let input_para = Paragraph::new(input_label)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(input_para, chunks[1]);

    // 3) Greeting
    //   Convert &app.greeting to a string slice
    let greeting_para = Paragraph::new(app.greeting.as_str())
        .alignment(Alignment::Center)
        .style(Style::default().fg(random_tui_color()));
    frame.render_widget(greeting_para, chunks[2]);
}

/// Helper: returns an "inner rectangle" to avoid text overlapping the border.
fn inner_rect(area: Rect) -> Rect {
    Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
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
// Random TUI Color
////////////////////////////////////////////////////////////////////////////////

fn random_tui_color() -> Color {
    use Color::*;
    let colors = [Red, Green, Yellow, Blue, Magenta, Cyan, White, Gray];
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..colors.len());
    colors[idx]
}
