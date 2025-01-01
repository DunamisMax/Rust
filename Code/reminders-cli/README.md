# reminders-cli

A **Rust-based** CLI/TUI application that helps you manage tasks and reminders with optional due dates. **`reminders-cli`** is part of the larger [Rust](https://github.com/dunamismax/Rust) repository maintained by [dunamismax](https://dunamismax.com).

---

## Table of Contents

- [reminders-cli](#reminders-cli)
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

**`reminders-cli`** is a TUI (terminal user interface) application that allows you to track tasks (“reminders”) from your terminal. You can add tasks, specify optional due dates, mark tasks as done, remove tasks, and clear all completed tasks — all in an interactive interface. The application leverages:

- **crossterm** for terminal interaction
- **tui** for rendering text-based user interfaces
- **serde** / **serde_json** for data persistence
- **chrono** for date/time parsing and formatting

---

## Features

1. **Add & Manage Reminders**
   Create new reminders with a title and optional due date, list them on-screen, and track their status.

2. **Mark as Completed**
   Easily mark reminders as completed to keep track of finished tasks.

3. **Remove & Clear**
   Remove a single reminder or clear all completed reminders in one go.

4. **Interactive TUI**
   Use arrow keys or **j** / **k** to navigate the reminder list, **a** to add tasks, **d** to mark done, **r** to remove, **c** to clear, and **q** to quit.

5. **Cross-Platform Friendly**
   Uses terminal-based libraries that work on Linux, macOS, and Windows (although some filesystem or display variations may apply).

---

## Prerequisites

Before installing and running **`reminders-cli`**, ensure you have:

1. **Rust & Cargo** (latest stable version recommended)
   Installation instructions are available at [rustup.rs](https://rustup.rs).

2. (Optional) **Git** if you plan on cloning the entire repository.

---

## Installation

1. **Clone** the parent repository:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   cd Rust/Code/reminders-cli
   ```

2. **Build** the application using [Cargo](https://doc.rust-lang.org/cargo/):

   ```bash
   cargo build --release
   ```

3. **(Optional)** Adjust dependencies or features by modifying the `[dependencies]` section in the `Cargo.toml` file.

---

## Usage

1. **Run** the application in release mode:

   ```bash
   cargo run --release
   ```

   or run the compiled binary directly:

   ```bash
   ./target/release/reminders-cli
   ```

2. **Controls**:
   - **q**: Quit the application.
   - **j** / **Down Arrow**: Move the selection cursor down.
   - **k** / **Up Arrow**: Move the selection cursor up.
   - **a**: Add a new reminder (prompts for title and optional due date).
   - **d**: Mark the selected reminder as done.
   - **r**: Remove the currently selected reminder.
   - **c**: Clear all completed reminders.
   - **Esc**: Cancel adding a new reminder (while in input mode).

3. **Due Date Format**:
   The app attempts to parse dates in `YYYY-mm-dd HH:MM` format. If parsing fails, your new reminder is stored without a due date.

---

## Examples

1. **Start in Normal Mode**:

   ```bash
   cargo run --release
   ```

   Follow on-screen prompts to add and manage reminders.

2. **Verbose Output**:

   ```bash
   cargo run --release -- --verbose
   ```

   This prints some additional logs when starting up.

3. **Adding a Reminder**:
   - Press **a**.
   - Enter a title (e.g., "Buy groceries"), press Enter.
   - Enter a due date (optional, e.g., "2024-01-01 08:00"), then press Enter to confirm or skip.

4. **Marking as Done**:
   - Navigate to the reminder using **j** / **k**.
   - Press **d** to mark the selected reminder as completed.

---

## Project Structure

The **`reminders-cli`** folder is one of several standalone Rust applications located within [Code/](https://github.com/dunamismax/Rust/tree/main/Code) in the main [Rust repository](https://github.com/dunamismax/Rust). The overall layout is:

```bash
Rust/
├─ Code/
│  ├─ hello-world-cli/
│  ├─ net-commander/
│  ├─ reminders-cli/
│  │  ├─ src/
│  │  ├─ Cargo.toml
│  │  └─ README.md  <-- You are here!
│  ├─ ...
│  └─ rust-top/
├─ Wiki/
│  ├─ ...
├─ LICENSE
└─ README.md         <-- main repository README
```

Each subfolder under **Code/** is a standalone Cargo project with its own `Cargo.toml` and `README.md`.

---

## Contributing

Contributions are welcome! If you encounter a bug or have a feature request, please open an [issue](https://github.com/dunamismax/Rust/issues) or submit a [pull request](https://github.com/dunamismax/Rust/pulls) in the main repository. Please follow the project’s coding style and guidelines.

---

## License

This project is licensed under the [MIT License](https://github.com/dunamismax/Rust/blob/main/LICENSE).
See the [`LICENSE` file](https://github.com/dunamismax/Rust/blob/main/LICENSE) in the root of the main repository for details.

---

## Contact

- **Author**: [dunamismax](https://dunamismax.com)
- **Email**: [dunamismax@tutamail.com](mailto:dunamismax@tutamail.com)
- **Website/Blog**: [dunamismax.com](https://dunamismax.com)

For questions about **`reminders-cli`** or the [Rust repository](https://github.com/dunamismax/Rust), feel free to reach out or open an issue!
