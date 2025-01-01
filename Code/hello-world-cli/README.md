# hello-world-cli

A **TUI-based** greeting application that displays random multilingual greetings. **`hello-world-cli`** is part of the larger [Rust](https://github.com/dunamismax/Rust) repository maintained by [dunamismax](https://dunamismax.com).

---

## Table of Contents

- [hello-world-cli](#hello-world-cli)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Features](#features)
  - [Installation](#installation)
  - [Usage](#usage)
  - [Repository Structure](#repository-structure)
  - [Contributing](#contributing)
  - [License](#license)
  - [Contact](#contact)

---

## Overview

**`hello-world-cli`** is a Rust-based Text User Interface (TUI) application that greets you in random languages from around the world. Simply type a name, press **Enter**, and you’ll see a multilingual greeting displayed in an eye-catching color! It uses [`tui`][tui-crate] + [`crossterm`][crossterm-crate] for rendering the interface, and runs on an asynchronous runtime powered by [`tokio`][tokio-crate].

---

## Features

1. **Multilingual Greetings**
   Selects from a large list of greetings in various languages (e.g., Spanish, Chinese, Korean, etc.).

2. **Randomized Colors**
   Each greeting is displayed in a random color to add variety and fun.

3. **Keyboard-Driven**
   Uses non-blocking keyboard input to capture typed names and handle application exit (`Esc`, `Ctrl-C`).

4. **Optional Verbose Mode**
   Pass `--verbose` to enable additional logging output on startup.

---

## Installation

1. **Clone** the parent repository:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   cd Rust/Code/hello-world-cli
   ```

2. **Build** using Cargo (Rust’s package manager):

   ```bash
   cargo build --release
   ```

3. **(Optional)** Adjust dependencies or feature flags in `Cargo.toml` if you want to customize the application.

---

## Usage

1. **Run** the application:

   ```bash
   cargo run --release
   ```

   or

   ```bash
   ./target/release/hello-world-cli
   ```

2. **Controls**:
   - **Type** any name, then **press Enter** to generate a random greeting.
   - **Esc** or **Ctrl-C** to **quit** the application.

3. **Flag**:
   - `--verbose`: Prints extra log information on startup.

4. **Example**:

   ```bash
   cargo run -- --verbose
   ```

   Type a name and press **Enter** to see greetings like “Spanish: Hola — Carlos!” displayed in random colors.

---

## Repository Structure

The **`hello-world-cli`** folder is one of several standalone Rust applications located within [Code/](https://github.com/dunamismax/Rust/tree/main/Code) in the main [Rust repository](https://github.com/dunamismax/Rust). Each subfolder contains its own Cargo project and its own `README.md`. The overall layout is:

```bash
Rust/
├─ Code/
│  ├─ hello-world-cli/
│  ├─ net-commander/
│  ├─ reminders-cli/
│  ├─ ...
│  └─ rust-top/
├─ Wiki/
│  ├─ ...
├─ LICENSE
└─ README.md         <-- main repository README
```

---

## Contributing

Contributions are welcome! If you encounter a bug or want to request a feature, please open an [issue](https://github.com/dunamismax/Rust/issues) or a [pull request](https://github.com/dunamismax/Rust/pulls) in the main repository. Please follow the project’s coding style and guidelines.

---

## License

This project is licensed under the [MIT License](https://github.com/dunamismax/Rust/blob/main/LICENSE). For details, see the [`LICENSE` file](https://github.com/dunamismax/Rust/blob/main/LICENSE) in the root of the main repository.

---

## Contact

- **Author**: [dunamismax](https://dunamismax.com)
- **Email**: [dunamismax@tutamail.com](mailto:dunamismax@tutamail.com)
- **Website/Blog**: [dunamismax.com](https://dunamismax.com)

For questions about **`hello-world-cli`** or the [Rust repository](https://github.com/dunamismax/Rust), feel free to reach out or open an issue!
