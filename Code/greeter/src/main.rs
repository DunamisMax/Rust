////////////////////////////////////////////////////////////////////////////////
// Imports
////////////////////////////////////////////////////////////////////////////////

use anyhow::{Context, Result};
use clap::Parser;
use std::io::{self, Write};

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
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
#[command(author, version, about = "Greeter", long_about = None)]
struct CliArgs {
    /// Example of a positional argument
    #[arg(value_name = "SOME_VALUE")]
    input: Option<String>,

    /// Example of a flag
    #[arg(long, short, help = "Turn on verbose mode")]
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

    // 2) Enable raw mode automatically via RAII guard.
    //    Once the guard is dropped (goes out of scope), raw mode is disabled.
    let _raw_guard = RawModeGuard::new().context("Failed to enable raw mode")?;

    // 3) Create TUI Terminal and clear screen
    let mut terminal = setup_terminal().context("Failed to create terminal")?;
    clear_screen(&mut terminal).context("Failed to clear terminal")?;

    // 4) Draw the TUI “Welcome” screen (banner + lines)
    draw_welcome_screen(&mut terminal).context("Failed to draw welcome screen")?;

    // 5) Temporarily drop raw mode to let the user type normally
    drop(_raw_guard);
    // Because we've dropped our guard, raw mode is OFF now.

    // 6) If user didn’t pass an input argument, prompt them for a name
    let name = match args.input {
        Some(val) => val,
        None => {
            // The TUI is still visible, but we’re in normal mode. Type below the TUI lines:
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .context("Failed to read line")?;
            let trimmed = input.trim().to_string();
            if trimmed.is_empty() {
                "Stranger".to_string()
            } else {
                trimmed
            }
        }
    };

    // 7) Re-enable raw mode for the final TUI
    let _raw_guard = RawModeGuard::new().context("Failed to re-enable raw mode")?;

    // 8) Re-create the terminal (stdout might need refreshing after raw mode changes)
    let mut terminal = setup_terminal().context("Failed to create terminal")?;
    clear_screen(&mut terminal).context("Failed to clear terminal")?;
    draw_greeting(&mut terminal, &name).context("Failed to draw greeting")?;

    // 9) Disable raw mode so user can press Enter, then exit
    drop(_raw_guard);

    print!("   Press Enter to exit...{}", LINE_ENDING);
    io::stdout().flush().context("Failed to flush stdout")?;
    let mut exit_buf = String::new();
    io::stdin()
        .read_line(&mut exit_buf)
        .context("Failed to read line")?;

    // 10) Final cleanup: clear screen, print goodbye
    execute!(terminal.backend_mut(), Clear(ClearType::All), MoveTo(0, 0))?;
    print!("Goodbye!{}", LINE_ENDING);

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// RAII guard for raw mode
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
            // If something goes wrong while disabling, we can’t do much more than ignore it.
            let _ = disable_raw_mode();
        }
    }
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
// Utility: Draw the “Welcome” TUI
////////////////////////////////////////////////////////////////////////////////

fn draw_welcome_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    // Banner ASCII – adjust if you want your own style
    let banner_text = r#"
   _   _      _ _         __        __         _     _
  | | | | ___| | | ___     \ \      / /__  _ __| | __| |
  | |_| |/ _ \ | |/ _ \     \ \ /\ / / _ \| '__| |/ _` |
  |  _  |  __/ | | (_) |     \ V  V / (_) | |  | | (_| |
  |_| |_|\___|_|_|\___/       \_/\_/ \___/|_|  |_|\__,_|
"#;

    terminal.draw(|frame| {
        let size = frame.size();

        // layout:
        // chunk[0]: banner
        // chunk[1]: blank line
        // chunk[2]: "Welcome...!"
        // chunk[3]: blank line
        // chunk[4]: "Please enter your name:"
        // chunk[5]: blank line
        // chunk[6]: prompt ">"
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(6), // banner height
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(size);

        // chunk[0] – banner
        let banner_lines = banner_text
            .lines()
            .map(|line| {
                Spans::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            })
            .collect::<Vec<_>>();

        let banner_paragraph = Paragraph::new(banner_lines)
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(banner_paragraph, layout[0]);

        // chunk[1]: blank line
        let blank_par = Paragraph::new("").block(Block::default());
        frame.render_widget(blank_par, layout[1]);

        // chunk[2]: "Welcome to Hello World CLI!"
        let welcome_par = Paragraph::new("Welcome to Hello World CLI!")
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(welcome_par, layout[2]);

        // chunk[3]: blank line
        let blank_par = Paragraph::new("").block(Block::default());
        frame.render_widget(blank_par, layout[3]);

        // chunk[4]: "Please enter your name:"
        let prompt_line = Paragraph::new("Please enter your name:")
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(prompt_line, layout[4]);

        // chunk[5]: blank line
        let blank_par = Paragraph::new("").block(Block::default());
        frame.render_widget(blank_par, layout[5]);

        // chunk[6]: ">"
        let arrow_par = Paragraph::new(">")
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(arrow_par, layout[6]);
    })?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Show greeting with TUI
////////////////////////////////////////////////////////////////////////////////

fn draw_greeting(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    name: &str,
) -> Result<()> {
    terminal.draw(|frame| {
        let size = frame.size();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(100)])
            .split(size);

        let lines = vec![
            Spans::from(Span::styled(
                format!("Hello, {name}!"),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Spans::from(""),
            Spans::from(Span::styled(
                "This is a simple Hello World TUI.",
                Style::default().fg(Color::Yellow),
            )),
            Spans::from(""),
            Spans::from(Span::styled(
                "Press Enter to exit.",
                Style::default().fg(Color::Blue),
            )),
            Spans::from(""),
        ];

        let block = Block::default().borders(Borders::ALL).title("Greetings!");
        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Left)
            .block(block);

        frame.render_widget(paragraph, layout[0]);
    })?;

    Ok(())
}
