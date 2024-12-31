////////////////////////////////////////////////////////////////////////////////
// File Commander - TUI Version with Tokio, Clap, crossterm, and tui
////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
// Imports
////////////////////////////////////////////////////////////////////////////////

use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
// Removed `rayon::prelude::*` since we're no longer using `.par_iter()`
use std::io::Write;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
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
#[command(
    author,
    version,
    about = "File Commander TUI - Demonstration of file ops with crossterm+tui",
    long_about = None
)]
struct CliArgs {
    /// Whether to enable verbose mode
    #[arg(long, short)]
    verbose: bool,
}

////////////////////////////////////////////////////////////////////////////////
// Type Aliases / Errors
////////////////////////////////////////////////////////////////////////////////

type DynError = Box<dyn Error + Send + Sync + 'static>;

////////////////////////////////////////////////////////////////////////////////
// State
////////////////////////////////////////////////////////////////////////////////

/// Tracks the current state of the TUI application.
struct AppState {
    /// Current directory context
    current_dir: PathBuf,
    /// Lines to display in our "log" output at the bottom
    log_lines: Vec<String>,
    /// Index of currently highlighted menu item
    menu_index: usize,
    /// The main menu items
    menu_items: Vec<&'static str>,
}

impl AppState {
    fn new() -> Result<Self, DynError> {
        Ok(Self {
            current_dir: std::env::current_dir()?,
            log_lines: Vec::new(),
            menu_index: 0,
            // The same menu you had, but each entry is a line in the TUI
            menu_items: vec![
                "1) Change directory (cd)",
                "2) List contents (ls)",
                "3) Show directory tree (tree)",
                "4) Show directory info",
                "5) Create file (touch)",
                "6) Create directory (mkdir)",
                "7) Copy file/directory (cp)",
                "8) Move/rename file/directory (mv)",
                "9) Delete file/directory (rm)",
                "10) Duplicate file/directory",
                "11) Organize files (by extension/date/size)",
                "12) Exit",
            ],
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
// Main (Tokio) Entry Point
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<(), DynError> {
    let args = CliArgs::parse();
    if args.verbose {
        print!("Verbose mode enabled...{}", LINE_ENDING);
    }

    // Enable raw mode for TUI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    // Capture mouse events, if you want them
    execute!(stdout, EnableMouseCapture)?;

    // Construct a CrosstermBackend for tui
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Clear screen & print banner
    clear_screen(&mut terminal)?;
    print_welcome_banner(&mut terminal)?;

    // Create our app state
    let mut app_state = AppState::new()?;

    // Enter the TUI event loop
    let res = run_app(&mut terminal, &mut app_state);

    // On exit, restore normal terminal mode
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), DisableMouseCapture)?;

    if let Err(e) = res {
        eprintln!("Error: {e}");
    }

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// TUI: Main Event-Loop
////////////////////////////////////////////////////////////////////////////////

fn run_app<B: tui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app_state: &mut AppState,
) -> Result<(), DynError> {
    loop {
        // Draw the UI
        terminal.draw(|frame| {
            // Split the screen vertically into top/middle/bottom
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),  // banner/instructions area
                    Constraint::Length(14), // menu area
                    Constraint::Min(10),    // log area
                ])
                .split(frame.size());

            // 1) Top pane: Banner + instructions
            let top_text = vec![
                Spans::from(Span::styled(
                    "File Commander TUI",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(tui::style::Modifier::BOLD),
                )),
                Spans::from("Use Up/Down arrows to navigate, Enter to select."),
                Spans::from("Press 'q' to exit at any time."),
                Spans::from(format!(
                    "Current directory: {}",
                    app_state.current_dir.display()
                )),
            ];
            let top_paragraph = Paragraph::new(top_text)
                .block(Block::default().borders(Borders::ALL).title(" Banner "));
            frame.render_widget(top_paragraph, chunks[0]);

            // 2) Middle pane: Menu
            let items: Vec<ListItem> = app_state
                .menu_items
                .iter()
                .enumerate()
                .map(|(i, &title)| {
                    let style = if i == app_state.menu_index {
                        Style::default().fg(Color::Black).bg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    ListItem::new(Span::styled(title, style))
                })
                .collect();
            let menu =
                List::new(items).block(Block::default().borders(Borders::ALL).title(" Menu "));
            frame.render_widget(menu, chunks[1]);

            // 3) Bottom pane: Log output
            let log_items: Vec<ListItem> = app_state
                .log_lines
                .iter()
                .map(|line| ListItem::new(line.clone()))
                .collect();
            let log_widget =
                List::new(log_items).block(Block::default().borders(Borders::ALL).title(" Log "));
            frame.render_widget(log_widget, chunks[2]);
        })?;

        // Handle input
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) => {
                    match (key_event.code, key_event.modifiers) {
                        (KeyCode::Char('q'), _) => {
                            // User pressed 'q' -> exit
                            app_state
                                .log_lines
                                .push("Exiting File Commander. Goodbye!".to_string());
                            return Ok(());
                        }
                        (KeyCode::Up, _) => {
                            if app_state.menu_index > 0 {
                                app_state.menu_index -= 1;
                            }
                        }
                        (KeyCode::Down, _) => {
                            if app_state.menu_index < app_state.menu_items.len() - 1 {
                                app_state.menu_index += 1;
                            }
                        }
                        (KeyCode::Enter, _) => {
                            let choice = app_state.menu_index + 1;
                            if choice == 1 {
                                change_directory(app_state)?;
                            } else if choice == 2 {
                                list_contents(app_state)?;
                            } else if choice == 3 {
                                show_tree_view(app_state)?;
                            } else if choice == 4 {
                                show_directory_info(app_state)?;
                            } else if choice == 5 {
                                create_file(app_state)?;
                            } else if choice == 6 {
                                create_directory(app_state)?;
                            } else if choice == 7 {
                                copy_interactive(app_state)?;
                            } else if choice == 8 {
                                move_or_rename_interactive(app_state)?;
                            } else if choice == 9 {
                                delete_interactive(app_state)?;
                            } else if choice == 10 {
                                duplicate_interactive(app_state)?;
                            } else if choice == 11 {
                                organize_files_interactive(app_state)?;
                            } else if choice == 12 {
                                app_state
                                    .log_lines
                                    .push("Exiting File Commander. Goodbye!".to_string());
                                return Ok(());
                            }
                        }
                        // Support Ctrl+C to exit quickly
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            app_state
                                .log_lines
                                .push("Exiting File Commander via Ctrl+C. Goodbye!".to_string());
                            return Ok(());
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Clear Screen & Welcome Banner (TUI)
////////////////////////////////////////////////////////////////////////////////

/// Clears the terminal screen for a clean start using tui.
fn clear_screen(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<(), DynError> {
    terminal.clear()?;
    Ok(())
}

/// Prints a banner with ASCII art at the top using tui widgets.
fn print_welcome_banner(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
) -> Result<(), DynError> {
    // Example ASCII banner (similar to before)
    let banner = r#"
  __  _  _                                                              _
 / _|(_)| |                                                            | |
| |_  _ | |  ___    ___   ___   _ __ ___   _ __ ___    __ _  _ __    __| |  ___  _ __
|  _|| || | / _ \  / __| / _ \ | '_ ` _ \ | '_ ` _ \  / _` || '_ \  / _` | / _ \| '__|
| |  | || ||  __/ | (__ | (_) || | | | | || | | | | || (_| || | | || (_| ||  __/| |
|_|  |_||_| \___|  \___| \___/ |_| |_| |_||_| |_| |_| \__,_||_| |_| \__,_| \___||_|
    "#;
    // We simply log the banner (it appears in the bottom "Log" after the first draw)
    let mut lines = banner.lines().collect::<Vec<&str>>();
    lines.push("Welcome to the File Commander CLI!");
    for ln in lines {
        print!("{}{}", ln, LINE_ENDING);
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Menu Actions
////////////////////////////////////////////////////////////////////////////////

/// 1) Change directory (cd).
fn change_directory(app_state: &mut AppState) -> Result<(), DynError> {
    let path = read_user_input("Enter path to change directory: ")?;
    let trimmed = path.trim();
    if trimmed.is_empty() {
        app_state
            .log_lines
            .push("No directory provided. Aborting.".to_string());
        return Ok(());
    }

    let target = if trimmed.starts_with('/') {
        PathBuf::from(trimmed)
    } else {
        app_state.current_dir.join(trimmed)
    };

    if target.is_dir() {
        app_state.current_dir = target.canonicalize()?;
        app_state
            .log_lines
            .push(format!("Directory changed to {:?}", app_state.current_dir));
    } else {
        app_state
            .log_lines
            .push(format!("Error: {:?} is not a valid directory.", target));
    }
    Ok(())
}

/// 2) List directory contents, similar to `ls`.
fn list_contents(app_state: &mut AppState) -> Result<(), DynError> {
    let show_hidden = read_user_input("Show hidden files? (y/n): ")?;
    let show_hidden = matches_yes(&show_hidden);

    let dir = &app_state.current_dir;
    let entries = fs::read_dir(dir)?;
    app_state.log_lines.push(format!("Contents of {:?}:", dir));

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !show_hidden && file_name.starts_with('.') {
            continue;
        }
        app_state.log_lines.push(format!("  {}", file_name));
    }
    Ok(())
}

/// 3) Show all files/folders in a tree view.
fn show_tree_view(app_state: &mut AppState) -> Result<(), DynError> {
    let path = read_user_input(&format!(
        "Enter directory path for tree view (default: {}): ",
        app_state.current_dir.display()
    ))?;
    let dir_path = if path.trim().is_empty() {
        app_state.current_dir.clone()
    } else {
        PathBuf::from(path.trim())
    };

    if !dir_path.is_dir() {
        app_state
            .log_lines
            .push(format!("Error: {:?} is not a valid directory.", dir_path));
        return Ok(());
    }

    app_state
        .log_lines
        .push("=== Directory Tree View ===".to_string());
    print_directory_tree(&dir_path, 0, app_state)?;
    Ok(())
}

fn print_directory_tree(
    dir: &Path,
    level: usize,
    app_state: &mut AppState,
) -> Result<(), DynError> {
    let indent = "  ".repeat(level);
    let dir_name = dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    app_state
        .log_lines
        .push(format!("{}- {}", indent, dir_name));

    let entries = fs::read_dir(dir)?;
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        } else {
            files.push(path);
        }
    }
    for d in &dirs {
        print_directory_tree(d, level + 1, app_state)?;
    }
    for f in &files {
        let file_name = f.file_name().unwrap_or_default().to_string_lossy();
        app_state
            .log_lines
            .push(format!("{}  * {}", "  ".repeat(level + 1), file_name));
    }
    Ok(())
}

/// 4) Show directory info.
fn show_directory_info(app_state: &mut AppState) -> Result<(), DynError> {
    let path = read_user_input(&format!(
        "Enter directory path for info (default: {}): ",
        app_state.current_dir.display()
    ))?;
    let dir_path = if path.trim().is_empty() {
        app_state.current_dir.clone()
    } else {
        PathBuf::from(path.trim())
    };

    if !dir_path.is_dir() {
        app_state
            .log_lines
            .push(format!("Error: {:?} is not a valid directory.", dir_path));
        return Ok(());
    }

    app_state
        .log_lines
        .push("=== Directory Info ===".to_string());
    let (total_size, file_count, dir_count) = compute_directory_stats(&dir_path)?;
    app_state
        .log_lines
        .push(format!("Path: {}", dir_path.display()));
    app_state
        .log_lines
        .push(format!("Total size (bytes): {}", total_size));
    app_state
        .log_lines
        .push(format!("Files: {}, Directories: {}", file_count, dir_count));

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let metadata = fs::metadata(&dir_path)?;
        app_state
            .log_lines
            .push(format!("Owner UID: {}", metadata.uid()));
        app_state
            .log_lines
            .push(format!("Owner GID: {}", metadata.gid()));
    }

    Ok(())
}

/// Recursively compute total size, file count, and directory count of a directory.
fn compute_directory_stats(dir: &Path) -> Result<(u64, u64, u64), DynError> {
    let mut total_size = 0;
    let mut file_count = 0;
    let mut dir_count = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let meta = entry.metadata()?;

        if path.is_dir() {
            dir_count += 1;
            let (sub_size, sub_files, sub_dirs) = compute_directory_stats(&path)?;
            total_size += sub_size;
            file_count += sub_files;
            dir_count += sub_dirs;
        } else {
            file_count += 1;
            total_size += meta.len();
        }
    }

    Ok((total_size, file_count, dir_count))
}

/// 5) Create a new file (touch).
fn create_file(app_state: &mut AppState) -> Result<(), DynError> {
    let filename = read_user_input("Enter name of file to create: ")?;
    let trimmed = filename.trim();
    if trimmed.is_empty() {
        app_state
            .log_lines
            .push("Aborted: no filename provided.".to_string());
        return Ok(());
    }
    let new_file_path = app_state.current_dir.join(trimmed);
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&new_file_path)
    {
        Ok(_) => {
            app_state
                .log_lines
                .push(format!("File created at {:?}", new_file_path));
        }
        Err(e) => {
            app_state
                .log_lines
                .push(format!("Could not create file: {}", e));
        }
    }
    Ok(())
}

/// 6) Create a new directory (mkdir).
fn create_directory(app_state: &mut AppState) -> Result<(), DynError> {
    let name = read_user_input("Enter name of directory to create: ")?;
    let trimmed = name.trim();
    if trimmed.is_empty() {
        app_state
            .log_lines
            .push("Aborted: no directory name provided.".to_string());
        return Ok(());
    }
    let new_dir_path = app_state.current_dir.join(trimmed);
    match fs::create_dir(&new_dir_path) {
        Ok(_) => {
            app_state
                .log_lines
                .push(format!("Directory created at {:?}", new_dir_path));
        }
        Err(e) => {
            app_state
                .log_lines
                .push(format!("Could not create directory: {}", e));
        }
    }
    Ok(())
}

/// 7) Copy file/directory (cp).
fn copy_interactive(app_state: &mut AppState) -> Result<(), DynError> {
    let source = read_user_input("Enter source file/directory: ")?;
    let destination = read_user_input("Enter destination path: ")?;

    let source_path = PathBuf::from(source.trim());
    let destination_path = PathBuf::from(destination.trim());

    if !source_path.exists() {
        app_state
            .log_lines
            .push(format!("Error: source {:?} does not exist.", source_path));
        return Ok(());
    }

    if source_path.is_file() {
        match fs::copy(&source_path, &destination_path) {
            Ok(_) => app_state
                .log_lines
                .push("File copied successfully.".to_string()),
            Err(e) => app_state.log_lines.push(format!("File copy failed: {}", e)),
        }
    } else {
        copy_directory_recursive(&source_path, &destination_path)?;
        app_state
            .log_lines
            .push("Directory copied successfully.".to_string());
    }

    Ok(())
}

/// Recursively copy a directory and its contents.
fn copy_directory_recursive(source: &Path, dest: &Path) -> Result<(), DynError> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if path.is_dir() {
            copy_directory_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

/// 8) Move/rename file/directory (mv).
fn move_or_rename_interactive(app_state: &mut AppState) -> Result<(), DynError> {
    let source = read_user_input("Enter source file/directory: ")?;
    let dest = read_user_input("Enter new path/filename: ")?;

    let source_path = PathBuf::from(source.trim());
    let dest_path = PathBuf::from(dest.trim());

    if !source_path.exists() {
        app_state
            .log_lines
            .push(format!("Error: source {:?} does not exist.", source_path));
        return Ok(());
    }

    match fs::rename(&source_path, &dest_path) {
        Ok(_) => app_state
            .log_lines
            .push("Move/rename succeeded.".to_string()),
        Err(e) => app_state
            .log_lines
            .push(format!("Move/rename failed: {}", e)),
    }
    Ok(())
}

/// 9) Delete file/directory (rm).
fn delete_interactive(app_state: &mut AppState) -> Result<(), DynError> {
    let target = read_user_input("Enter file/directory to delete: ")?;
    let target_path = PathBuf::from(target.trim());

    if !target_path.exists() {
        app_state
            .log_lines
            .push(format!("Error: {:?} does not exist.", target_path));
        return Ok(());
    }

    let confirm = read_user_input(&format!(
        "Are you sure you want to delete {:?}? (y/n): ",
        target_path
    ))?;
    if matches_yes(&confirm) {
        if target_path.is_dir() {
            match fs::remove_dir_all(&target_path) {
                Ok(_) => app_state.log_lines.push("Directory deleted.".to_string()),
                Err(e) => app_state
                    .log_lines
                    .push(format!("Failed to delete directory: {}", e)),
            }
        } else {
            match fs::remove_file(&target_path) {
                Ok(_) => app_state.log_lines.push("File deleted.".to_string()),
                Err(e) => app_state
                    .log_lines
                    .push(format!("Failed to delete file: {}", e)),
            }
        }
    } else {
        app_state
            .log_lines
            .push("Delete action canceled.".to_string());
    }
    Ok(())
}

/// 10) Duplicate a file/directory quickly by adding `_copy` or similar suffix.
fn duplicate_interactive(app_state: &mut AppState) -> Result<(), DynError> {
    let source = read_user_input("Enter file/directory to duplicate: ")?;
    let source_path = PathBuf::from(source.trim());

    if !source_path.exists() {
        app_state
            .log_lines
            .push(format!("Error: {:?} does not exist.", source_path));
        return Ok(());
    }

    let mut duplicate_path = source_path.clone();
    let file_name = duplicate_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let extension = duplicate_path
        .extension()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let new_name = if extension.is_empty() {
        format!("{}_copy", file_name)
    } else {
        format!("{}_copy.{}", file_name, extension)
    };
    duplicate_path.set_file_name(new_name);

    if source_path.is_dir() {
        copy_directory_recursive(&source_path, &duplicate_path)?;
    } else {
        fs::copy(&source_path, &duplicate_path)?;
    }
    app_state
        .log_lines
        .push(format!("Duplicate created at {:?}", duplicate_path));

    Ok(())
}

/// 11) Organize files (now single-threaded).
fn organize_files_interactive(app_state: &mut AppState) -> Result<(), DynError> {
    app_state
        .log_lines
        .push("=== Organize Files ===".to_string());
    let input_dir_str = read_user_input("Enter the path of the directory to organize: ")?;
    let input_dir = PathBuf::from(input_dir_str.trim());

    if !input_dir.is_dir() {
        app_state
            .log_lines
            .push(format!("Error: {:?} is not a valid directory.", input_dir));
        return Ok(());
    }

    let method_str = read_user_input(
        "Organization Methods:\r\n  1) By Extension\r\n  2) By Date\r\n  3) By Size\r\nSelect a method (1/2/3): "
    )?;

    let dry_run_str = read_user_input("Dry Run? (y/n): ")?;
    let dry_run = matches_yes(&dry_run_str);

    let files = collect_files(&input_dir)?;

    match method_str.trim() {
        "1" => {
            for e in files.iter() {
                organize_by_extension(e, &input_dir, dry_run, app_state)?;
            }
            app_state
                .log_lines
                .push("Organized by extension!".to_string());
        }
        "2" => {
            for e in files.iter() {
                organize_by_date(e, &input_dir, dry_run, app_state)?;
            }
            app_state.log_lines.push("Organized by date!".to_string());
        }
        "3" => {
            for e in files.iter() {
                organize_by_size(e, &input_dir, dry_run, app_state)?;
            }
            app_state.log_lines.push("Organized by size!".to_string());
        }
        _ => {
            app_state
                .log_lines
                .push("Invalid method chosen. Returning to main menu.".to_string());
        }
    }

    Ok(())
}

/// Recursively collects files (not directories) from the given directory.
fn collect_files(dir: &Path) -> Result<Vec<fs::DirEntry>, io::Error> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_files(&path)?);
        } else {
            files.push(entry);
        }
    }
    Ok(files)
}

fn organize_by_extension(
    entry: &fs::DirEntry,
    root_dir: &Path,
    dry_run: bool,
    app_state: &mut AppState,
) -> Result<(), DynError> {
    let path = entry.path();
    if let Some(ext_os) = path.extension() {
        let extension = ext_os.to_string_lossy();
        let target_dir = root_dir.join("by_extension").join(extension.to_lowercase());
        move_file_or_dry_run(&path, &target_dir, dry_run, app_state)?;
    } else {
        let target_dir = root_dir.join("by_extension").join("no_ext");
        move_file_or_dry_run(&path, &target_dir, dry_run, app_state)?;
    }
    Ok(())
}

fn organize_by_date(
    entry: &fs::DirEntry,
    root_dir: &Path,
    dry_run: bool,
    app_state: &mut AppState,
) -> Result<(), DynError> {
    let path = entry.path();
    let metadata = fs::metadata(&path)?;
    let file_time = metadata.created().or_else(|_| metadata.modified())?;
    let datetime: DateTime<Local> = file_time.into();
    let date_str = datetime.format("%Y-%m-%d").to_string();
    let target_dir = root_dir.join("by_date").join(date_str);
    move_file_or_dry_run(&path, &target_dir, dry_run, app_state)?;
    Ok(())
}

fn organize_by_size(
    entry: &fs::DirEntry,
    root_dir: &Path,
    dry_run: bool,
    app_state: &mut AppState,
) -> Result<(), DynError> {
    let path = entry.path();
    let metadata = fs::metadata(&path)?;
    let file_size = metadata.len();

    let size_label = if file_size < 1_000_000 {
        "small"
    } else if file_size < 100_000_000 {
        "medium"
    } else {
        "large"
    };

    let target_dir = root_dir.join("by_size").join(size_label);
    move_file_or_dry_run(&path, &target_dir, dry_run, app_state)?;
    Ok(())
}

/// Move file to target dir, or log a dry-run message.
fn move_file_or_dry_run(
    path: &Path,
    target_dir: &Path,
    dry_run: bool,
    app_state: &mut AppState,
) -> Result<(), DynError> {
    if !dry_run {
        fs::create_dir_all(target_dir)?;
        let target_path = target_dir.join(
            path.file_name()
                .ok_or_else(|| "No filename found in path")?,
        );
        fs::rename(path, &target_path)?;
        app_state.log_lines.push(format!(
            "Moved {:?} to {:?}",
            path.file_name().unwrap(),
            target_dir
        ));
    } else {
        app_state.log_lines.push(format!(
            "[DRY RUN] Would move {:?} to {:?}",
            path.file_name().unwrap(),
            target_dir
        ));
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Misc Helpers
////////////////////////////////////////////////////////////////////////////////

/// Helper to interpret "y"/"yes" input as true, everything else as false.
fn matches_yes(input: &str) -> bool {
    let s = input.trim().to_lowercase();
    s == "y" || s == "yes"
}

/// A blocking function to read user input from stdin.
fn read_user_input(prompt_msg: &str) -> Result<String, DynError> {
    print!("{prompt_msg}{}", LINE_ENDING);
    io::stdout().flush()?;

    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf)
}
