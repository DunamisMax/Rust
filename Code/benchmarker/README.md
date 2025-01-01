# benchmarker

A **Rust-based** application for benchmarking **CPU** and **RAM** usage, powered by a TUI (text-based user interface) courtesy of **ratatui** and **crossterm**. **`benchmarker`** is part of the [Rust](https://github.com/dunamismax/Rust) repository maintained by [dunamismax](https://dunamismax.com).

---

## Table of Contents

- [benchmarker](#benchmarker)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Features](#features)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Usage](#usage)
    - [Command-Line Arguments](#command-line-arguments)
  - [Examples](#examples)
  - [Project Structure](#project-structure)
  - [Contributing](#contributing)
  - [License](#license)
  - [Contact](#contact)

---

## Overview

**`benchmarker`** is a **Rust** application designed to assess CPU and RAM performance via a minimalistic text user interface (TUI). It simultaneously or individually tests:

1. **CPU** - Runs a multi-threaded loop utilizing various floating-point operations.
2. **RAM** - Allocates and writes to a large block of memory.
3. **Combined** - Launches both CPU and RAM benchmarks at the same time, pushing your system to its limits.

The TUI interface provides a welcome screen, a menu to select the benchmark type, and a benchmark progress view. Pressing **Esc** during a benchmark returns you to the menu, stopping the tests.

---

## Features

1. **CPU Benchmark**
   - Spins multiple tasks (one per CPU core) that perform floating-point operations in a loop.

2. **RAM Benchmark**
   - Allocates a user-specified chunk of memory and constantly writes to it.

3. **Combined CPU+RAM**
   - Simultaneously runs both benchmarks to maximize system stress.

4. **TUI Navigation**
   - A simple screen-based interface: **Welcome**, **Menu**, **Benchmark In Progress**, and graceful exit.

5. **Cross-Platform Line Endings**
   - Ensures consistent output on Windows, macOS, and Linux.

---

## Prerequisites

- **Rust** (1.60+ recommended)
- **Cargo** (bundled with Rust)

No additional API keys are required; all benchmarks are local to your machine.

---

## Installation

1. **Clone the repository**:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   cd Rust/Code/benchmarker
   ```

2. **Build** the project:

   ```bash
   cargo build --release
   ```

   This will produce a binary (`benchmarker`) inside the `./target/release/` folder.

---

## Usage

From the **`benchmarker`** directory, you can run:

```bash
cargo run --release
```

This opens a text-based UI with a welcome screen. Press **Enter** to proceed to the benchmark menu, then select:

- **1** for CPU Benchmark
- **2** for RAM Benchmark
- **3** for Combined Benchmark
- **4** to Exit

### Command-Line Arguments

- **`--ram_mb <megabytes>`** (optional)
  Specifies how many megabytes of memory to allocate for the RAM benchmark. Default is `0`, which the program interprets as approximately **4GB**.

For example:

```bash
cargo run --release -- --ram_mb 512
```

Would allocate **512 MB** for the RAM test.

---

## Examples

1. **Run the TUI and choose CPU Benchmark**

   ```bash
   cargo run --release
   ```

   - Press **Enter** at the welcome screen.
   - Select **1** for CPU Benchmark.
   - Press **Esc** to stop and return to the menu.

2. **Run the TUI with a custom RAM size**

   ```bash
   cargo run --release -- --ram_mb 1024
   ```

   - Allocates **1 GB** of RAM.
   - In the menu, choose **2** to start the RAM benchmark.

3. **Combined CPU+RAM**
   - After launching, select **3** to stress both CPU and RAM simultaneously.

---

## Project Structure

Below is a high-level look at the **`benchmarker`** folder within the main [Rust repository][rust-repo-url]:

```bash
Rust/
├─ Code/
│  ├─ ...
│  └─ benchmarker/
│     ├─ src/
│     ├─ Cargo.toml
│     └─ README.md         <-- You are here!
├─ Wiki/
│  ├─ ...
├─ LICENSE
└─ README.md
```

Each subfolder in `Code/` is an independent Cargo project. **`benchmarker`** specifically contains:

- **`main.rs`**: The entry point, including the TUI logic, benchmarks, and CLI parsing.
- **`Cargo.toml`**: Dependency and metadata definitions.

---

## Contributing

Contributions are welcome! If you find any issues or want to add more benchmarking features, please open a [pull request][pulls-url] or file an [issue][issues-url]. Keep the coding style consistent and ensure all changes compile and pass basic testing before submitting.

---

## License

This project is licensed under the [MIT License][license-url]. See the [`LICENSE` file][license-url] in the root of the main repository for details.

---

## Contact

- **Author**: [dunamismax](https://dunamismax.com)
- **Repository**: [dunamismax/Rust][rust-repo-url]
- **Email**: [dunamismax@tutamail.com](mailto:dunamismax@tutamail.com)

Feel free to reach out or open an issue if you have questions about **`benchmarker`** or the [Rust repository][rust-repo-url]!

---

[rust-repo-url]: https://github.com/dunamismax/Rust
[issues-url]: https://github.com/dunamismax/Rust/issues
[pulls-url]: https://github.com/dunamismax/Rust/pulls
[license-url]: https://github.com/dunamismax/Rust/blob/main/LICENSE
