use clap::{Arg, ArgAction, Command, Subcommand};
use rayon::prelude::*;
use std::error::Error;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Local};

#[derive(Subcommand)]
enum OrganizeCommand {
    /// Organize files in the target directory by extension
    Extension,
    /// Organize files by their creation (or modification) date
    Date,
    /// Organize files by size (small, medium, large)
    Size,
}

#[derive(Debug)]
struct Config {
    input_dir: PathBuf,
    command: OrganizeCommand,
    dry_run: bool,
}

fn cli() -> Command {
    Command::new("file_organizer")
        .version("0.1.0")
        .about("A CLI tool to organize files in a directory by extension, date, or size.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("input_dir")
                .help("The directory to organize")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .help("Show the planned moves without executing them")
                .action(ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("extension")
                .about("Organize files by extension")
        )
        .subcommand(
            Command::new("date")
                .about("Organize files by creation (or modification) date")
        )
        .subcommand(
            Command::new("size")
                .about("Organize files by size (small, medium, large)")
        )
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli().get_matches();

    // Extract the input directory
    let input_dir = matches
        .get_one::<String>("input_dir")
        .expect("Directory path is required.")
        .into();

    // Check which subcommand was used
    let (command, dry_run) = match matches.subcommand() {
        Some(("extension", _)) => (OrganizeCommand::Extension, matches.get_flag("dry-run")),
        Some(("date", _)) => (OrganizeCommand::Date, matches.get_flag("dry-run")),
        Some(("size", _)) => (OrganizeCommand::Size, matches.get_flag("dry-run")),
        _ => {
            eprintln!("No valid subcommand provided.");
            std::process::exit(1);
        }
    };

    let config = Config {
        input_dir,
        command,
        dry_run,
    };

    organize_files(&config)?;

    Ok(())
}

/// Main organizer function.
///
/// 1. Collects files in the `config.input_dir`.
/// 2. Depending on the `config.command`, organizes them in parallel.
fn organize_files(config: &Config) -> Result<(), Box<dyn Error>> {
    let files = collect_files(&config.input_dir)?;

    match config.command {
        OrganizeCommand::Extension => {
            files.par_iter().try_for_each(|entry| {
                organize_by_extension(entry, &config.input_dir, config.dry_run)
            })?;
        }
        OrganizeCommand::Date => {
            files.par_iter().try_for_each(|entry| {
                organize_by_date(entry, &config.input_dir, config.dry_run)
            })?;
        }
        OrganizeCommand::Size => {
            files.par_iter().try_for_each(|entry| {
                organize_by_size(entry, &config.input_dir, config.dry_run)
            })?;
        }
    }

    Ok(())
}

/// Recursively collects files (not directories) from the given directory.
fn collect_files(dir: &Path) -> io::Result<Vec<DirEntry>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Recurse into subdirectories
            files.extend(collect_files(&path)?);
        } else {
            files.push(entry);
        }
    }

    Ok(files)
}

/// Organize a file by its extension.
/// E.g., moving `document.pdf` to `<input_dir>/by_extension/pdf/document.pdf`
fn organize_by_extension(
    entry: &DirEntry,
    root_dir: &Path,
    dry_run: bool,
) -> Result<(), Box<dyn Error>> {
    let path = entry.path();
    if let Some(ext_os) = path.extension() {
        let extension = ext_os.to_string_lossy();
        let target_dir = root_dir.join("by_extension").join(extension.to_lowercase());

        if !dry_run {
            fs::create_dir_all(&target_dir)?;
            let target_path = target_dir.join(
                path.file_name().ok_or("No filename found")?
            );
            fs::rename(&path, &target_path)?;
        } else {
            println!(
                "[DRY RUN] Would move {:?} to {:?}",
                path.file_name(), target_dir
            );
        }
    } else {
        // Files without extension go into a "no_ext" folder
        let target_dir = root_dir.join("by_extension").join("no_ext");

        if !dry_run {
            fs::create_dir_all(&target_dir)?;
            let target_path = target_dir.join(
                path.file_name().ok_or("No filename found")?
            );
            fs::rename(&path, &target_path)?;
        } else {
            println!(
                "[DRY RUN] Would move {:?} to {:?}",
                path.file_name(), target_dir
            );
        }
    }
    Ok(())
}

/// Organize a file by its creation (or last modification) date.
///
/// Creates folders like: `<input_dir>/by_date/2024-12-28/filename.ext`
fn organize_by_date(
    entry: &DirEntry,
    root_dir: &Path,
    dry_run: bool,
) -> Result<(), Box<dyn Error>> {
    let path = entry.path();
    let metadata = fs::metadata(&path)?;

    // Try creation time first; if unavailable/fails, fallback to modification time
    let file_time = metadata
        .created()
        .or_else(|_| metadata.modified())?;
    let datetime: DateTime<Local> = file_time.into();

    // Format date (e.g., "2024-12-28")
    let date_str = datetime.format("%Y-%m-%d").to_string();
    let target_dir = root_dir.join("by_date").join(date_str);

    if !dry_run {
        fs::create_dir_all(&target_dir)?;
        let target_path = target_dir.join(
            path.file_name().ok_or("No filename found")?
        );
        fs::rename(&path, &target_path)?;
    } else {
        println!(
            "[DRY RUN] Would move {:?} to {:?}",
            path.file_name(),
            target_dir
        );
    }

    Ok(())
}

/// Organize files by "small", "medium", or "large" size categories.
/// E.g., "small" < 1 MB, "medium" < 100 MB, "large" >= 100 MB
fn organize_by_size(
    entry: &DirEntry,
    root_dir: &Path,
    dry_run: bool,
) -> Result<(), Box<dyn Error>> {
    let path = entry.path();
    let metadata = fs::metadata(&path)?;
    let file_size = metadata.len(); // in bytes

    // Basic categorization
    let size_label = if file_size < 1_000_000 {
        "small"
    } else if file_size < 100_000_000 {
        "medium"
    } else {
        "large"
    };

    let target_dir = root_dir.join("by_size").join(size_label);;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}

    if !dry_run {
        fs::create_dir_all(&target_dir)?;
        let target_path = target_dir.join(
            path.file_name().ok_or("No filename found")?
        );
        fs::rename(&path, &target_path)?;
    } else {
        println!(
            "[DRY RUN] Would move {:?} ({}) to {:?}",
            path.file_name(),
            size_label,
            target_dir
        );
    }

    Ok(())
}
