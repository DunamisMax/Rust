////////////////////////////////////////////////////////////////////////////////
// greeter - A Compact Ratatui TUI w/ extra spacing before prompt
////////////////////////////////////////////////////////////////////////////////

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
    layout::{Alignment, Constraint, Direction, Layout, Rect},
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
#[command(author, version, about = "A compact Ratatui TUI example", long_about = None)]
struct CliArgs {
    /// Example of a positional argument
    #[arg(value_name = "SOME_VALUE")]
    input: Option<String>,

    /// Example of a flag
    #[arg(long, short, help = "Turn on verbose mode")]
    verbose: bool,
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

    // 3) Create Terminal & clear screen
    let mut terminal = setup_terminal().context("Failed to create terminal")?;
    clear_screen(&mut terminal)?;

    // 4) Draw the welcome TUI
    draw_welcome_screen(&mut terminal)?;

    // 5) Temporarily drop raw mode to prompt for name
    drop(_raw_guard);

    // 6) If user didn’t pass an input argument, prompt them with some extra spacing
    let name = match args.input {
        Some(val) => val,
        None => {
            println!("{}", LINE_ENDING); // Extra blank line
            println!("{}", LINE_ENDING); // Another blank line

            println!("Please enter your name:"); // "Please enter your name:" on its own line
            print!("> "); // The ">" prompt on the next line
            io::stdout().flush()?; // Flush so prompt is immediately visible

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

    // 7) Re-enable raw mode to show final TUI
    let _raw_guard = RawModeGuard::new().context("Failed to re-enable raw mode")?;

    // 8) Re-create terminal & clear
    let mut terminal = setup_terminal().context("Failed to create terminal")?;
    clear_screen(&mut terminal)?;

    // 9) Draw greeting TUI
    draw_greeting(&mut terminal, &name)?;

    // 10) Drop raw mode so user can press Enter to exit
    drop(_raw_guard);

    println!("{}", LINE_ENDING); // Extra blank line
    println!("{}", LINE_ENDING); // Another blank line

    print!("Press Enter to exit...{}", LINE_ENDING);
    io::stdout().flush()?;
    let mut exit_buf = String::new();
    io::stdin().read_line(&mut exit_buf)?;

    // Final cleanup
    execute!(terminal.backend_mut(), Clear(ClearType::All), MoveTo(0, 0))?;
    print!("Goodbye!{}", LINE_ENDING);

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Setup Terminal & Clear
////////////////////////////////////////////////////////////////////////////////

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn clear_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    terminal.clear()?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Draw a small "Welcome" TUI (banner + steps box, centered)
////////////////////////////////////////////////////////////////////////////////

fn draw_welcome_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    terminal.draw(|frame| {
        let size = frame.area();

        // Layout:
        // - Top banner area: length 5
        // - Remainder for the main body
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
            .split(size);

        // Banner
        draw_banner(frame, chunks[0]);

        // Center the steps box in the main area
        let steps_area = centered_rect(50, 30, chunks[1]);

        // The steps to display
        let steps = vec![
            ListItem::new("1) Enter your name"),
            ListItem::new("2) See the greeting"),
            ListItem::new("3) Press Enter to exit"),
        ];

        let steps_list = List::new(steps)
            .block(
                Block::default()
                    .title("Steps")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .highlight_symbol(">>");

        frame.render_widget(steps_list, steps_area);
    })?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Draw a "Greeting" TUI, also centered
////////////////////////////////////////////////////////////////////////////////

fn draw_greeting(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    name: &str,
) -> Result<()> {
    terminal.draw(|frame| {
        let size = frame.area();

        // Layout: banner + main content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
            .split(size);

        draw_banner(frame, chunks[0]);

        let greeting_area = centered_rect(50, 40, chunks[1]);

        let lines = vec![
            Line::from(Span::styled(
                format!("Hello, {name}!"),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Enjoy this minimal Ratatui example!",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press Enter to exit.",
                Style::default().fg(Color::Blue),
            )),
        ];

        let paragraph = Paragraph::new(lines).alignment(Alignment::Center).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Greetings! ")
                .border_style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
        );

        frame.render_widget(paragraph, greeting_area);
    })?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Draw a top banner
////////////////////////////////////////////////////////////////////////////////

fn draw_banner(frame: &mut Frame, area: Rect) {
    let line1 = Line::from(Span::styled(
        "GREETER APP",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let line2 = Line::from("A minimal TUI demonstration");

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
// Helper: center a smaller box within a given area
////////////////////////////////////////////////////////////////////////////////

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
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
