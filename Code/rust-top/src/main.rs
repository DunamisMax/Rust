////////////////////////////////////////////////////////////////////////////////
// Imports
////////////////////////////////////////////////////////////////////////////////

use anyhow::Result;
use clap::Parser;
use std::{
    io::{self, Write},
    path::Path,
    time::Duration,
};

use crossterm::{
    cursor::MoveTo,
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Terminal,
};

use tokio::time::interval;

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
#[command(
    author,
    version,
    about = "Linux TUI-based Task Manager",
    long_about = None
)]
struct CliArgs {
    /// How often (in milliseconds) to refresh process list
    #[arg(long, default_value = "2000")]
    refresh_ms: u64,

    /// Enable mouse capture? (default: false)
    #[arg(long)]
    mouse: bool,
}

////////////////////////////////////////////////////////////////////////////////
// Struct: ProcessInfo
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    name: String,
    state: String,
    ppid: u32,
    memory_kb: u64,
}

////////////////////////////////////////////////////////////////////////////////
// Main (Tokio) Entry Point
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<()> {
    // 1) Parse CLI arguments
    let args = CliArgs::parse();

    // 2) Enable raw mode, create Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let mut backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    clear_screen(&mut terminal)?;

    // 3) Draw the TUI “Welcome” screen (banner + lines)
    draw_welcome_screen(&mut terminal)?;
    disable_raw_mode()?; // Turn off raw mode to let user read the prompt, etc.

    // Give user a moment to see the welcome banner:
    print!("Press Enter to launch the Task Manager...{}", LINE_ENDING);
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;

    // 4) Re‐enable raw mode, optionally enable mouse capture
    enable_raw_mode()?;
    if args.mouse {
        execute!(terminal.backend_mut(), EnableMouseCapture)?;
    }

    // 5) Enter the main TUI loop that updates the process list
    run_task_manager_tui(&mut terminal, &args).await?;

    // 6) Cleanup before exit
    if args.mouse {
        execute!(terminal.backend_mut(), DisableMouseCapture)?;
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), Clear(ClearType::All), MoveTo(0, 0))?;
    print!("Goodbye!{}", LINE_ENDING);

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Setup and run the main TUI loop
////////////////////////////////////////////////////////////////////////////////

async fn run_task_manager_tui(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    args: &CliArgs,
) -> Result<()> {
    // Create an interval to tick on a regular basis
    let mut refresh_interval = interval(Duration::from_millis(args.refresh_ms));

    loop {
        tokio::select! {
            _ = refresh_interval.tick() => {
                // 1) Gather process info
                let processes = read_process_list()?;

                // 2) Draw updated TUI
                terminal.draw(|frame| {
                    let size = frame.size();

                    // Layout for the TUI
                    let layout = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints([
                            Constraint::Length(3),  // banner area
                            Constraint::Length(1),  // blank spacer
                            Constraint::Min(5),     // table area
                        ])
                        .split(size);

                    // -- [0] Banner (top bar)
                    let banner_lines = get_banner_lines();
                    let banner_par = Paragraph::new(banner_lines)
                        .alignment(Alignment::Left)
                        .block(Block::default().borders(Borders::NONE));
                    frame.render_widget(banner_par, layout[0]);

                    // -- [1] Blank spacer
                    let blank_par = Paragraph::new("");
                    frame.render_widget(blank_par, layout[1]);

                    // -- [2] Process Table
                    let table_block = Block::default()
                        .borders(Borders::ALL)
                        .title(" Process List ");

                    let header = Row::new(vec![
                        Span::styled("PID", Style::default().fg(Color::Yellow)),
                        Span::styled("Name", Style::default().fg(Color::Yellow)),
                        Span::styled("State", Style::default().fg(Color::Yellow)),
                        Span::styled("PPID", Style::default().fg(Color::Yellow)),
                        Span::styled("Memory (KB)", Style::default().fg(Color::Yellow)),
                    ]);

                    // Map each process to a row
                    let rows: Vec<Row> = processes.into_iter().map(|p| {
                        Row::new(vec![
                            Span::raw(p.pid.to_string()),
                            Span::raw(p.name),
                            Span::raw(p.state),
                            Span::raw(p.ppid.to_string()),
                            Span::raw(p.memory_kb.to_string()),
                        ])
                    }).collect();

                    let table = Table::new(rows)
                        .header(header)
                        .block(table_block)
                        .widths(&[
                            Constraint::Length(6),
                            Constraint::Length(20),
                            Constraint::Length(6),
                            Constraint::Length(6),
                            Constraint::Length(12),
                        ])
                        .column_spacing(1);

                    frame.render_widget(table, layout[2]);
                })?;
            }

            // 2) Check for keyboard input
            evt = tokio::task::spawn_blocking(|| crossterm::event::read()) => {
                match evt? {
                    Event::Key(KeyEvent { code, modifiers: KeyModifiers::NONE, .. }) => {
                        match code {
                            KeyCode::Char('q') => {
                                // Quit
                                break;
                            }
                            KeyCode::Esc => {
                                // Also quit
                                break;
                            }
                            _ => {
                                // ignore other keys
                            }
                        }
                    }
                    // If user presses SHIFT+Q, SHIFT+ESC, etc., handle if desired
                    Event::Key(KeyEvent { code, modifiers, .. }) => {
                        if code == KeyCode::Char('Q') && modifiers.contains(KeyModifiers::SHIFT) {
                            break;
                        }
                    }
                    // We won't handle mouse events unless needed
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Read Process List from /proc
////////////////////////////////////////////////////////////////////////////////

fn read_process_list() -> Result<Vec<ProcessInfo>> {
    let mut processes = vec![];

    // In a real app, you'd handle errors or skip directories as needed
    for entry in std::fs::read_dir("/proc")? {
        let entry = entry?;
        let path = entry.path();
        if let Some(file_name) = path.file_name() {
            // If the folder name is all digits, treat it as a PID
            if let Ok(pid) = file_name.to_string_lossy().parse::<u32>() {
                if path.join("stat").exists() {
                    if let Ok(proc_info) = parse_proc_stat(pid) {
                        processes.push(proc_info);
                    }
                }
            }
        }
    }

    Ok(processes)
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Parse /proc/<pid>/stat
// See 'man proc' for the format
////////////////////////////////////////////////////////////////////////////////

fn parse_proc_stat(pid: u32) -> Result<ProcessInfo> {
    let stat_path = format!("/proc/{pid}/stat");
    let contents = std::fs::read_to_string(stat_path)?;
    let parts: Vec<&str> = contents.split_whitespace().collect();
    if parts.len() < 24 {
        // Minimal length check; stat has many fields
        return Err(anyhow::anyhow!("Invalid stat format for pid: {}", pid));
    }

    // Example fields:
    // parts[0] = pid
    // parts[1] = comm (process name, wrapped in parentheses)
    // parts[2] = state (R, S, D, Z, T, etc.)
    // parts[3] = ppid
    // ...
    // parts[22] = RSS usage or similar (not exactly memory in KB, but let's approximate)
    let name = parts[1]
        .trim_start_matches('(')
        .trim_end_matches(')')
        .to_string();
    let state = parts[2].to_string();
    let ppid: u32 = parts[3].parse().unwrap_or(0);

    // The stat file's 24th field (index 23) is `rss`, but for a rough "Memory (KB)" we might read from /proc/<pid>/statm or use a factor.
    // For simplicity, just use RSS from stat.
    let rss: i64 = parts[23].parse().unwrap_or(0);
    // Typically, rss * page_size gives memory in bytes. For a rough KB measure, do:
    let page_size_kb = 4; // Usually 4 KB on many Linux systems
    let memory_kb = rss.saturating_mul(page_size_kb) as u64;

    Ok(ProcessInfo {
        pid,
        name,
        state,
        ppid,
        memory_kb,
    })
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
    // Basic banner
    let banner_text = r#"
                               mm            mm
                               MM            MM
`7Mb,od8 `7MM  `7MM  ,pP"Ybd mmMMmm        mmMMmm   ,pW"Wq.  `7MMpdMAo.
  MM' "'   MM    MM  8I   `"   MM            MM    6W'   `Wb   MM   `Wb
  MM       MM    MM  `YMMMa.   MM    mmmmm   MM    8M     M8   MM    M8
  MM       MM    MM  L.   I8   MM            MM    YA.   ,A9   MM   ,AP
.JMML.     `Mbod"YML.M9mmmP'   `Mbmo         `Mbmo  `Ybmd9'    MMbmmd'
                                                               MM
                                                             .JMML.
"#;

    terminal.draw(|frame| {
        let size = frame.size();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(7), // banner
                Constraint::Length(2), // blank
                Constraint::Length(1), // text line
                Constraint::Length(1), // blank
                Constraint::Length(1), // text line
            ])
            .split(size);

        // chunk[0]: banner
        let banner_lines = banner_text
            .lines()
            .map(|line| {
                Spans::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ))
            })
            .collect::<Vec<_>>();

        let banner_par = Paragraph::new(banner_lines)
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(banner_par, layout[0]);

        // chunk[2]: "Welcome to Task Manager"
        let welcome_par = Paragraph::new("Welcome to Rust TUI Task Manager!")
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(welcome_par, layout[2]);

        // chunk[4]: "Press Enter to start"
        let prompt_par = Paragraph::new("Press Enter to start monitoring processes...")
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(prompt_par, layout[4]);
    })?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Return lines for the top banner in the TUI main loop
////////////////////////////////////////////////////////////////////////////////

fn get_banner_lines() -> Vec<Spans<'static>> {
    vec![
        Spans::from(vec![Span::styled(
            "Rust Linux Task Manager",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Spans::from(vec![Span::styled(
            "Press 'q' or 'Esc' to quit.",
            Style::default().fg(Color::White),
        )]),
    ]
}
