```markdown
# File Commander

A Rust-based command-line application that organizes files by extension, date, or size, and also provides convenient commands to copy, move/rename, or delete files. This project is part of the [dunamismax/Rust](https://github.com/dunamismax/Rust) repository, located in the `Rust/Code/file-commander` subdirectory.

---

## Table of Contents

- [File Commander](#file-commander)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Usage](#usage)
  - [Examples](#examples)
  - [Project Structure](#project-structure)
  - [Contributing](#contributing)
  - [Contact](#contact)

---

## Features

- **Organize files** by:
  - **Extension** (e.g., `.png`, `.pdf`)
  - **Date** (creation or last modified date)
  - **Size** (small, medium, large)
- **Copy** any file to a new location.
- **Move/Rename** a file to a new path or name.
- **Delete** a file or folder with confirmation.
- **Dry run** option for organizing so you can see changes before applying them.
- **Parallel processing** using **Rayon** for faster organizing.
- **Colorful CLI banner** via the **colored** crate, displayed in a random color on each run.

---

## Prerequisites

1. **Rust & Cargo**
   Make sure you have Rust (and Cargo) installed. You can get them from [rustup](https://www.rust-lang.org/tools/install).

2. **Operating System**
   Works on Windows, macOS, or Linux—any system that supports the Rust toolchain.

No external API keys or environment variables are required.

---

## Installation

1. **Clone the repository**:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   ```

2. **Navigate to the `file-commander` directory**:

   ```bash
   cd Rust/Code/file-commander
   ```

3. **Build and run**:

   ```bash
   cargo build
   cargo run
   ```

---

## Usage

1. **Launch the CLI**:

   ```bash
   cargo run
   ```

   Upon starting, you’ll see the main menu with the following options:

   ```bash
   1) Organize Files (by extension, date, size)
   2) Copy a File
   3) Move/Rename a File
   4) Delete a File
   5) Exit
   ```

2. **Organize Files**:
   - Choose **1** in the main menu.
   - Specify a directory to organize.
   - Choose an organization method (Extension, Date, or Size).
   - Optionally, enable “Dry Run” to simulate changes before actually moving files.
   - Files will be reorganized into folders named by their extension, date, or size category.

3. **Copy a File**:
   - Choose **2**.
   - Enter the source file path.
   - Enter the destination path (including the new filename).

4. **Move/Rename a File**:
   - Choose **3**.
   - Enter the current file path.
   - Enter the new path/filename.

5. **Delete a File**:
   - Choose **4**.
   - Enter the file path or directory path to delete.
   - Confirm the action.

6. **Exit**:
   - Choose **5** to exit the application.

---

## Examples

```bash
# Start the CLI:
cargo run

# When prompted:
#   Select an option: 1
#   Enter the path of the directory to organize: ./my_documents
#   Organization Methods: 1) By Extension, 2) By Date, 3) By Size
#   Select a method: 1
#   Dry Run? (y/n): n
# Files in ./my_documents will be reorganized into subfolders by extension.
```

```bash
# Copy a file:
cargo run
#   Select an option: 2
#   Enter the source file path: ./my_documents/readme.txt
#   Enter the destination path (including filename): ./backup/readme_backup.txt
```

---

## Project Structure

```bash
Rust
└── Code
    ├── file-commander  <-- You are here
    │   ├── Cargo.toml
    │   └── src
    │       └── main.rs
    └── <other projects>
```

---

## Contributing

Contributions are welcome! Please open an [issue](https://github.com/dunamismax/Rust/issues) or submit a pull request for any bug fixes or new features.

1. Fork the repository.
2. Create a new branch: `git checkout -b feature/my-feature`.
3. Commit your changes: `git commit -m "Add some feature"`.
4. Push to your fork: `git push origin feature/my-feature`.
5. Open a Pull Request.

---

## Contact

Maintained by [dunamismax.com](https://github.com/dunamismax).

For any inquiries, please reach out via [email](mailto:dunamismax@tutamail.com).
