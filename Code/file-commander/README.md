# File Organizer

A cross-platform command-line tool that organizes files in a target directory by **extension**, **creation/modification date**, or **file size**. This project leverages [Rayon](https://github.com/rayon-rs/rayon) for parallel processing, so it can handle large directories efficiently.

## Features

1. **Organize by Extension**
   Creates subdirectories by file extension, e.g. `by_extension/pdf/`, `by_extension/png/`, etc.

2. **Organize by Date**
   Uses file creation or last modification time to group files under `by_date/YYYY-MM-DD/`.

3. **Organize by Size**
   Categorizes files into `small` (<1 MB), `medium` (<100 MB), and `large` (≥100 MB) directories, e.g. `by_size/small/`, `by_size/medium/`, and `by_size/large/`.

4. **Dry-Run Mode**
   Preview the moves without actually performing them, ensuring that the user is aware of file changes before proceeding.

5. **Recursive**
   Recursively traverses subdirectories, bringing everything together under one structure (organized by your chosen method).

6. **Parallel Execution**
   Uses multi-threading (via Rayon) to speed up file operations, especially beneficial for large directories.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.60+ recommended)
- Cargo (installed alongside Rust)

### Installation

1. **Clone** this repository:

   ```bash
   git clone https://github.com/<YOUR_USERNAME>/<REPO_NAME>.git
   cd <REPO_NAME>
   ```

2. **Build** the application:

   ```bash
   cargo build --release
   ```

3. The compiled binary can be found in `target/release/file_organizer` (Linux/macOS) or `target\release\file_organizer.exe` (Windows).

### Usage

```bash
file_organizer [OPTIONS] <input_dir> <SUBCOMMAND>
```

**Subcommands**:

- `extension`
  Organize files by their extension.
- `date`
  Organize files by creation/last modified date.
- `size`
  Organize files into `small`, `medium`, and `large` folders.

**Options**:

- `--dry-run`
  Prints the moves that would be performed, without making any changes.

#### Examples

1. **Organize by extension**:

   ```bash
   file_organizer /path/to/your/folder extension
   ```

   This will create a `by_extension` folder inside `/path/to/your/folder` and subfolders named according to each file extension.

2. **Organize by date**:

   ```bash
   file_organizer --dry-run /path/to/your/folder date
   ```

   This will show which files *would* be moved to `by_date/YYYY-MM-DD` directories, but won’t actually move them. Remove `--dry-run` to execute.

3. **Organize by size**:

   ```bash
   file_organizer /path/to/your/folder size
   ```

   This will create a `by_size` directory in `/path/to/your/folder` containing `small`, `medium`, and `large` subfolders.

### Example File Tree

Below is how your folder might look after organizing by extension:

```bash
/path/to/your/folder
└── by_extension
    ├── pdf
    │   ├── Document1.pdf
    │   └── Document2.pdf
    ├── png
    │   ├── Image1.png
    │   └── Image2.png
    └── no_ext
        └── README
```

## Configuration & Customization

- **Log/Output**: Currently, progress and actions are only printed to stdout. Consider integrating a logger or custom progress bar for more advanced workflows.
- **File Collisions**: If the target file already exists, you may need to implement an additional rename or overwrite policy.
- **Filtering**: Add CLI flags to exclude certain file types, subdirectories, etc.
- **Undo or “Dry-Run”**: The `--dry-run` flag gives you a preview, but a more sophisticated rollback might be useful for production scenarios.

## Contributing

Contributions, issues, and feature requests are welcome! Feel free to reach out to discuss improvements or submit fixes.

## License

This project is available under the [MIT License](LICENSE).
Feel free to modify, distribute, or use it privately or commercially as permitted under the MIT terms.

---

Thank you for using **File Organizer**! If you find this tool helpful, consider giving the repository a star ⭐ on GitHub to show your support. Happy organizing!
