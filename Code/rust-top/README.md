# rust-top

A **Rust-based** “top-like” command-line tool that displays real-time information about running processes on a Linux system, including CPU usage and memory consumption. **`rust-top`** is part of the larger [Rust](https://github.com/dunamismax/Rust) repository maintained by [dunamismax](https://dunamismax.com).

---

## Table of Contents

- [rust-top](#rust-top)
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

**`rust-top`** is a TUI (text-based user interface) application similar to the standard Linux `top` command. It leverages the `/proc` filesystem to gather process metrics and displays them in an interactive UI. By default, it shows process details such as:

- **PID** (Process ID)
- **Name**
- **State**
- **PPID** (Parent Process ID)
- **CPU%** usage
- **Memory** (in human-readable format)

This project demonstrates Rust’s concurrency, async/await patterns with [`tokio`][tokio-url], and TUI frameworks (`crossterm` + `tui`).

---

## Features

1. **Real-Time Process Monitoring**
   Refreshes the process list at a specified interval (default: 2000 ms).

2. **CPU & Memory Stats**
   Displays approximate CPU usage and memory footprint for each process, sorted in descending order by memory usage.

3. **Responsive TUI**
   Uses non-blocking keyboard input so you can **press** `q`, `Esc`, **Ctrl-C**, or **SHIFT+Q** to **quit** gracefully.

4. **Mouse Capture (Optional)**
   Run with `--mouse` to enable mouse input capture (though minimal mouse interaction is implemented by default).

5. **Cross-Platform Friendly**
   Compiles on non-Linux systems but will show an empty process list. The main functionality is Linux-specific via `/proc`.

---

## Installation

1. **Clone** the parent repository:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   cd Rust/Code/rust-top
   ```

2. **Build** using Cargo (Rust’s package manager):

   ```bash
   cargo build --release
   ```

3. **(Optional)** Enable additional features (mouse, more logging, etc.) by updating the `[dependencies]` or feature flags in `Cargo.toml`.

---

## Usage

1. **Run** the application:

   ```bash
   cargo run --release
   ```

   or

   ```bash
   ./target/release/rust-top
   ```

2. **Key Flags**:
   - `--refresh-ms <millis>`: How often to refresh (default: 2000 ms).
   - `--mouse`: Enable mouse capture.

3. **Controls**:
   - **q** / **Esc** / **Ctrl-C**: Quit the application.
   - **SHIFT+Q**: Also quits.

4. **Example**:

   ```bash
   ./rust-top --refresh-ms 1000 --mouse
   ```

---

## Repository Structure

The **`rust-top`** folder is one of several standalone Rust applications located within [Code/](https://github.com/dunamismax/Rust/tree/main/Code) in the main [Rust repository](https://github.com/dunamismax/Rust). Each subfolder contains its own Cargo project and its own `README.md`. The overall layout is:

```bash
Rust/
├─ Code/
│  ├─ hello-world-cli/
│  ├─ net-commander/
│  ├─ reminders-cli/
│  ├─ ...
│  └─ rust-top/
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

For questions about **`rust-top`** or the [Rust repository](https://github.com/dunamismax/Rust), feel free to reach out or open an issue!
