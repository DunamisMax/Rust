use chrono::{DateTime, Local};
use rayon::prelude::*;
use std::error::Error;
use std::fs::{self, DirEntry};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

// Add the necessary imports for colors and random selection
use colored::{Color, Colorize};
use rand::Rng;

/// A type alias for a `Send + Sync + 'static` error, required by Rayon
type DynError = Box<dyn Error + Send + Sync + 'static>;

fn main() -> Result<(), DynError> {
    // Print the welcome banner in a random color
    print_welcome_banner();

    loop {
        println!("\n===== File Commander: Swiss Army Knife =====");
        println!("1) Organize Files (by extension, date, size)");
        println!("2) Copy a File");
        println!("3) Move/Rename a File");
        println!("4) Delete a File");
        println!("5) Exit\n");

        // Prompt user for choice
        let choice = prompt("Select an option: ")?;

        match choice.trim() {
            "1" => organize_files_interactive()?,
            "2" => copy_file_interactive()?,
            "3" => move_or_rename_file_interactive()?,
            "4" => delete_file_interactive()?,
            "5" => {
                println!("Exiting File Commander. Goodbye!");
                break;
            }
            _ => {
                println!("Invalid option. Please try again.");
            }
        }
    }
    Ok(())
}

/// Prints a banner with ASCII art in a random color.
fn print_welcome_banner() {
    let banner = r#"
  __  _  _                                                              _
 / _|(_)| |                                                            | |
| |_  _ | |  ___    ___   ___   _ __ ___   _ __ ___    __ _  _ __    __| |  ___  _ __
|  _|| || | / _ \  / __| / _ \ | '_ ` _ \ | '_ ` _ \  / _` || '_ \  / _` | / _ \| '__|
| |  | || ||  __/ | (__ | (_) || | | | | || | | | | || (_| || | | || (_| ||  __/| |
|_|  |_||_| \___|  \___| \___/ |_| |_| |_||_| |_| |_| \__,_||_| |_| \__,_| \___||_|
    "#;

    // Print the banner and welcome message in random color
    cprintln(banner);
    cprintln("Welcome to the file-commander CLI!\n");
}

/// A small helper function that prints text in a randomly chosen color.
fn cprintln(text: &str) {
    let color = random_color();
    // `.color(color)` will apply the color to the entire multiline string
    println!("{}", text.color(color));
}

/// Prompts user for text input, returning a `String`.
fn prompt(message: &str) -> Result<String, DynError> {
    print!("{message}");
    // Make sure the prompt is displayed before reading input
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

/* ---------------------------------------------------------------------------
Interactive Menu: Organize Files
--------------------------------------------------------------------------- */

fn organize_files_interactive() -> Result<(), DynError> {
    println!("\n=== Organize Files ===");

    // Ask for input directory
    let input_dir = prompt("Enter the path of the directory to organize: ")?;
    let input_dir = PathBuf::from(input_dir.trim());

    // Check if directory exists
    if !input_dir.is_dir() {
        println!("Error: {:?} is not a valid directory.", input_dir);
        return Ok(());
    }

    // Ask user which method of organization
    println!("Organization Methods:");
    println!("1) By Extension");
    println!("2) By Date");
    println!("3) By Size");
    let method = prompt("Select a method (1/2/3): ")?;

    // Ask if it's a dry run
    let dry_run_str = prompt("Dry Run? (y/n): ")?;
    let dry_run = matches_yes(&dry_run_str);

    let files = collect_files(&input_dir)?;

    match method.trim() {
        "1" => {
            // By extension
            files
                .par_iter()
                .try_for_each(|entry| organize_by_extension(entry, &input_dir, dry_run))?;
            println!("Organized by extension!");
        }
        "2" => {
            // By date
            files
                .par_iter()
                .try_for_each(|entry| organize_by_date(entry, &input_dir, dry_run))?;
            println!("Organized by date!");
        }
        "3" => {
            // By size
            files
                .par_iter()
                .try_for_each(|entry| organize_by_size(entry, &input_dir, dry_run))?;
            println!("Organized by size!");
        }
        _ => {
            println!("Invalid method chosen. Returning to main menu.");
        }
    }

    Ok(())
}

/* ---------------------------------------------------------------------------
Interactive Menu: Copy File
--------------------------------------------------------------------------- */

fn copy_file_interactive() -> Result<(), DynError> {
    println!("\n=== Copy a File ===");
    let source_path = prompt("Enter the source file path: ")?;
    let source_path = PathBuf::from(source_path.trim());

    if !source_path.is_file() {
        println!("Error: {:?} is not a valid file.", source_path);
        return Ok(());
    }

    let dest_path = prompt("Enter the destination path (including filename): ")?;
    let dest_path = PathBuf::from(dest_path.trim());

    match fs::copy(&source_path, &dest_path) {
        Ok(_) => {
            println!("Successfully copied {:?} to {:?}", source_path, dest_path);
        }
        Err(e) => {
            println!("Failed to copy file: {}", e);
        }
    }

    Ok(())
}

/* ---------------------------------------------------------------------------
Interactive Menu: Move or Rename File
--------------------------------------------------------------------------- */

fn move_or_rename_file_interactive() -> Result<(), DynError> {
    println!("\n=== Move/Rename a File ===");
    let old_path = prompt("Enter the current file path: ")?;
    let old_path = PathBuf::from(old_path.trim());

    if !old_path.exists() {
        println!("Error: {:?} does not exist.", old_path);
        return Ok(());
    }

    let new_path = prompt("Enter the new file path/filename: ")?;
    let new_path = PathBuf::from(new_path.trim());

    match fs::rename(&old_path, &new_path) {
        Ok(_) => {
            println!(
                "Successfully moved/renamed {:?} to {:?}",
                old_path, new_path
            );
        }
        Err(e) => {
            println!("Failed to move/rename file: {}", e);
        }
    }

    Ok(())
}

/* ---------------------------------------------------------------------------
Interactive Menu: Delete File
--------------------------------------------------------------------------- */

fn delete_file_interactive() -> Result<(), DynError> {
    println!("\n=== Delete a File ===");
    let file_path = prompt("Enter the file path to delete: ")?;
    let file_path = PathBuf::from(file_path.trim());

    if !file_path.exists() {
        println!("Error: {:?} does not exist.", file_path);
        return Ok(());
    }

    // Confirm
    let confirm = prompt(&format!(
        "Are you sure you want to delete {:?}? (y/n): ",
        file_path
    ))?;
    if matches_yes(&confirm) {
        // If directory, remove_dir_all; if file, remove_file
        if file_path.is_dir() {
            match fs::remove_dir_all(&file_path) {
                Ok(_) => println!("{:?} directory deleted.", file_path),
                Err(e) => println!("Failed to delete directory: {}", e),
            }
        } else {
            match fs::remove_file(&file_path) {
                Ok(_) => println!("{:?} file deleted.", file_path),
                Err(e) => println!("Failed to delete file: {}", e),
            }
        }
    } else {
        println!("Delete action canceled.");
    }

    Ok(())
}

/* ---------------------------------------------------------------------------
Common Helpers & Existing Logic
--------------------------------------------------------------------------- */

/// Recursively collects files (not directories) from the given directory.
fn collect_files(dir: &Path) -> Result<Vec<DirEntry>, io::Error> {
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
fn organize_by_extension(entry: &DirEntry, root_dir: &Path, dry_run: bool) -> Result<(), DynError> {
    let path = entry.path();
    if let Some(ext_os) = path.extension() {
        let extension = ext_os.to_string_lossy();
        let target_dir = root_dir.join("by_extension").join(extension.to_lowercase());

        if !dry_run {
            fs::create_dir_all(&target_dir)?;
            let target_path = target_dir.join(path.file_name().ok_or("No filename found")?);
            fs::rename(&path, &target_path)?;
        } else {
            println!(
                "[DRY RUN] Would move {:?} to {:?}",
                path.file_name(),
                target_dir
            );
        }
    } else {
        // Files without extension go into a "no_ext" folder
        let target_dir = root_dir.join("by_extension").join("no_ext");

        if !dry_run {
            fs::create_dir_all(&target_dir)?;
            let target_path = target_dir.join(path.file_name().ok_or("No filename found")?);
            fs::rename(&path, &target_path)?;
        } else {
            println!(
                "[DRY RUN] Would move {:?} to {:?}",
                path.file_name(),
                target_dir
            );
        }
    }
    Ok(())
}

/// Organize a file by its creation (or last modification) date.
/// E.g., `<input_dir>/by_date/2024-12-28/filename.ext`
fn organize_by_date(entry: &DirEntry, root_dir: &Path, dry_run: bool) -> Result<(), DynError> {
    let path = entry.path();
    let metadata = fs::metadata(&path)?;

    // Try creation time first; fallback to modification time
    let file_time = metadata.created().or_else(|_| metadata.modified())?;
    let datetime: DateTime<Local> = file_time.into();
    let date_str = datetime.format("%Y-%m-%d").to_string();

    let target_dir = root_dir.join("by_date").join(date_str);

    if !dry_run {
        fs::create_dir_all(&target_dir)?;
        let target_path = target_dir.join(path.file_name().ok_or("No filename found")?);
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
fn organize_by_size(entry: &DirEntry, root_dir: &Path, dry_run: bool) -> Result<(), DynError> {
    let path = entry.path();
    let metadata = fs::metadata(&path)?;
    let file_size = metadata.len(); // in bytes

    let size_label = if file_size < 1_000_000 {
        "small"
    } else if file_size < 100_000_000 {
        "medium"
    } else {
        "large"
    };

    let target_dir = root_dir.join("by_size").join(size_label);

    if !dry_run {
        fs::create_dir_all(&target_dir)?;
        let target_path = target_dir.join(path.file_name().ok_or("No filename found")?);
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

/// Helper to interpret "y"/"yes" input as true, everything else as false.
fn matches_yes(input: &str) -> bool {
    let s = input.trim().to_lowercase();
    s == "y" || s == "yes"
}

/// Returns a random color from the `colored` crate (standard 8 + bright 8).
fn random_color() -> Color {
    let colors = [
        // Standard colors
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::White,
        // Bright variants
        Color::BrightRed,
        Color::BrightGreen,
        Color::BrightYellow,
        Color::BrightBlue,
        Color::BrightMagenta,
        Color::BrightCyan,
        Color::BrightWhite,
    ];

    // Note: Black or BrightBlack is omitted to avoid invisible text on dark backgrounds
    let idx = rand::thread_rng().gen_range(0..colors.len());
    colors[idx]
}
