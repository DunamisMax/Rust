# secure-notes

A **Rust-based** command-line application that securely manages encrypted notes using password-based encryption. **`secure-notes`** is part of the larger [Rust](https://github.com/dunamismax/Rust) repository maintained by [dunamismax](https://dunamismax.com).

---

## Table of Contents

- [secure-notes](#secure-notes)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Features](#features)
  - [Installation](#installation)
  - [Usage](#usage)
  - [Examples](#examples)
  - [Repository Structure](#repository-structure)
  - [Contributing](#contributing)
  - [License](#license)
  - [Contact](#contact)

---

## Overview

**`secure-notes`** is a terminal user interface (TUI) application that encrypts and manages your text-based notes. It leverages the ring library for password-based encryption/decryption, [`tui`][tui-url] + `crossterm` for its interactive interface, and the Rust async runtime [`tokio`][tokio-url] to manage concurrency. With **`secure-notes`**, you can:

- Create, edit, and delete encrypted notes
- Secure all notes behind a single master password
- View a list of notes (with partial content preview)
- Safely store data in an encrypted file on disk

---

## Features

1. **Password-Protected Access**
   Derives a secure key using **PBKDF2** to protect your notes with a master password.

2. **Encrypted at Rest**
   Uses **ChaCha20-Poly1305** for authenticated encryption, ensuring your notes are unreadable without the correct key.

3. **TUI Navigation**
   - A built-in text-based interface for creating, editing, viewing, or deleting notes.
   - Keyboard shortcuts for quick saving, discarding, and menu navigation.

4. **Cross-Platform**
   - The TUI can run on Windows, Linux, or macOS.
   - On Linux, macOS, and Windows, the CLI usage remains similar.

5. **CLI Arguments**
   - `--file`: Specify a custom path to the encrypted notes file (default: `secure_notes.json.enc`).

---

## Installation

1. **Clone** the parent repository:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   cd Rust/Code/secure-notes
   ```

2. **Build** using Cargo (Rust’s package manager):

   ```bash
   cargo build --release
   ```

3. **(Optional)** Review or update dependencies in `Cargo.toml` to enable additional feature flags (e.g., enabling mouse capture, etc.).

---

## Usage

1. **Run** the application:

   ```bash
   cargo run --release
   ```

   or

   ```bash
   ./target/release/secure-notes
   ```

2. **Specify an alternate file path** (if desired):

   ```bash
   ./secure-notes --file "my_custom_notes.enc"
   ```

3. **Controls**:
   - **Password Prompt**: Enter your master password; press **Enter** to confirm, **Esc** to quit.
   - **Menu Options**:
     - **1**: View Notes
     - **2**: Create Note
     - **3**: Edit Note
     - **4**: Delete Note
     - **5**: Open Note
     - **6**: Delete ALL Notes
     - **7**: Exit
   - **Create/Edit Screen**:
     - **Esc**: Save changes and return to menu
     - **F2**: Discard changes and return to menu
   - **Simple Input Screens** (e.g., delete by ID):
     - **Enter**: Confirm
     - **Esc**: Cancel

---

## Examples

1. **Basic Run**

   ```bash
   ./secure-notes
   ```

   - Prompts for your master password, then displays the main menu.

2. **Specifying a Custom Encrypted File**

   ```bash
   ./secure-notes --file "special_notes_file.enc"
   ```

   - Uses a non-default file to store/retrieve your notes.

3. **Viewing & Creating Notes**
   - Once running, press **1** to view notes, then **2** to create a new note.
   - Provide your text in the editor, press **Esc** to save, or **F2** to discard.

---

## Repository Structure

The **`secure-notes`** folder is one of several standalone Rust applications located within [Code/](https://github.com/dunamismax/Rust/tree/main/Code) in the main [Rust repository](https://github.com/dunamismax/Rust). Each subfolder has its own Cargo project and README. The overall layout is:

```bash
Rust/
├─ Code/
│  ├─ hello-world-cli/
│  ├─ net-commander/
│  ├─ reminders-cli/
│  ├─ greeter/
│  ├─ file-commander/
│  ├─ rust-top/
│  └─ secure-notes/
│     ├─ src/
│     ├─ Cargo.toml
│     └─ README.md  <-- You are here!
├─ Wiki/
│  ├─ ...
├─ LICENSE
└─ README.md         <-- main repository README
```

---

## Contributing

Contributions are welcome! If you encounter a bug or want to request a feature, please open an [issue](https://github.com/dunamismax/Rust/issues) or a [pull request](https://github.com/dunamismax/Rust/pulls) in the main repository. Make sure to follow the project’s coding style and guidelines.

---

## License

This project is licensed under the [MIT License](https://github.com/dunamismax/Rust/blob/main/LICENSE). Please see the [`LICENSE` file](https://github.com/dunamismax/Rust/blob/main/LICENSE) in the root of the main repository for details.

---

## Contact

- **Author**: [dunamismax](https://dunamismax.com)
- **Email**: [dunamismax@tutamail.com](mailto:dunamismax@tutamail.com)
- **Website/Blog**: [dunamismax.com](https://dunamismax.com)

For questions about **`secure-notes`** or the [Rust repository](https://github.com/dunamismax/Rust), feel free to reach out or open an issue!
