////////////////////////////////////////////////////////////////////////////////
// Imports
////////////////////////////////////////////////////////////////////////////////

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rand::{seq::SliceRandom, Rng};
use std::io::{self, Write};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
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
// CLI Arguments (Example)
////////////////////////////////////////////////////////////////////////////////

#[derive(Parser, Debug)]
#[command(author, version, about = "Multilingual Hello-World TUI", long_about = None)]
struct CliArgs {
    /// An optional flag to demonstrate Clap usage
    #[arg(long, short, help = "Enable verbose mode")]
    verbose: bool,
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

    // 2) Enable raw mode for TUI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3) Clear the screen & display welcome banner via TUI
    clear_screen(&mut terminal)?;
    print_welcome_banner(&mut terminal)?;

    // 4) Temporarily disable raw mode to gather user input (name)
    disable_raw_mode()?;
    let name = prompt_for_name()?;
    enable_raw_mode()?;

    // 5) Greet the user (random language) in TUI
    greet_in_tui(&mut terminal, &name)?;

    // 6) Cleanly exit
    disable_raw_mode()?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// TUI "Clear Screen" and "Welcome Banner"
////////////////////////////////////////////////////////////////////////////////

fn clear_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    // TUI-based clearing
    terminal.clear()?;
    Ok(())
}

fn print_welcome_banner(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    // Example ASCII banner
    let banner = r#"
 _            _  _                                 _      _
| |          | || |                               | |    | |
| |__    ___ | || |  ___   __      __  ___   _ __ | |  __| |
| '_ \  / _ \| || | / _ \  \ \ /\ / / / _ \ | '__|| | / _` |
| | | ||  __/| || || (_) |  \ V  V / | (_) || |   | || (_| |
|_| |_| \___||_||_| \___/    \_/\_/   \___/ |_|   |_| \__,_|
    "#;

    // Render banner in the entire terminal area
    terminal.draw(|frame| {
        let size = frame.size();
        let paragraph = Paragraph::new(banner)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(paragraph, size);
    })?;

    print!("Welcome to the Interactive, Multilingual Greeter!{}", LINE_ENDING);
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// User Input (Name) + Greeting
////////////////////////////////////////////////////////////////////////////////

/// Prompts the user for their name.
/// If no input is given, returns "World" as a default.
fn prompt_for_name() -> Result<String> {
    print!("What is your name?{}", LINE_ENDING);
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .context("Failed to read from stdin")?;

    let trimmed = name.trim();
    if trimmed.is_empty() {
        Ok("World".to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

/// Draws a random greeting for `name` in a random color using TUI.
fn greet_in_tui(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    name: &str,
) -> Result<()> {
    let greeting_line = pick_random_greeting(name);

    terminal.draw(|frame| {
        // Split the screen to draw in a separate region
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(1)].as_ref())
            .split(frame.size());

        // Random color from tui::style::Color
        let color = random_tui_color();

        let paragraph = Paragraph::new(greeting_line)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(color));
        frame.render_widget(paragraph, chunks[0]);
    })?;

    Ok(())
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
