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

#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    name: String,
    state: String,
    ppid: u32,
    memory_kb: u64,
    cpu_percent: f32,
}

/// Tracks CPU usage deltas between intervals
#[derive(Debug, Default)]
struct CpuTracker {
    /// Key: pid, Value: (last total process jiffies, last total system jiffies)
    per_pid_cpu: HashMap<u32, (u64, u64)>,
    /// Last total system jiffies (all CPUs)
    last_total_jiffies: u64,
}

////////////////////////////////////////////////////////////////////////////////
// Main (Tokio) Entry Point
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    // 1) Enable raw mode with RAII guard
    let _raw_guard = RawModeGuard::new().context("Failed to enable raw mode")?;

    // 2) Create a TUI terminal & clear the screen
    let mut terminal = setup_terminal().context("Failed to create terminal")?;
    clear_screen(&mut terminal).context("Failed to clear terminal")?;

    // 3) Draw welcome screen
    draw_welcome_screen(&mut terminal).context("Failed to draw welcome screen")?;

    // 4) Temporarily drop raw mode so user can press Enter
    drop(_raw_guard);
    print!("Press Enter to launch the Task Manager...{}", LINE_ENDING);
    io::stdout().flush()?;

    let mut input_buf = String::new();
    io::stdin()
        .read_line(&mut input_buf)
        .context("Failed to read from stdin")?;

    // 5) Re-enable raw mode for the main TUI
    let _raw_guard = RawModeGuard::new().context("Failed to re-enable raw mode")?;

    // If requested, enable mouse capture
    if args.mouse {
        execute!(terminal.backend_mut(), EnableMouseCapture)
            .context("Failed to enable mouse capture")?;
    }

    // **Clear screen again** to remove the old prompt
    clear_screen(&mut terminal).context("Failed to clear terminal before launching TUI")?;

    // 6) Run the main TUI loop
    run_task_manager_tui(&mut terminal, &args).await?;

    // 7) Cleanup on exit
    if args.mouse {
        execute!(terminal.backend_mut(), DisableMouseCapture)
            .context("Failed to disable mouse capture")?;
    }
    drop(_raw_guard);

    // 8) Final screen clear and goodbye
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
// Main TUI Loop
////////////////////////////////////////////////////////////////////////////////

async fn run_task_manager_tui(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    args: &CliArgs,
) -> Result<()> {
    let mut refresh_interval = interval(Duration::from_millis(args.refresh_ms));
    let mut cpu_tracker = CpuTracker::default();

    loop {
        tokio::select! {
            // 1) On interval tick, gather process info and redraw
            _ = refresh_interval.tick() => {
                #[cfg(target_os="linux")]
                let total_jiffies_now = read_total_jiffies().unwrap_or(0);
                #[cfg(not(target_os="linux"))]
                let total_jiffies_now = 0;

                #[cfg(target_os="linux")]
                let mut processes = read_process_list(&mut cpu_tracker, total_jiffies_now).unwrap_or_default();
                #[cfg(not(target_os="linux"))]
                let mut processes = Vec::new();

                processes.sort_by(|a, b| b.memory_kb.cmp(&a.memory_kb));

                // 2) Redraw TUI
                terminal.draw(|frame| {
                    let size = frame.size();

                    let layout = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints([
                            Constraint::Length(3),
                            Constraint::Length(1),
                            Constraint::Min(5),
                        ])
                        .split(size);

                    // [0] Banner
                    let banner_lines = get_banner_lines();
                    let banner_par = Paragraph::new(banner_lines)
                        .alignment(Alignment::Left)
                        .block(Block::default().borders(Borders::NONE));
                    frame.render_widget(banner_par, layout[0]);

                    // [1] Blank spacer
                    let blank_par = Paragraph::new("");
                    frame.render_widget(blank_par, layout[1]);

                    // [2] Process Table
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

                    let table = Table::new(rows)
                        .header(header)
                        .block(table_block)
                        .widths(&[
                            Constraint::Length(6),
                            Constraint::Length(20),
                            Constraint::Length(6),
                            Constraint::Length(6),
                            Constraint::Length(6),
                            Constraint::Length(12),
                        ])
                        .column_spacing(1);

                    frame.render_widget(table, layout[2]);
                })?;
            },

            // 3) Check for keyboard input with short timeout
            event_result = tokio::task::spawn_blocking(|| {
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
                // The JoinHandle returns a Result<Option<Event>, JoinError>
                let maybe_event = event_result?;
                if let Some(event) = maybe_event {
                    match event {
                        Event::Key(KeyEvent { code, modifiers, .. }) => {
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
                        },
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// CPU / Proc Implementation
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

    let utime: u64 = parts[13].parse().unwrap_or(0);
    let stime: u64 = parts[14].parse().unwrap_or(0);
    let proc_total = utime + stime;

    let rss: i64 = parts[23].parse().unwrap_or(0);
    let page_size_kb = 4;
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

fn human_readable_mem(kb: u64) -> String {
    let bytes = kb.saturating_mul(1024);
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;

    let bytes_f = bytes as f64;
    if bytes_f >= GB {
        format!("{:.2} GB", bytes_f / GB)
    } else if bytes_f >= MB {
        format!("{:.2} MB", bytes_f / MB)
    } else if bytes_f >= KB {
        format!("{:.2} KB", bytes_f / KB)
    } else {
        format!("{} B", bytes)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Draw the “Welcome” TUI
////////////////////////////////////////////////////////////////////////////////

fn draw_welcome_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    let banner_text = r#"
                    |           |
  __|  |   |   __|  __|         __|   _ \   __ \
 |     |   | \__ \  |   _____|  |    (   |  |   |
_|    \__,_| ____/ \__|        \__| \___/   .__/
                                           _|
"#;

    terminal.draw(|frame| {
        let size = frame.size();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(7),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(size);

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

        let welcome_par = Paragraph::new("Welcome to the Rust TUI Task Manager!")
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(welcome_par, layout[2]);

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
        Spans::from(Span::styled(
            "Rust Linux Task Manager",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            "Press 'q', 'Esc', or Ctrl-C to quit.",
            Style::default().fg(Color::White),
        )),
    ]
}
