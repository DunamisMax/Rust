////////////////////////////////////////////////////////////////////////////////
// benchmarker - CPU + RAM TUI
////////////////////////////////////////////////////////////////////////////////

use anyhow::{Context, Result}; // Removed `anyhow` macro since it's unused
use clap::Parser;
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::task;

////////////////////////////////////////////////////////////////////////////////
// Cross-platform line endings
////////////////////////////////////////////////////////////////////////////////

#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";

#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";

////////////////////////////////////////////////////////////////////////////////
// CLI
////////////////////////////////////////////////////////////////////////////////

#[derive(Parser, Debug)]
#[command(author, version, about = "System Bench (CPU + RAM)", long_about = None)]
struct CliArgs {
    /// How large in MB to attempt usage for the RAM benchmark
    #[arg(long, default_value_t = 0)]
    ram_mb: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Benchmark {
    None,
    Cpu,
    Ram,
    Combined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Welcome,
    Menu,
    BenchInProgress,
    Exit,
}

struct App {
    screen: Screen,
    active_bench: Benchmark,
    status_message: String,
    cli_ram_mb: usize,
}

/// RAII guard for raw mode
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
// main
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    // Enable raw mode
    let _rm = RawModeGuard::new()?;

    // Set up the TUI
    let mut terminal = setup_terminal()?;
    clear_screen(&mut terminal)?;

    let app = App {
        screen: Screen::Welcome,
        active_bench: Benchmark::None,
        status_message: String::new(),
        cli_ram_mb: args.ram_mb,
    };

    // Run main TUI loop
    if let Err(e) = run_app(&mut terminal, app).await {
        finalize_terminal(&mut terminal)?;
        eprintln!("Error: {e}");
        return Err(e);
    }

    finalize_terminal(&mut terminal)?;
    drop(_rm);

    // Prompt user to press Enter before exiting
    print!("Press Enter to exit...{LINE_ENDING}");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;

    execute!(io::stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
    print!("Goodbye!{LINE_ENDING}");

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Terminal Setup Utilities
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

fn finalize_terminal(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    execute!(terminal.backend_mut(), Clear(ClearType::All), MoveTo(0, 0))?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Main TUI Loop
////////////////////////////////////////////////////////////////////////////////

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    mut app: App,
) -> Result<()> {
    // A shared atomic bool to signal benchmark loops to stop.
    let run_flag = Arc::new(AtomicBool::new(false));

    loop {
        // Draw the current UI
        terminal.draw(|f| draw_ui(f, &app))?;

        // Check for key input, replacing single-pattern match with an if let
        if crossterm::event::poll(Duration::from_millis(200))? {
            if let Event::Key(k) = event::read()? {
                handle_key_event(k, &mut app, &run_flag).await?;
            }
        }

        // Exit condition
        if app.screen == Screen::Exit {
            break;
        }
    }

    run_flag.store(false, Ordering::SeqCst);
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// UI Drawing
////////////////////////////////////////////////////////////////////////////////

fn draw_ui(frame: &mut Frame, app: &App) {
    let screen_area = frame.area();

    let chunks = ratatui::layout::Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // banner
            Constraint::Min(0),    // main area
            Constraint::Length(3), // status
        ])
        .split(screen_area);

    // Banner
    draw_banner(frame, chunks[0]);

    // Main content
    match app.screen {
        Screen::Welcome => draw_welcome(frame, chunks[1]),
        Screen::Menu => draw_menu(frame, chunks[1]),
        Screen::BenchInProgress => draw_bench_in_progress(frame, app, chunks[1]),
        Screen::Exit => {}
    }

    // Status bar
    let block = ratatui::widgets::Block::default()
        .title(" Status ")
        .borders(Borders::ALL);
    let para = Paragraph::new(app.status_message.as_str())
        .block(block)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(para, chunks[2]);
}

fn draw_banner(frame: &mut Frame, area: Rect) {
    let l1 = Line::from(Span::styled(
        " SYSTEM BENCH ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    let l2 = Line::from(Span::styled(
        " CPU + RAM ",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::ITALIC),
    ));
    let paragraph = Paragraph::new(vec![l1, l2])
        .block(Block::default().borders(Borders::ALL).title(" Welcome "))
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn draw_welcome(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("Welcome!")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let lines = vec![
        Line::from("Welcome to the System Bench TUI (CPU + RAM)."),
        Line::from("Press Enter to continue."),
        Line::from("Press Esc to exit."),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn draw_menu(frame: &mut Frame, area: Rect) {
    let opts = [
        "1) CPU Benchmark",
        "2) RAM Benchmark",
        "3) Combined CPU+RAM",
        "4) Exit",
    ];
    let items: Vec<ListItem> = opts.iter().map(|&s| ListItem::new(Span::raw(s))).collect();

    let block = Block::default()
        .title(" Select a Benchmark ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let list = ratatui::widgets::List::new(items).block(block);
    frame.render_widget(list, area);
}

fn draw_bench_in_progress(frame: &mut Frame, app: &App, area: Rect) {
    let desc = match app.active_bench {
        Benchmark::Cpu => "CPU Benchmark Running (Esc=stop)",
        Benchmark::Ram => "RAM Benchmark Running (Esc=stop)",
        Benchmark::Combined => "Combined CPU+RAM Running (Esc=stop)",
        Benchmark::None => "No active benchmark...",
    };
    let block = Block::default()
        .title("Benchmark In Progress")
        .borders(Borders::ALL);
    let paragraph = Paragraph::new(desc)
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Magenta));

    frame.render_widget(paragraph, area);
}

////////////////////////////////////////////////////////////////////////////////
// Input Handling
////////////////////////////////////////////////////////////////////////////////

async fn handle_key_event(key: KeyEvent, app: &mut App, run_flag: &Arc<AtomicBool>) -> Result<()> {
    match app.screen {
        Screen::Welcome => match key.code {
            KeyCode::Enter => {
                app.screen = Screen::Menu;
            }
            KeyCode::Esc => {
                app.screen = Screen::Exit;
            }
            _ => {}
        },
        Screen::Menu => match key.code {
            KeyCode::Char('1') => start_benchmark(app, Benchmark::Cpu, run_flag).await?,
            KeyCode::Char('2') => start_benchmark(app, Benchmark::Ram, run_flag).await?,
            KeyCode::Char('3') => start_benchmark(app, Benchmark::Combined, run_flag).await?,
            KeyCode::Char('4') => app.screen = Screen::Exit,
            _ => {}
        },
        Screen::BenchInProgress => {
            if key.code == KeyCode::Esc {
                // Stop the current benchmark
                run_flag.store(false, Ordering::SeqCst);
                app.active_bench = Benchmark::None;
                app.screen = Screen::Menu;
                app.status_message.clear();
            }
        }
        Screen::Exit => {}
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Benchmarks
////////////////////////////////////////////////////////////////////////////////

/// Initiates the appropriate benchmark(s).
async fn start_benchmark(
    app: &mut App,
    bench: Benchmark,
    run_flag: &Arc<AtomicBool>,
) -> Result<()> {
    // Stop any running benchmark
    run_flag.store(false, Ordering::SeqCst);

    // Start the new benchmark
    run_flag.store(true, Ordering::SeqCst);
    app.active_bench = bench;
    app.screen = Screen::BenchInProgress;
    app.status_message = format!("Starting {bench:?} benchmark...");

    match bench {
        Benchmark::Cpu => spawn_cpu_bench(run_flag.clone()).await,
        Benchmark::Ram => spawn_ram_bench(run_flag.clone(), app.cli_ram_mb).await,
        Benchmark::Combined => {
            // Launch CPU + RAM in parallel
            spawn_cpu_bench(run_flag.clone()).await;
            spawn_ram_bench(run_flag.clone(), app.cli_ram_mb).await;
        }
        Benchmark::None => {}
    }

    Ok(())
}

/// Spawns multiple CPU-bound tasks that spin using trigonometric ops.
async fn spawn_cpu_bench(run_flag: Arc<AtomicBool>) {
    let cores = num_cpus::get();
    for _ in 0..cores {
        let r = run_flag.clone();
        task::spawn(async move {
            while r.load(Ordering::SeqCst) {
                let _ = 2.0_f64.sqrt().sin().cos().tan();
            }
        });
    }
}

/// Spawns a task that continuously writes to a large buffer in memory.
async fn spawn_ram_bench(run_flag: Arc<AtomicBool>, cli_mb: usize) {
    // If no CLI input, default to ~4GB attempt for demonstration
    let guess = 8_000_000_000; // 8GB
    let desired = if cli_mb == 0 {
        guess / 2 // 4GB
    } else {
        cli_mb * 1_000_000
    };

    let mut buffer = vec![0u8; desired];
    let r = run_flag.clone();
    task::spawn(async move {
        let mut idx = 0;
        while r.load(Ordering::SeqCst) {
            buffer[idx] = (idx % 256) as u8;
            idx = (idx + 1) % buffer.len();
        }
    });
}
