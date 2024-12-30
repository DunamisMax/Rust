# reminders-cli

A command-line reminders application written in Rust, with persistent storage in the user's home directory. You can add, list, complete, and remove reminders—all within an interactive menu system.
This project is part of the [dunamismax/Rust](https://github.com/dunamismax/Rust) repository, located in the `Rust/Code/reminders-cli` subdirectory.

---

## Table of Contents

- [reminders-cli](#reminders-cli)
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

- **Interactive menu** for listing, adding, completing, and removing reminders.
- **Persistent storage** in `~/.reminders.json` (within the user's home directory).
- **Optional due dates** using various date/time formats.
- **Colored output** for a more engaging terminal experience.
- **Automatic ID generation** for new reminders.
- **Clear all completed reminders** in one quick step.

---

## Prerequisites

1. **Rust & Cargo**
   Ensure you have Rust (and Cargo) installed. You can install Rust using [rustup](https://www.rust-lang.org/tools/install).

No external API keys or services are required.

---

## Installation

1. **Clone the repository**:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   ```

2. **Navigate to the `reminders-cli` directory**:

   ```bash
   cd Rust/Code/reminders-cli
   ```

3. **Build and run**:

   ```bash
   cargo build
   cargo run
   ```

---

## Usage

Upon running `cargo run`, you will see a welcome banner followed by a menu with several options:

1. **List all reminders**
   Displays all reminders, including their ID, title, due date (if any), and completion status.

2. **Add a new reminder**
   Prompts you for a reminder title and an optional due date/time.

3. **Mark a reminder as completed**
   Asks for the reminder's ID, then marks it as completed.

4. **Remove a reminder**
   Deletes the reminder identified by the provided ID.

5. **Clear all completed reminders**
   Deletes all reminders currently marked as completed.

6. **Quit**
   Exits the application.

All reminders are stored in a JSON file located at:

```bash
~/.reminders.json
```

(This file is automatically created if it does not exist.)

---

## Examples

```bash
# Standard run (with interactive menu)
cargo run
```

**Sample Interaction**:

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

After you choose an option, follow the on-screen prompts to complete the desired action.

---

## Project Structure

```bash
Rust
└── Code
    ├── user-greeter
    ├── weather-cli
    ├── file-commander
    ├── <other projects>
    └── reminders-cli  <-- You are here
        ├── Cargo.toml
        ├── src
        │   └── main.rs
        └── README.md (this file)
```

---

## Contributing

Contributions are welcome! Please open an [issue](https://github.com/dunamismax/Rust/issues) or submit a pull request for any bug fixes or new features.

1. Fork the repository
2. Create a new branch: `git checkout -b feature/my-feature`
3. Commit your changes: `git commit -m "Add some feature"`
4. Push to your fork: `git push origin feature/my-feature`
5. Open a Pull Request

---

## Contact

Maintained by [dunamismax.com](https://github.com/dunamismax).
For any inquiries, please reach out via [email](mailto:dunamismax@tutamail.com).
