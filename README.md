```markdown
# Rust

A **collection** of Rust-based projects maintained by [dunamismax](https://dunamismax.com). This repository showcases various **standalone** command-line applications, each in its own folder under the [`Code/`](Code) directory. Whether you’re exploring simple CLI workflows or more advanced TUI experiments, you’ll find examples of modern Rust patterns and practices here.

---

## Table of Contents

- [Overview](#overview)
- [Repository Structure](#repository-structure)
- [Subprojects](#subprojects)
- [Getting Started](#getting-started)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)

---

## Overview

This repo serves as a **playground** and **showcase** for Rust. Each subproject has a dedicated folder with its own `README.md`, `Cargo.toml`, and source code. Topics include:

- **Basic CLI** tools (`hello-world-cli`).
- **Networking** utilities (`net-commander`).
- **Task management** apps (`reminders-cli`).
- **System monitoring** (`rust-top`).
- **Other** projects exploring new features or libraries.

If you’re interested in learning Rust by example or want to see how to structure small-to-medium Rust projects, this repository offers a variety of patterns and solutions.

---

## Repository Structure

Below is a simplified view of the core layout:

```bash

Rust/
├─ Code/
│  ├─ hello-world-cli/
│  ├─ net-commander/
│  ├─ reminders-cli/
│  ├─ rust-top/
│  ├─ secure-notes/
│  ├─ weather-cli/
│  └─ ... (other applications)
├─ Wiki/
│  ├─ ...
├─ LICENSE
└─ README.md  <-- You're here (main repository README)

```

Each subdirectory under [`Code/`](Code) is an individual Rust application with its own build script (`Cargo.toml`) and a dedicated `README.md` providing more details.

---

## Subprojects

Here are some notable subprojects (not an exhaustive list):

1. **[rust-top](Code/rust-top)**
   - A “top-like” TUI tool for monitoring processes, CPU usage, and memory.

2. **[hello-world-cli](Code/hello-world-cli)**
   - A minimal command-line greeting utility demonstrating argument parsing and TUI basics.

3. **[net-commander](Code/net-commander)**
   - A simple network utility for checking connectivity, performing pings, or retrieving HTTP endpoints.

4. **[reminders-cli](Code/reminders-cli)**
   - A CLI tool for managing short-term tasks and reminders on the local machine.

5. **[weather-cli](Code/weather-cli)**
   - Retrieves weather data from a public API and displays it via TUI with configurable refresh intervals.

Check each folder for specific build/run instructions, usage examples, and additional features.

---

## Getting Started

1. **Clone** this repository:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   cd Rust
   ```

2. **Explore** the applications in the `Code/` directory:

   ```bash
   cd Code/rust-top   # or any other subproject
   cargo run --release
   ```

3. **Customize** or **build** each subproject independently. They do not share dependencies beyond Rust itself.

---

## Contributing

Pull requests and issues are welcome! If you’d like to contribute:

1. Fork the repository.
2. Create a new branch with your changes (`git checkout -b feature-xyz`).
3. Submit a Pull Request with a clear description of your changes.

Please open issues if you encounter bugs or have feature ideas. Follow the coding style, provide adequate testing, and keep commits atomic.

---

## License

All projects in this repository are licensed under the [MIT License](LICENSE). Feel free to use, modify, and distribute the code in accordance with this license.

---

## Contact

- **Author**: [dunamismax](https://dunamismax.com)
- **Email**: [dunamismax@tutamail.com](mailto:dunamismax@tutamail.com)
- **Website / Blog**: [dunamismax.com](https://dunamismax.com)

For any questions or suggestions, feel free to reach out or open an [issue](https://github.com/dunamismax/Rust/issues). Enjoy exploring Rust!
