# Reminders CLI

A simple, cross-platform, command-line reminders application written in Rust.
It stores reminders in a JSON file in your home directory and lets you list, add, complete, and remove them from the terminal.

## Features

- **Add** reminders with or without a due date/time.
- **List** incomplete or **all** reminders.
- **Mark** reminders as done.
- **Remove** reminders permanently.
- **Clear** completed reminders in one command.
- **Local Time** parsing using [Chrono](https://crates.io/crates/chrono) (with support for multiple date/time formats).
- **JSON** storage in `~/.reminders.json`.

## Prerequisites

- **Rust** (1.64 or higher recommended, but it should work on earlier 1.x versions)
- **Cargo** (comes with the Rust toolchain)
- A basic understanding of command-line usage

## Installation

1. **Clone** the repository:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   cd /Rust/Code/reminders-cli
   ```

2. **Build and install** the CLI tool:

   ```bash
   cargo install --path .
   ```

   This will create a `reminders-app-cli` (or whatever your package name/binary is) in your Cargo bin directory (often `~/.cargo/bin/`).

3. **Verify** the installation:

   ```bash
   reminders-app-cli --help
   ```

## Usage

```bash
USAGE:
    reminders-app-cli <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information

SUBCOMMANDS:
    add              Add a new reminder
    done             Mark a reminder as completed by its ID
    remove           Remove a reminder by its ID
    list             List reminders
    clear-completed  Clear all completed reminders
    help             Prints this message or the help of the given subcommand(s)
```

### Add a reminder

```bash
reminders-app-cli add "Buy milk" --due "2024-12-29 10:00"
```

- Creates a new reminder with ID `1` (if it’s the first reminder).
- Sets a due date of `Dec 29, 2024, 10:00 local time`.

### List reminders

```bash
reminders-app-cli list
```

- By default, only shows incomplete reminders.
- Use `--all` to see completed as well:

  ```bash
  reminders-app-cli list --all
  ```

### Mark a reminder as done

```bash
reminders-app-cli done 1
```

- Marks reminder with ID `1` as completed.

### Remove a reminder

```bash
reminders-app-cli remove 1
```

- Deletes the reminder with ID `1` from the file.

### Clear all completed reminders

```bash
reminders-app-cli clear-completed
```

- Removes every reminder that has already been completed.

## File Location & Data Format

Reminders are stored in a JSON file at `~/.reminders.json`. Example structure:

```json
[
  {
    "id": 1,
    "title": "Buy milk",
    "due": "2024-12-29T10:00:00-05:00",
    "completed": true
  },
  {
    "id": 2,
    "title": "Take out trash",
    "due": null,
    "completed": false
  }
]
```

Feel free to edit this file manually if needed, but be sure to keep valid JSON!

## Customization

- **File Location**: Adjust the `REMINDERS_FILE` constant (and logic in `get_reminders_file_path`) if you’d like to store reminders somewhere else.
- **Date Parsing**: Our `parse_datetime` function accepts multiple formats. Add or remove formats in the code as needed.
- **Dependencies**:
    - [Clap](https://crates.io/crates/clap) for CLI parsing
    - [Chrono](https://crates.io/crates/chrono) for date/time
    - [Serde + Serde JSON](https://serde.rs/) for JSON
    - [Anyhow](https://crates.io/crates/anyhow) for error handling
    - [Dirs](https://crates.io/crates/dirs) for locating home directory

## Contributing

1. Fork the repository and clone locally.
2. Create a feature branch for your work: `git checkout -b feature/<feature-name>`.
3. Commit your changes and push to GitHub.
4. Create a Pull Request (PR) and we’ll review it.

## License

MIT License. See [`LICENSE`](LICENSE) for details.

## Author

- [dunamismax](https://github.com/dunamismax)

---

**Enjoy using this Reminders CLI App!** If you have questions or ideas for improvements, please open an issue or submit a PR.
