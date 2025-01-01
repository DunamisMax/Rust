////////////////////////////////////////////////////////////////////////////////
// main.rs - rust-top: A TUI-based Task Manager for Linux
//
// Showcases a terminal-based interface for listing processes (on Linux)
// with CPU and memory usage, updated periodically.
////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
// Imports
////////////////////////////////////////////////////////////////////////////////

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::{
    collections::HashMap,
    io::{self, Write},
    time::Duration,
};

use crossterm::{
    cursor::MoveTo,
    event::{
        poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame, Terminal,
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

/// CLI arguments for rust-top, using Clap's derive API.
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "A cross-platform TUI-based Task Manager (Linux /proc usage)",
    long_about = None
)]
struct CliArgs {
    /// How often (in milliseconds) to refresh process list
    #[arg(long, default_value = "2000")]
    refresh_ms: u64,

    /// Enable mouse capture (default: false)
    #[arg(long)]
    mouse: bool,
}

////////////////////////////////////////////////////////////////////////////////
// Data Structures
////////////////////////////////////////////////////////////////////////////////

/// Holds high-level info about a single process (Linux /proc).
#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    name: String,
    state: String,
    ppid: u32,
    memory_kb: u64,
    cpu_percent: f32,
}

/// Tracks CPU usage per PID (last known jiffies) plus system total jiffies.
#[derive(Debug, Default)]
struct CpuTracker {
    // Key: PID, Value: (last total proc jiffies, last total system jiffies)
    per_pid_cpu: HashMap<u32, (u64, u64)>,
    // Stores the last known total system jiffies from /proc/stat
    last_total_jiffies: u64,
}

////////////////////////////////////////////////////////////////////////////////
// RAII Guard for Raw Mode
////////////////////////////////////////////////////////////////////////////////

/// Guards the terminal’s raw mode so we never forget to disable it on drop.
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

    // 2) Enable raw mode
    let _raw_guard = RawModeGuard::new().context("Failed to enable raw mode")?;

    // 3) Create a TUI terminal & clear the screen
    let mut terminal = setup_terminal().context("Failed to create terminal")?;
    clear_screen(&mut terminal).context("Failed to clear terminal")?;

    // 4) Draw welcome screen (Ratatui banner, instructions)
    draw_welcome_screen(&mut terminal).context("Failed to draw welcome screen")?;

    // 5) Temporarily drop raw mode so the user can press Enter to continue
    drop(_raw_guard);
    println!("{}", LINE_ENDING); // Extra blank line
    print!("Press Enter to launch the Task Manager...{}", LINE_ENDING);
    io::stdout().flush()?;

    let mut input_buf = String::new();
    io::stdin()
        .read_line(&mut input_buf)
        .context("Failed to read from stdin")?;

    // 6) Re-enable raw mode for the main TUI
    let _raw_guard = RawModeGuard::new().context("Failed to re-enable raw mode")?;

    // 7) Enable mouse capture if requested
    if args.mouse {
        execute!(terminal.backend_mut(), EnableMouseCapture)
            .context("Failed to enable mouse capture")?;
    }

    // Clear screen again to remove the old prompt
    clear_screen(&mut terminal).context("Failed to clear terminal before TUI")?;

    // 8) Run the main TUI loop
    run_task_manager_tui(&mut terminal, &args).await?;

    // 9) Cleanup: disable mouse capture if it was enabled
    if args.mouse {
        execute!(terminal.backend_mut(), DisableMouseCapture)
            .context("Failed to disable mouse capture")?;
    }
    drop(_raw_guard);

    // 10) Final screen clear and goodbye
    execute!(terminal.backend_mut(), Clear(ClearType::All), MoveTo(0, 0))?;
    print!("Goodbye!{}", LINE_ENDING);

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Setup Terminal
////////////////////////////////////////////////////////////////////////////////

/// Creates a `ratatui::Terminal` using Crossterm as the backend.
fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Clears the terminal screen
////////////////////////////////////////////////////////////////////////////////

/// Clears the TUI terminal buffer fully using Ratatui’s `terminal.clear()`.
fn clear_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    terminal.clear()?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// "Welcome" TUI Screen
////////////////////////////////////////////////////////////////////////////////

/// Draws a Ratatui welcome banner with steps to continue.
/// Shown once before the user presses Enter to proceed.
fn draw_welcome_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    terminal.draw(|frame| {
        let screen = frame.area();

        // We create two chunks:
        // 1) A top chunk for our banner
        // 2) The remainder for instructions
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
            .split(screen);

        // (A) Draw a top banner in chunk 0
        draw_banner(frame, chunks[0]);

        // (B) Center instructions in chunk 1
        let centered = centered_rect(60, 40, chunks[1]);

        // For the text, we can either use a Paragraph or a List.
        // We'll just do a short welcome paragraph:
        let lines = vec![
            Line::from(Span::raw("Welcome to rust-top!")),
            Line::from(""),
            Line::from(Span::raw("Press Enter to launch the Task Manager...")),
        ];

        let block = Block::default()
            .title(" Welcome ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let par = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(block);

        frame.render_widget(par, centered);
    })?;

    Ok(())
}

/// Draws a stylized banner at the top.
fn draw_banner(frame: &mut Frame, area: Rect) {
    let line1 = Line::from(Span::styled(
        "RUST-TOP",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    ));
    let line2 = Line::from("A TUI-based Task Manager for Linux in Rust");

    let paragraph = Paragraph::new(vec![line1, line2])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Banner ")
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Centers a rectangular area of (percent_x, percent_y) within `area`.
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

////////////////////////////////////////////////////////////////////////////////
// Main TUI Loop
////////////////////////////////////////////////////////////////////////////////

/// Runs the continuous process-monitor loop.
/// Renders the process table every `refresh_ms` and checks for keyboard events.
async fn run_task_manager_tui(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    args: &CliArgs,
) -> Result<()> {
    let mut refresh_interval = interval(Duration::from_millis(args.refresh_ms));
    let mut cpu_tracker = CpuTracker::default();

    loop {
        tokio::select! {
            // On interval tick, gather process info and redraw
            _ = refresh_interval.tick() => {
                #[cfg(target_os = "linux")]
                let total_jiffies_now = read_total_jiffies().unwrap_or(0);
                #[cfg(not(target_os = "linux"))]
                let total_jiffies_now = 0;

                #[cfg(target_os = "linux")]
                let mut processes = read_process_list(&mut cpu_tracker, total_jiffies_now).unwrap_or_default();
                #[cfg(not(target_os = "linux"))]
                let mut processes = Vec::new();

                // Sort by memory usage descending
                processes.sort_by(|a, b| b.memory_kb.cmp(&a.memory_kb));

                // Redraw TUI
                terminal.draw(|frame| {
                    let screen = frame.area();

                    // We create three main chunks:
                    // 1) a small chunk for the top banner
                    // 2) a 1-line blank spacer
                    // 3) the rest for the process table
                    let layout = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints([
                            Constraint::Length(3),  // banner area
                            Constraint::Length(1),  // blank spacer
                            Constraint::Min(5),     // table area
                        ])
                        .split(screen);

                    // (1) A top banner line
                    let banner_lines = vec![
                        Line::from(Span::styled(
                            "rust-top (press 'q', 'Esc', or Ctrl-C to quit)",
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                        ))
                    ];
                    let banner_par = Paragraph::new(banner_lines)
                        .alignment(Alignment::Left)
                        .block(Block::default().borders(Borders::NONE));
                    frame.render_widget(banner_par, layout[0]);

                    // (2) Blank spacer
                    let blank_par = Paragraph::new(Line::from(""));
                    frame.render_widget(blank_par, layout[1]);

                    // (3) Process Table
                    let table_block = Block::default()
                        .borders(Borders::ALL)
                        .title(" Process List ");

                    let header = Row::new(vec![
                        Span::styled("PID", Style::default().fg(Color::Yellow)),
                        Span::styled("Name", Style::default().fg(Color::Yellow)),
                        Span::styled("State", Style::default().fg(Color::Yellow)),
                        Span::styled("PPID", Style::default().fg(Color::Yellow)),
                        Span::styled("CPU%", Style::default().fg(Color::Yellow)),
                        Span::styled("Memory", Style::default().fg(Color::Yellow)),
                    ]);

                    let rows: Vec<Row> = processes.into_iter().map(|p| {
                        let mem_str = human_readable_mem(p.memory_kb);
                        Row::new(vec![
                            Span::raw(p.pid.to_string()),
                            Span::raw(p.name),
                            Span::raw(p.state),
                            Span::raw(p.ppid.to_string()),
                            Span::raw(format!("{:.1}", p.cpu_percent)),
                            Span::raw(mem_str),
                        ])
                    }).collect();

                    let table = Table::new(
                        rows,
                        &[
                            Constraint::Length(6),   // PID
                            Constraint::Length(20),  // Name
                            Constraint::Length(6),   // State
                            Constraint::Length(6),   // PPID
                            Constraint::Length(6),   // CPU%
                            Constraint::Length(12),  // Memory
                        ],
                    )
                    .header(header)
                    .block(table_block)
                    .column_spacing(1);

                    frame.render_widget(table, layout[2]);
                })?;
            },

            // Check for keyboard input with short timeout
            event_result = tokio::task::spawn_blocking(|| {
                // Poll for an event (non-async, hence spawn_blocking)
                if poll(Duration::from_millis(100)).unwrap_or(false) {
                    // If an event is available, read it
                    if let Ok(ev) = read() {
                        Some(ev)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }) => {
                let maybe_event = event_result?;
                // ** Fixed: Replace nested match with if let **
                if let Some(Event::Key(KeyEvent { code, modifiers, .. })) = maybe_event {
                    // Normal keys
                    if modifiers.is_empty() {
                        match code {
                            KeyCode::Char('q') | KeyCode::Esc => break,
                            _ => {}
                        }
                    }
                    // SHIFT+Q or Ctrl-C
                    if modifiers.contains(KeyModifiers::SHIFT)
                        && code == KeyCode::Char('Q') {
                        break;
                    }
                    if modifiers.contains(KeyModifiers::CONTROL)
                        && code == KeyCode::Char('c') {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Linux-Specific: Reading /proc for CPU & Process Info
////////////////////////////////////////////////////////////////////////////////

#[cfg(target_os = "linux")]
fn read_total_jiffies() -> Result<u64> {
    let contents = std::fs::read_to_string("/proc/stat")?;
    let line = contents
        .lines()
        .find(|l| l.starts_with("cpu "))
        .ok_or_else(|| anyhow!("Could not find 'cpu ' line in /proc/stat"))?;

    let parts: Vec<&str> = line.split_whitespace().skip(1).collect();
    let mut total: u64 = 0;
    for val in parts {
        if let Ok(num) = val.parse::<u64>() {
            total += num;
        }
    }
    Ok(total)
}

#[cfg(target_os = "linux")]
fn read_process_list(
    cpu_tracker: &mut CpuTracker,
    total_jiffies_now: u64,
) -> Result<Vec<ProcessInfo>> {
    let mut processes = vec![];

    for entry in std::fs::read_dir("/proc")? {
        let entry = entry?;
        let path = entry.path();
        if let Some(file_name) = path.file_name() {
            if let Ok(pid) = file_name.to_string_lossy().parse::<u32>() {
                if path.join("stat").exists() {
                    if let Ok(proc_info) = parse_proc_stat(pid, cpu_tracker, total_jiffies_now) {
                        processes.push(proc_info);
                    }
                }
            }
        }
    }

    cpu_tracker.last_total_jiffies = total_jiffies_now;
    Ok(processes)
}

#[cfg(target_os = "linux")]
fn parse_proc_stat(
    pid: u32,
    cpu_tracker: &mut CpuTracker,
    total_jiffies_now: u64,
) -> Result<ProcessInfo> {
    let stat_path = format!("/proc/{pid}/stat");
    let contents =
        std::fs::read_to_string(&stat_path).context(format!("Could not read {}", stat_path))?;

    let parts: Vec<&str> = contents.split_whitespace().collect();
    if parts.len() < 24 {
        return Err(anyhow!("Invalid stat format for pid: {pid}"));
    }

    let name = parts[1]
        .trim_start_matches('(')
        .trim_end_matches(')')
        .to_string();
    let state = parts[2].to_string();
    let ppid: u32 = parts[3].parse().unwrap_or(0);

    // utime + stime fields
    let utime: u64 = parts[13].parse().unwrap_or(0);
    let stime: u64 = parts[14].parse().unwrap_or(0);
    let proc_total = utime + stime;

    // RSS from /proc/<PID>/stat (23rd field)
    let rss: i64 = parts[23].parse().unwrap_or(0);
    let page_size_kb = 4; // typical 4KB
    let memory_kb = (rss.max(0) as u64).saturating_mul(page_size_kb);

    let (old_jiffies_proc, old_jiffies_total) = cpu_tracker
        .per_pid_cpu
        .get(&pid)
        .cloned()
        .unwrap_or((0, cpu_tracker.last_total_jiffies));

    let delta_proc = proc_total.saturating_sub(old_jiffies_proc) as f32;
    let delta_total = total_jiffies_now.saturating_sub(old_jiffies_total) as f32;

    let cpu_percent = if delta_total > 0.0 {
        (delta_proc / delta_total) * 100.0
    } else {
        0.0
    };

    cpu_tracker
        .per_pid_cpu
        .insert(pid, (proc_total, total_jiffies_now));

    Ok(ProcessInfo {
        pid,
        name,
        state,
        ppid,
        memory_kb,
        cpu_percent,
    })
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Memory Format
////////////////////////////////////////////////////////////////////////////////

/// Convert memory from KB to a human-readable string (e.g., "32.0 MB").
fn human_readable_mem(kb: u64) -> String {
    let bytes = kb.saturating_mul(1024);
    const KB_F: f64 = 1024.0;
    const MB_F: f64 = 1024.0 * 1024.0;
    const GB_F: f64 = 1024.0 * 1024.0 * 1024.0;

    let bytes_f = bytes as f64;
    if bytes_f >= GB_F {
        format!("{:.2} GB", bytes_f / GB_F)
    } else if bytes_f >= MB_F {
        format!("{:.2} MB", bytes_f / MB_F)
    } else if bytes_f >= KB_F {
        format!("{:.2} KB", bytes_f / KB_F)
    } else {
        format!("{} B", bytes)
    }
}
