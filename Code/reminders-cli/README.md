# Reminders CLI

A simple interactive **menu-driven** reminders application written in Rust. It allows you to:

- **Add** new reminders with optional due dates.
- **List** all reminders (showing completed or incomplete statuses).
- **Mark** reminders as completed.
- **Remove** specific reminders.
- **Clear** all completed reminders in one shot.

This is part of the [**dunamismax/Rust**](https://github.com/dunamismax/Rust) repository, and lives under `Code/reminders-cli`.

---

## Table of Contents

- [Reminders CLI](#reminders-cli)
  - [Table of Contents](#table-of-contents)
  - [Getting Started](#getting-started)
  - [Features](#features)
  - [Installation](#installation)
  - [Usage](#usage)
  - [Example](#example)
  - [Project Structure](#project-structure)
  - [Contributing](#contributing)
  - [License](#license)

---

## Getting Started

This CLI tool is designed to help you manage your reminders in a straightforward, interactive manner—no need to memorize command-line arguments. Just run it, follow the on-screen menu, and keep track of your tasks hassle-free.

---

## Features

- **Menu-Driven UX**: An ASCII-based menu that prompts you step-by-step.
- **Add Reminders**: Include an optional due date/time in various date formats.
- **Mark as Completed**: Flip a reminder’s status when it’s done.
- **Remove & Clear**: Remove single reminders or clear all completed ones.
- **Colored Output**: Uses ANSI colors (via [`colored`](https://crates.io/crates/colored)) for better readability.
- **JSON Storage**: Persist your reminders to a `~/.reminders.json` file across sessions.

---

## Installation

1. **Clone** or **download** [this repository](https://github.com/dunamismax/Rust) (and navigate to `Code/reminders-cli`).
2. **Install Rust** if you haven’t already:

   ```bash
   curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
   ```

3. **Build** the application from the `reminders-cli` directory:

   ```bash
   cargo build
   ```

> **Note**: Ensure you have an up-to-date Rust toolchain (e.g., via `rustup update`).

---

## Usage

To run the interactive CLI:

```bash
cargo run
```

You should see something like:

```bash
                        _             _                          _  _
                       (_)           | |                        | |(_)
 _ __   ___  _ __ ___   _  _ __    __| |  ___  _ __  ___    ___ | | _
| '__| / _ \| '_ ` _ \ | || '_ \  / _` | / _ \| '__|/ __|  / __|| || |
| |   |  __/| | | | | || || | | || (_| ||  __/| |   \__ \ | (__ | || |
|_|    \___||_| |_| |_||_||_| |_| \__,_| \___||_|   |___/  \___||_||_|


===== MAIN MENU =====
1) List all reminders
2) Add a new reminder
3) Mark a reminder as completed
4) Remove a reminder
5) Clear all completed reminders
6) Quit
=====================
Enter a choice (1-6):
```

From here, follow the prompts to add/view/remove/mark reminders.

---

## Example

1. **Add a Reminder**
   - Choose **Option 2** in the menu.
   - Enter a **title** (e.g. "Buy groceries").
   - (Optional) Enter a **due date/time** (e.g. `2025-05-01 15:00`).
   - The app confirms that the reminder was added and saves it to disk.

2. **List Reminders**
   - Choose **Option 1** to see all reminders, along with IDs and statuses.

3. **Mark a Reminder Done**
   - Choose **Option 3** and provide the **ID** of the reminder you want to mark completed.

4. **Remove a Reminder**
   - Choose **Option 4** and provide the **ID** of the reminder to remove.

5. **Clear Completed**
   - Choose **Option 5** to remove all reminders that are marked as done.

---

## Project Structure

```bash
reminders-cli
├── Cargo.toml        # Project metadata & dependencies
├── Cargo.lock        # Version-lock file
└── src
    └── main.rs       # Entry point & core logic
```

> The application persists data in `~/.reminders.json` by default.

---

## Contributing

Contributions are welcome! If you want to make improvements:

1. **Fork** this repository.
2. **Create** a new branch for your feature or bug fix.
3. **Submit** a Pull Request (PR) back to [dunamismax/Rust](https://github.com/dunamismax/Rust).

We appreciate your help in making this CLI better!

---

## License

[MIT License](../LICENSE) © 2024 [dunamismax](https://github.com/dunamismax)

Feel free to modify and distribute under the terms of the MIT license.
