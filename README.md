# Rust

A **collection** of Rust-based projects maintained by [dunamismax](https://dunamismax.com). This repository showcases various **standalone** command-line applications, each in its own folder under the [`Code/`](Code) directory. Whether you’re exploring simple CLI workflows or more advanced TUI experiments, you’ll find examples of modern Rust patterns and practices here.

---

## Table of Contents

- [Rust](#rust)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Repository Structure](#repository-structure)
  - [Subprojects](#subprojects)
  - [Getting Started](#getting-started)
  - [Contributing](#contributing)
  - [License](#license)
  - [Contact](#contact)

---

## Overview

This repo serves as a **playground** and **showcase** for Rust. Each subproject has a dedicated folder with its own `README.md`, `Cargo.toml`, and source code. Featured topics include:

- **Task management** apps (`reminders-cli`)
- **Weather data retrieval** (`weather-cli`)
- **System monitoring** tools (`rust-top`)
- **Encrypted note management** (`secure-notes`)
- **File and directory command utilities** (`file-commander`)

If you’re interested in learning Rust by example or want to see how to structure small-to-medium Rust projects, this repository offers a variety of patterns and solutions.

---

## Repository Structure

Below is a simplified view of the core layout:

```bash
Rust/
├─ Code/
│  ├─ file-commander/
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

Here are some **notable** subprojects (not an exhaustive list):

1. **[reminders-cli](Code/reminders-cli/)**
   A **command-line to-do tool** for scheduling quick reminders and tasks. Features include creating, listing, and removing reminders, plus simple search capabilities.

2. **[weather-cli](Code/weather-cli/)**
   Retrieves weather data from a public API and displays it via a **text-based user interface**. Supports multiple units of measurement and location inputs (city or ZIP).

3. **[rust-top](Code/rust-top/)**
   A “top-like” **TUI application** for monitoring processes, CPU usage, memory consumption, and more—providing insights into your system’s live performance.

4. **[secure-notes](Code/secure-notes/)**
   Offers a **secure CLI-based vault** for storing encrypted notes or credentials. Utilizes Rust’s strong cryptographic libraries to ensure data privacy.

5. **[file-commander](Code/file-commander/)**
   A **file-management CLI** capable of performing operations like copying, moving, deleting, and renaming files and folders. Includes support for batch operations.

Other subprojects, such as **[hello-world-cli](Code/hello-world-cli/)** and **[net-commander](Code/net-commander/)**, demonstrate smaller-scale examples of Rust-based CLI utilities. Check each folder for specific build/run instructions, usage examples, and additional features.

---

## Getting Started

1. **Clone** this repository:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   cd Rust
   ```

2. **Explore** the applications in the `Code/` directory:

   ```bash
   cd Code/reminders-cli   # or any other subproject
   cargo run --release
   ```

3. **Customize or build** each subproject independently. They do not share dependencies beyond Rust itself, so you can work on any application in isolation.

---

## Contributing

Contributions are welcome! If you have ideas or improvements, please:

1. Fork the repository.
2. Create a new branch with your changes (`git checkout -b feature-xyz`).
3. Commit and push your changes.
4. Open a Pull Request with a clear description of your modifications or additions.

Feel free to open an [issue](https://github.com/dunamismax/Rust/issues) if you encounter bugs or have feature suggestions. Please follow the existing code style, provide sufficient testing, and keep commits atomic.

---

## License

All projects in this repository are licensed under the [MIT License](LICENSE). You can use, modify, and distribute the code under the terms of this license.

---

## Contact

- **Author**: [dunamismax](https://dunamismax.com)
- **Email**: [dunamismax@tutamail.com](mailto:dunamismax@tutamail.com)
- **Website / Blog**: [dunamismax.com](https://dunamismax.com)

For questions or suggestions, please reach out or open an [issue](https://github.com/dunamismax/Rust/issues). Happy coding and exploring Rust!
