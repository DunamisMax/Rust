# file-commander

A **Rust-based** TUI application for managing files and directories, demonstrating various file operations (copy, move, delete, etc.) via **Tokio**, **Clap**, **crossterm**, and **tui**. **`file-commander`** is part of the larger [Rust](https://github.com/dunamismax/Rust) repository maintained by [dunamismax](https://dunamismax.com).

---

## Table of Contents

- [file-commander](#file-commander)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Features](#features)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Usage](#usage)
  - [Examples](#examples)
  - [Project Structure](#project-structure)
  - [Contributing](#contributing)
  - [License](#license)
  - [Contact](#contact)

---

## Overview

**`file-commander`** provides a “menu-driven” file management interface in the terminal. It leverages Rust’s async runtime (**Tokio**), the **Clap** CLI argument parser, **crossterm** for terminal interactions, and **tui** for building an interactive text-based user interface. You can perform operations such as:

- Changing directories
- Listing and displaying directory trees
- Creating, copying, moving, and deleting files
- Organizing files by extension, date, or size

By default, the app launches in a TUI that displays a menu with all available operations. The goal is to demonstrate Rust’s concurrency alongside straightforward file-system tasks in a user-friendly manner.

---

## Features

1. **TUI Menu Navigation**
   Use arrow keys to scroll through a list of file operations, and press **Enter** to select them.

2. **File & Directory Operations**
   - Create files/directories.
   - Copy, move/rename, or delete items (with basic prompts).
   - Duplicate an item quickly (appends `"_copy"`).

3. **Directory Tree View**
   Recursively displays all files/directories in a “tree” format.

4. **File Organizer**
   Automatically sorts files based on **extension**, **date**, or **size** into subdirectories (with an optional “dry-run” mode).

5. **Cross-Platform Compatibility**
   Runs on most operating systems, though certain filesystem details (e.g., UNIX owner/group IDs) may only be shown on Linux/Unix.

6. **Verbose Mode**
   Toggle additional logging with `--verbose`.

---

## Prerequisites

Before installing and running **`file-commander`**, ensure you have:

1. **Rust & Cargo**
   - [Install Rust](https://www.rust-lang.org/tools/install) (1.60+ recommended).
2. **Git** (optional)
   - Required if you plan to clone the entire [dunamismax/Rust](https://github.com/dunamismax/Rust) repository.

---

## Installation

1. **Clone** the parent repository:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   cd Rust/Code/file-commander
   ```

2. **Build** the application using Cargo:

   ```bash
   cargo build --release
   ```

3. **(Optional)** Update dependencies or feature flags in `Cargo.toml` if you want to tweak functionality (e.g., enable debug logs, etc.).

---

## Usage

1. **Run** the application:

   ```bash
   cargo run --release
   ```

   Or execute the compiled binary:

   ```bash
   ./target/release/file-commander
   ```

2. **CLI Arguments**:
   - `--verbose` (or `-v`): Display extra debug info.

3. **Controls**:
   - **Up/Down arrows**: Move cursor in the menu.
   - **Enter**: Select a menu item to execute.
   - **q** or **Ctrl+C**: Quit the application.

4. **Flow**:
   - After you run **`file-commander`**, you will see a menu with items like **1) Change directory**, **2) List contents**, **3) Show directory tree**, and so on.
   - Selecting an item often prompts you to input a path or confirm your intentions (e.g., “Are you sure you want to delete...?”).

---

## Examples

Here are a few usage examples:

1. **Verbose Mode**:

   ```bash
   cargo run --release -- --verbose
   ```

   This prints additional log messages in the terminal about ongoing operations.

2. **Organize Files**:
   - Select **“11) Organize files (by extension/date/size)”** from the TUI menu.
   - Provide the path to the directory you want to organize.
   - Choose the method of organization (extension, date, or size).
   - Decide whether to perform a dry run or actually move the files.

3. **Delete a Directory**:
   - Navigate to the **“9) Delete file/directory”** option.
   - Enter the directory path you want to remove.
   - Confirm the delete operation when prompted.

---

## Project Structure

Below is a snapshot of how **`file-commander`** fits into the [Rust](https://github.com/dunamismax/Rust) repository:

```bash
Rust/
├─ Code/
│  ├─ file-commander/
│  │  ├─ src/
│  │  │  └─ main.rs
│  │  ├─ Cargo.toml
│  │  └─ README.md         <-- You are here!
│  ├─ rust-top/
│  ├─ hello-world-cli/
│  ├─ ...
│
├─ Wiki/
│  ├─ ...
├─ LICENSE
└─ README.md               <-- main repository README
```

Each subdirectory under `Code/` contains its own Cargo project, letting you independently build or run each program.

---

## Contributing

Contributions are welcome! If you find a bug or have a feature request, please open an [issue](https://github.com/dunamismax/Rust/issues) or submit a [pull request](https://github.com/dunamismax/Rust/pulls) in the main repository. Make sure to follow the repository’s coding style and guidelines.

---

## License

This project is licensed under the [MIT License](https://github.com/dunamismax/Rust/blob/main/LICENSE). See the [`LICENSE` file](https://github.com/dunamismax/Rust/blob/main/LICENSE) in the root of the main repository for details.

---

## Contact

- **Author**: [dunamismax](https://dunamismax.com)
- **Email**: [dunamismax@tutamail.com](mailto:dunamismax@tutamail.com)
- **Website/Blog**: [dunamismax.com](https://dunamismax.com)

For questions about **`file-commander`** or any other projects in the [Rust repository](https://github.com/dunamismax/Rust), feel free to reach out or open an issue!
