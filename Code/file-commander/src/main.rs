//! A File Commander CLI with directory navigation, file operations, and theming.
//! Demonstrates:
//! - crossterm for terminal I/O, coloring, and screen management
//! - tokio for async runtime
//! - parallel file organizing logic (Rayon)
//! - directory tree view, directory info, create/copy/move/rename/delete/duplicate, etc.

use chrono::{DateTime, Local};
use rayon::prelude::*;
use std::error::Error;
use std::fs::{self, DirEntry};
use std::io::{self, Write};
use std::os::unix::fs::MetadataExt; // for ownership on Unix
use std::path::{Path, PathBuf};

// crossterm for all terminal I/O and styling
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Stylize},
    terminal::{Clear, ClearType},
};

/// A type alias for a `Send + Sync + 'static` error, required by Rayon
type DynError = Box<dyn Error + Send + Sync + 'static>;

/// Asynchronous entry point using Tokio.
#[tokio::main]
async fn main() -> Result<(), DynError> {
    clear_screen()?;
    print_welcome_banner()?;

    // We'll maintain a "current directory" context.
    let mut current_dir = std::env::current_dir()?;

    loop {
        print_breadcrumb(&current_dir)?;
        print!("Select an option:\r\n");
        print!("  1) Change directory (cd)\r\n");
        print!("  2) List contents (ls)\r\n");
        print!("  3) Show directory tree (tree)\r\n");
        print!("  4) Show directory info\r\n");
        print!("  5) Create file (touch)\r\n");
        print!("  6) Create directory (mkdir)\r\n");
        print!("  7) Copy file/directory (cp)\r\n");
        print!("  8) Move/rename file/directory (mv)\r\n");
        print!("  9) Delete file/directory (rm)\r\n");
        print!(" 10) Duplicate file/directory\r\n");
        print!(" 11) Organize files (by extension/date/size)\r\n");
        print!(" 12) Exit\r\n\r\n");

        let choice = prompt("Enter choice: ")?;

        match choice.trim() {
            "1" => change_directory(&mut current_dir)?,
            "2" => list_contents(&current_dir)?,
            "3" => show_tree_view(&current_dir)?,
            "4" => show_directory_info(&current_dir)?,
            "5" => create_file(&current_dir)?,
            "6" => create_directory(&current_dir)?,
            "7" => copy_interactive()?,
            "8" => move_or_rename_interactive()?,
            "9" => delete_interactive()?,
            "10" => duplicate_interactive()?,
            "11" => organize_files_interactive()?,
            "12" => {
                print!("Exiting File Commander. Goodbye!\r\n");
                break;
            }
            _ => {
                print!("Invalid option. Please try again.\r\n");
            }
        }
    }

    Ok(())
}

/* ---------------------------------------------------------------------------
   1) Clear screen & welcome banner (required structure)
---------------------------------------------------------------------------*/

/// Clears the terminal screen for a clean start using crossterm.
fn clear_screen() -> Result<(), DynError> {
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    Ok(())
}

/// Prints a banner with ASCII art in a consistent theme color.
fn print_welcome_banner() -> Result<(), DynError> {
    let banner = r#"
  __  _  _                                                              _
 / _|(_)| |                                                            | |
| |_  _ | |  ___    ___   ___   _ __ ___   _ __ ___    __ _  _ __    __| |  ___  _ __
|  _|| || | / _ \  / __| / _ \ | '_ ` _ \ | '_ ` _ \  / _` || '_ \  / _` | / _ \| '__|
| |  | || ||  __/ | (__ | (_) || | | | | || | | | | || (_| || | | || (_| ||  __/| |
|_|  |_||_| \___|  \___| \___/ |_| |_| |_||_| |_| |_| \__,_||_| |_| \__,_| \___||_|
    "#;

    // Print banner in, say, Cyan color
    themed_print(banner, Color::Cyan);
    themed_print("Welcome to the File Commander CLI!\r\n", Color::Cyan);
    Ok(())
}

/* ---------------------------------------------------------------------------
   2) Theming / Printing Helpers
---------------------------------------------------------------------------*/

/// Print text in a given color, followed by "\r\n" at the end.
fn themed_print(text: &str, color: Color) {
    print!("{}\r\n", text.with(color));
}

/// Prompts the user for text input, returning a `String`.
fn prompt(message: &str) -> Result<String, DynError> {
    print!("{message}");
    io::stdout().flush()?; // ensure prompt is shown before input

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}

/// Print a "breadcrumb" line that shows the current directory path.
fn print_breadcrumb(current_dir: &Path) -> Result<(), DynError> {
    themed_print(
        &format!("\r\n== Current Directory: {} ==", current_dir.display()),
        Color::Magenta,
    );
    Ok(())
}

/* ---------------------------------------------------------------------------
   3) Directory Navigation
---------------------------------------------------------------------------*/

/// Change directory (cd).
fn change_directory(current_dir: &mut PathBuf) -> Result<(), DynError> {
    let new_dir = prompt("Enter path to change directory: ")?;
    let new_dir = new_dir.trim();
    let target = if new_dir.starts_with('/') {
        // Absolute path
        PathBuf::from(new_dir)
    } else {
        // Relative path
        current_dir.join(new_dir)
    };

    if target.is_dir() {
        *current_dir = target.canonicalize()?;
        print!("Directory changed to {:?}\r\n", current_dir);
    } else {
        print!("Error: {:?} is not a valid directory.\r\n", target);
    }
    Ok(())
}

/// List directory contents, similar to `ls`.
fn list_contents(current_dir: &Path) -> Result<(), DynError> {
    let show_hidden = prompt("Show hidden files? (y/n): ")?;
    let show_hidden = matches_yes(&show_hidden);

    let entries = fs::read_dir(current_dir)?;
    print!("Contents of {:?}:\r\n", current_dir);

    // Flatten the iterator so we only get valid entries, ignoring errors
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        if !show_hidden && file_name_str.starts_with('.') {
            // Skip hidden files
            continue;
        }
        print!("  {}\r\n", file_name_str);
    }
    Ok(())
}

/* ---------------------------------------------------------------------------
   4) Tree View
---------------------------------------------------------------------------*/

/// Show all files/folders within the specified directory in a tree view.
fn show_tree_view(current_dir: &Path) -> Result<(), DynError> {
    let dir_input = prompt(&format!(
        "Enter directory path for tree view (default: {}): ",
        current_dir.display()
    ))?;
    let dir_path = if dir_input.trim().is_empty() {
        current_dir.to_path_buf()
    } else {
        PathBuf::from(dir_input.trim())
    };

    if !dir_path.is_dir() {
        print!("Error: {:?} is not a valid directory.\r\n", dir_path);
        return Ok(());
    }

    themed_print("=== Directory Tree View ===", Color::Green);
    print_directory_tree(&dir_path, 0)?;
    Ok(())
}

/// Recursively print a directory tree with indentation.
fn print_directory_tree(dir: &Path, level: usize) -> Result<(), DynError> {
    let indent = "  ".repeat(level);
    print!(
        "{}- {}\r\n",
        indent,
        dir.file_name().unwrap_or_default().to_string_lossy()
    );

    let entries = fs::read_dir(dir)?;
    let mut dirs = Vec::new();
    let mut files = Vec::new();

    // Flatten the iterator
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        } else {
            files.push(path);
        }
    }
    // Print subdirectories first
    for d in &dirs {
        print_directory_tree(d, level + 1)?;
    }
    // Print files
    for f in &files {
        let file_name = f.file_name().unwrap_or_default().to_string_lossy();
        print!("{}  * {}\r\n", "  ".repeat(level + 1), file_name);
    }

    Ok(())
}

/* ---------------------------------------------------------------------------
   5) Directory Info
---------------------------------------------------------------------------*/

/// Lists the size, ownership, and other relevant information about a directory.
fn show_directory_info(current_dir: &Path) -> Result<(), DynError> {
    let dir_input = prompt(&format!(
        "Enter directory path for info (default: {}): ",
        current_dir.display()
    ))?;
    let dir_path = if dir_input.trim().is_empty() {
        current_dir.to_path_buf()
    } else {
        PathBuf::from(dir_input.trim())
    };

    if !dir_path.is_dir() {
        print!("Error: {:?} is not a valid directory.\r\n", dir_path);
        return Ok(());
    }

    themed_print("=== Directory Info ===", Color::Blue);

    // We can compute total size by summing sizes of all files.
    let (total_size, file_count, dir_count) = compute_directory_stats(&dir_path)?;
    print!("Path: {}\r\n", dir_path.display());
    print!("Total size (bytes): {}\r\n", total_size);
    print!("Files: {}, Directories: {}\r\n", file_count, dir_count);

    // For ownership, we use Unix-specific metadata. On non-Unix systems, adapt as needed.
    let metadata = fs::metadata(&dir_path)?;
    #[cfg(unix)]
    {
        print!("Owner UID: {}\r\n", metadata.uid());
        print!("Owner GID: {}\r\n", metadata.gid());
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

/* ---------------------------------------------------------------------------
   6) File Creation / Directory Creation
---------------------------------------------------------------------------*/

/// Create a new file (touch).
fn create_file(current_dir: &Path) -> Result<(), DynError> {
    let filename = prompt("Enter name of file to create: ")?;
    let filename = filename.trim();
    if filename.is_empty() {
        print!("Aborted: no filename provided.\r\n");
        return Ok(());
    }

    let new_file_path = current_dir.join(filename);
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&new_file_path)
    {
        Ok(_) => {
            print!("File created at {:?}\r\n", new_file_path);
        }
        Err(e) => {
            print!("Could not create file: {}\r\n", e);
        }
    }

    Ok(())
}

/// Create a new directory (mkdir).
fn create_directory(current_dir: &Path) -> Result<(), DynError> {
    let dir_name = prompt("Enter name of directory to create: ")?;
    let dir_name = dir_name.trim();
    if dir_name.is_empty() {
        print!("Aborted: no directory name provided.\r\n");
        return Ok(());
    }

    let new_dir_path = current_dir.join(dir_name);
    match fs::create_dir(&new_dir_path) {
        Ok(_) => {
            print!("Directory created at {:?}\r\n", new_dir_path);
        }
        Err(e) => {
            print!("Could not create directory: {}\r\n", e);
        }
    }
    Ok(())
}

/* ---------------------------------------------------------------------------
   7) File/Directory Copy, Move, Delete, Duplicate
---------------------------------------------------------------------------*/

/// Copy file/directory (cp).
fn copy_interactive() -> Result<(), DynError> {
    let source = prompt("Enter source file/directory: ")?;
    let destination = prompt("Enter destination path: ")?;

    let source_path = PathBuf::from(source.trim());
    let destination_path = PathBuf::from(destination.trim());

    if !source_path.exists() {
        print!("Error: source {:?} does not exist.\r\n", source_path);
        return Ok(());
    }

    if source_path.is_file() {
        // Simple file copy
        match fs::copy(&source_path, &destination_path) {
            Ok(_) => print!("File copied successfully.\r\n"),
            Err(e) => print!("File copy failed: {}\r\n", e),
        }
    } else {
        // Directory copy
        copy_directory_recursive(&source_path, &destination_path)?;
        print!("Directory copied successfully.\r\n");
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

/// Move/rename file/directory (mv).
fn move_or_rename_interactive() -> Result<(), DynError> {
    let source = prompt("Enter source file/directory: ")?;
    let destination = prompt("Enter new path/filename: ")?;

    let source_path = PathBuf::from(source.trim());
    let destination_path = PathBuf::from(destination.trim());

    if !source_path.exists() {
        print!("Error: source {:?} does not exist.\r\n", source_path);
        return Ok(());
    }

    match fs::rename(&source_path, &destination_path) {
        Ok(_) => print!("Move/rename succeeded.\r\n"),
        Err(e) => print!("Move/rename failed: {}\r\n", e),
    }

    Ok(())
}

/// Delete file/directory (rm).
fn delete_interactive() -> Result<(), DynError> {
    let target = prompt("Enter file/directory to delete: ")?;
    let target_path = PathBuf::from(target.trim());

    if !target_path.exists() {
        print!("Error: {:?} does not exist.\r\n", target_path);
        return Ok(());
    }

    let confirm = prompt(&format!(
        "Are you sure you want to delete {:?}? (y/n): ",
        target_path
    ))?;
    if matches_yes(&confirm) {
        if target_path.is_dir() {
            match fs::remove_dir_all(&target_path) {
                Ok(_) => print!("Directory deleted.\r\n"),
                Err(e) => print!("Failed to delete directory: {}\r\n", e),
            }
        } else {
            match fs::remove_file(&target_path) {
                Ok(_) => print!("File deleted.\r\n"),
                Err(e) => print!("Failed to delete file: {}\r\n", e),
            }
        }
    } else {
        print!("Delete action canceled.\r\n");
    }
    Ok(())
}

/// Duplicate a file/directory quickly by adding `_copy` or similar suffix.
fn duplicate_interactive() -> Result<(), DynError> {
    let source = prompt("Enter file/directory to duplicate: ")?;
    let source_path = PathBuf::from(source.trim());

    if !source_path.exists() {
        print!("Error: {:?} does not exist.\r\n", source_path);
        return Ok(());
    }

    // Generate new name
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
    print!("Duplicate created at {:?}\r\n", duplicate_path);

    Ok(())
}

/* ---------------------------------------------------------------------------
   8) Organize Files
---------------------------------------------------------------------------*/

fn organize_files_interactive() -> Result<(), DynError> {
    print!("\r\n=== Organize Files ===\r\n");

    // Ask for input directory
    let input_dir = prompt("Enter the path of the directory to organize: ")?;
    let input_dir = PathBuf::from(input_dir.trim());

    // Check if directory exists
    if !input_dir.is_dir() {
        print!("Error: {:?} is not a valid directory.\r\n", input_dir);
        return Ok(());
    }

    // Ask user which method of organization
    print!("Organization Methods:\r\n");
    print!("  1) By Extension\r\n");
    print!("  2) By Date\r\n");
    print!("  3) By Size\r\n");
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
            print!("Organized by extension!\r\n");
        }
        "2" => {
            // By date
            files
                .par_iter()
                .try_for_each(|entry| organize_by_date(entry, &input_dir, dry_run))?;
            print!("Organized by date!\r\n");
        }
        "3" => {
            // By size
            files
                .par_iter()
                .try_for_each(|entry| organize_by_size(entry, &input_dir, dry_run))?;
            print!("Organized by size!\r\n");
        }
        _ => {
            print!("Invalid method chosen. Returning to main menu.\r\n");
        }
    }

    Ok(())
}

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

fn organize_by_extension(entry: &DirEntry, root_dir: &Path, dry_run: bool) -> Result<(), DynError> {
    let path = entry.path();
    if let Some(ext_os) = path.extension() {
        let extension = ext_os.to_string_lossy();
        let target_dir = root_dir.join("by_extension").join(extension.to_lowercase());
        move_file_or_dry_run(&path, &target_dir, dry_run)?;
    } else {
        // Files without extension go into a "no_ext" folder
        let target_dir = root_dir.join("by_extension").join("no_ext");
        move_file_or_dry_run(&path, &target_dir, dry_run)?;
    }
    Ok(())
}

fn organize_by_date(entry: &DirEntry, root_dir: &Path, dry_run: bool) -> Result<(), DynError> {
    let path = entry.path();
    let metadata = fs::metadata(&path)?;
    let file_time = metadata.created().or_else(|_| metadata.modified())?;
    let datetime: DateTime<Local> = file_time.into();
    let date_str = datetime.format("%Y-%m-%d").to_string();
    let target_dir = root_dir.join("by_date").join(date_str);
    move_file_or_dry_run(&path, &target_dir, dry_run)?;
    Ok(())
}

fn organize_by_size(entry: &DirEntry, root_dir: &Path, dry_run: bool) -> Result<(), DynError> {
    let path = entry.path();
    let metadata = fs::metadata(&path)?;
    let file_size = metadata.len();

    // E.g., "small" < 1 MB, "medium" < 100 MB, "large" >= 100 MB
    let size_label = if file_size < 1_000_000 {
        "small"
    } else if file_size < 100_000_000 {
        "medium"
    } else {
        "large"
    };

    let target_dir = root_dir.join("by_size").join(size_label);
    move_file_or_dry_run(&path, &target_dir, dry_run)?;
    Ok(())
}

/// Move file to target dir, or print dry-run message.
fn move_file_or_dry_run(path: &Path, target_dir: &Path, dry_run: bool) -> Result<(), DynError> {
    if !dry_run {
        fs::create_dir_all(target_dir)?;
        let target_path = target_dir.join(path.file_name().ok_or("No filename found")?);
        fs::rename(path, &target_path)?;
    } else {
        print!("[DRY RUN] Would move {:?} to {:?}\r\n", path, target_dir);
    }
    Ok(())
}

/* ---------------------------------------------------------------------------
   9) Misc Helpers
---------------------------------------------------------------------------*/

/// Helper to interpret "y"/"yes" input as true, everything else as false.
fn matches_yes(input: &str) -> bool {
    let s = input.trim().to_lowercase();
    s == "y" || s == "yes"
}
