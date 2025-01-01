# net-commander

A **TUI-based** command-line toolkit for performing common network operations such as ping, DNS lookups, port scanning, traceroute, and more. **`net-commander`** is part of the larger [Rust](https://github.com/dunamismax/Rust) repository maintained by [dunamismax](https://dunamismax.com).

---

## Table of Contents

- [net-commander](#net-commander)
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

**`net-commander`** is a text-based user interface (TUI) application designed to simplify common network commands. It offers a main menu with items to **ping** hosts, perform **DNS lookups**, **port scans**, and even do **continuous latency monitoring**. This program demonstrates how Rust can leverage async/await patterns with [`tokio`](https://tokio.rs/) for concurrency, along with TUI frameworks ([`crossterm`](https://crates.io/crates/crossterm) and [`tui`](https://crates.io/crates/tui)) for an interactive terminal experience.

---

## Features

1. **Main Menu Navigation**
   Offers a user-friendly TUI interface to access various network operations.
2. **Ping & Ping Sweep**
   Quickly ping individual hosts or entire subranges (e.g., 192.168.1.1–192.168.1.10).
3. **DNS Lookup**
   Resolves hostnames to IP addresses using Rust’s built-in `to_socket_addrs`.
4. **Port Scanning**
   Parallel TCP port checks to discover open ports within a specified range.
5. **Traceroute**
   Wrapper around native OS commands (`tracert` on Windows, `traceroute` on UNIX).
6. **Firewall/VPN Detection**
   Basic checks to detect local firewall states and active VPN interfaces.
7. **Latency Monitoring (Continuous Ping)**
   Sends recurring pings to track network latency over time until the user stops.

---

## Prerequisites

- **Rust** (and Cargo) installed (recommended version 1.60+).
- **git** for cloning the repository.
- **Tokio** runtime will be downloaded automatically via Cargo dependencies.
- Native system commands like `ping`, `traceroute`/`tracert`, `ipconfig`/`ifconfig` are utilized.
- For Windows, `netsh` may be used for firewall detection; for Linux, `firewalld`/`iptables` checks are done.

---

## Installation

1. **Clone** the main Rust repository:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   ```

2. **Navigate** to the `net-commander` directory:

   ```bash
   cd Rust/Code/net-commander
   ```

3. **Build** the project (in release mode):

   ```bash
   cargo build --release
   ```

---

## Usage

1. **Run** the application:

   ```bash
   cargo run --release
   ```

   or

   ```bash
   ./target/release/net-commander
   ```

2. **CLI Arguments**:
   - `--verbose` (or `-v`): Enables verbose mode (prints additional messages on start).

3. **Menu Controls**:
   - **Up/Down**: Move through the menu.
   - **Enter**: Select the highlighted menu item.
   - **Q** (or **Ctrl-C**, **Esc**): Quit the application.

4. **In-Menu Examples**:
   - **Ping Host**: Enter an IP or hostname; the tool will invoke the native `ping` command to check reachability.
   - **Port Scan**: Provide a start and end port (e.g., 1 to 1024) to find open TCP ports on the specified host.
   - **DNS Lookup**: Resolve a hostname (e.g., `example.com`) to its IP addresses.

---

## Examples

1. **Verbose Mode**:

   ```bash
   ./target/release/net-commander --verbose
   ```

   Displays extra logs in the terminal when starting the TUI.

2. **Normal Execution**:

   ```bash
   ./net-commander
   ```

   Launches the TUI right away with default settings.

3. **Continuous Latency Monitoring**:
   - From the main menu, select “8) Latency monitoring (continuous ping).”
   - Type in the host (e.g., `8.8.8.8`) when prompted.
   - The application will show ping results every second until a key is pressed.

---

## Project Structure

The **`net-commander`** folder is part of the [Code/](https://github.com/dunamismax/Rust/tree/main/Code) directory in the main [Rust repository](https://github.com/dunamismax/Rust). The layout is:

```bash
Rust/
├─ Code/
│  ├─ hello-world-cli/
│  ├─ net-commander/
│  │  ├─ src/
│  │  ├─ Cargo.toml
│  │  └─ README.md  <-- You are here!
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

Contributions are welcome! If you encounter a bug or have a feature request, please open an [issue](https://github.com/dunamismax/Rust/issues) or submit a [pull request](https://github.com/dunamismax/Rust/pulls) to the main repository. Be sure to follow the project’s coding style and guidelines.

---

## License

This project is licensed under the [MIT License](https://github.com/dunamismax/Rust/blob/main/LICENSE). Please see the [`LICENSE` file](https://github.com/dunamismax/Rust/blob/main/LICENSE) in the root of the main repository for details.

---

## Contact

- **Author**: [dunamismax](https://dunamismax.com)
- **Email**: [dunamismax@tutamail.com](mailto:dunamismax@tutamail.com)
- **Website/Blog**: [dunamismax.com](https://dunamismax.com)

For questions about **`net-commander`** or the [Rust repository](https://github.com/dunamismax/Rust), feel free to reach out or open an issue!
