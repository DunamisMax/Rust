# greeter

A **Rust-based** TUI application that prompts the user for their name, then greets them in a stylized terminal interface. **`greeter`** is part of the larger [Rust](https://github.com/dunamismax/Rust) repository maintained by [dunamismax](https://dunamismax.com).

---

## Table of Contents

- [greeter](#greeter)
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

**`greeter`** is a minimal “Hello World” application demonstrating how to build a text-based user interface (TUI) with [`crossterm`](https://crates.io/crates/crossterm) and [`tui`](https://crates.io/crates/tui) in Rust. It reads a name from either command-line input or stdin, displays a simple ASCII banner, and greets the user in a TUI window. This example also shows how to enable and disable raw mode at runtime, providing a beginner-friendly glimpse into TUI concepts in Rust.

---

## Features

1. **Interactive CLI + TUI**
   - Prompts for the user’s name if not provided via command-line arguments.
   - Uses a text-based user interface for drawing banners and messages.

2. **Raw Mode Management**
   - Demonstrates enabling and disabling raw mode to allow normal text input (for entering a name) and raw input (for TUI rendering).

3. **Colorful Display & ASCII Banner**
   - Displays an ASCII banner using various colors and styled text.

4. **Cross-Platform Line Endings**
   - Implements detection for different line endings on Windows vs. Unix-like systems.

5. **Verbose Mode**
   - Optional `--verbose` flag that prints an extra message.

---

## Prerequisites

- **Rust** (latest stable recommended)
- **Cargo** (comes with Rust)
- A terminal environment with support for raw mode (most modern terminals qualify).

---

## Installation

1. **Clone** the parent repository:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   cd Rust/Code/greeter
   ```

2. **Build** using Cargo (Rust’s package manager):

   ```bash
   cargo build --release
   ```

---

## Usage

1. **Run** the application from the project directory:

   ```bash
   cargo run --release
   ```

   or run the compiled executable:

   ```bash
   ./target/release/greeter
   ```

2. **CLI Arguments**:
   - **Positional**: `input` (an optional string that the TUI will greet if provided).
   - **Flags**:
     - `-v, --verbose`: Prints “Verbose mode enabled...” before showing the TUI.

3. **Controls**:
   - **Enter**: After the app finishes rendering the greeting, press Enter to exit.
   - **Raw Mode**: Automatically enabled when drawing the TUI; disabled before typing input.

---

## Examples

1. **Basic Run**:

   ```bash
   ./greeter
   ```

   - This will show the ASCII welcome banner and prompt for a name if none is provided as an argument.

2. **With Verbose Mode**:

   ```bash
   ./greeter --verbose
   ```

   - Prints an additional “Verbose mode enabled...” message before displaying the TUI.

3. **Passing a Name Directly**:

   ```bash
   cargo run -- "Alice"
   ```

   - Skips the prompt and greets “Alice” immediately.

---

## Project Structure

Below is a simplified view of where **`hello-world-tui`** resides within the [Rust](https://github.com/dunamismax/Rust) repository:

```bash
Rust/
├─ Code/
│  ├─ hello-world-cli/
│  ├─ net-commander/
│  ├─ reminders-cli/
│  ├─ rust-top/
│  ├─ ...
│  └─ greeter/
│     ├─ src/
│     │  └─ main.rs
│     ├─ Cargo.toml
│     └─ README.md          <-- You are here!
├─ Wiki/
│  ├─ ...
├─ LICENSE
└─ README.md               <-- main repository README
```

---

## Contributing

Contributions are always welcome! If you find a bug, wish to suggest an improvement, or want to add a feature, please open an [issue](https://github.com/dunamismax/Rust/issues) or a [pull request](https://github.com/dunamismax/Rust/pulls) in the main repository. Follow the standard Rust coding guidelines and ensure your code is well-documented.

---

## License

This project is under the [MIT License](https://github.com/dunamismax/Rust/blob/main/LICENSE). See the [`LICENSE` file](https://github.com/dunamismax/Rust/blob/main/LICENSE) in the root of the main repository for details.

---

## Contact

- **Author**: [dunamismax](https://dunamismax.com)
- **Email**: [dunamismax@tutamail.com](mailto:dunamismax@tutamail.com)
- **Website/Blog**: [dunamismax.com](https://dunamismax.com)

For any questions about **`greeter`** or the [Rust repository](https://github.com/dunamismax/Rust), feel free to reach out or open an issue!
