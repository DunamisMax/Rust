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
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    // Replaced Spans with Line
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
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
#[command(author, version, about = "Hello World Ratatui App", long_about = None)]
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

    // 3) Create Ratatui Terminal and clear screen
    let mut terminal = setup_terminal().context("Failed to create terminal")?;
    clear_screen(&mut terminal).context("Failed to clear terminal")?;

    // 4) Draw the Ratatui “Welcome” screen (banner + lines + sidebar + gauge)
    draw_welcome_screen(&mut terminal).context("Failed to draw welcome screen")?;

    // 5) Temporarily drop raw mode to let the user type normally
    drop(_raw_guard);

    // 6) If user didn’t pass an input argument, prompt them for a name
    let name = match args.input {
        Some(val) => val,
        None => {
            // The Ratatui screen is still visible, but we’re in normal mode. Type below the TUI lines:
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
// Utility: Draw the “Welcome” Ratatui
////////////////////////////////////////////////////////////////////////////////

fn draw_welcome_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    let banner_text = r#"
______  __      ____________              ___       __               _______________
___  / / /_____ ___  /___  /______        __ |     / /______ ___________  /______  /
__  /_/ / _  _ \__  / __  / _  __ \       __ | /| / / _  __ \__  ___/__  / _  __  /
_  __  /  /  __/_  /  _  /  / /_/ /       __ |/ |/ /  / /_/ /_  /    _  /  / /_/ /
/_/ /_/   \___/ /_/   /_/   \____/        ____/|__/   \____/ /_/     /_/   \__,_/
"#;

    terminal.draw(|frame| {
        let size = frame.area(); // replaced frame.size() with frame.area()

        // Split the screen vertically into two main chunks:
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(10), Constraint::Length(5)].as_ref())
            .split(size);

        // Further split the top chunk horizontally into a main area (banner + instructions)
        // and a sidebar with helpful tips.
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)])
            .split(chunks[0]);

        // Render the banner and instructions in the main area
        {
            let banner_lines = banner_text
                .lines()
                .map(|line| {
                    Line::from(Span::styled(
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

            frame.render_widget(banner_paragraph, top_chunks[0]);
        }

        // Render a quick "sidebar" list in the right chunk
        {
            let items = vec![
                ListItem::new("1) Enter your name"),
                ListItem::new("2) See the greeting"),
                ListItem::new("3) Press Enter to exit"),
            ];
            let list = List::new(items)
                .block(
                    Block::default()
                        .title("Steps")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Magenta)),
                )
                .highlight_symbol(">> ");

            frame.render_widget(list, top_chunks[1]);
        }

        // Render a gauge in the bottom chunk to show some “progress”
        {
            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .title("Startup Progress")
                        .borders(Borders::ALL),
                )
                .gauge_style(
                    Style::default()
                        .fg(Color::Green)
                        .bg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                )
                .ratio(0.66);

            frame.render_widget(gauge, chunks[1]);
        }
    })?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Show greeting with Ratatui
////////////////////////////////////////////////////////////////////////////////

fn draw_greeting(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    name: &str,
) -> Result<()> {
    terminal.draw(|frame| {
        let size = frame.area(); // replaced frame.size() with frame.area()

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Percentage(100)])
            .split(size);

        let lines = vec![
            Line::from(Span::styled(
                format!("Hello, {name}!"),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "This is a simple Hello World Ratatui app.",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press Enter to exit.",
                Style::default().fg(Color::Blue),
            )),
            Line::from(""),
        ];

        let block = Block::default().borders(Borders::ALL).title("Greetings!");
        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(block);

        frame.render_widget(paragraph, layout[0]);
    })?;

    Ok(())
}
