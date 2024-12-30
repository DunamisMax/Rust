# Hello World CLI

A simple command-line application written in Rust that greets users in random languages and colors.
This project is part of the [dunamismax/Rust](https://github.com/dunamismax/Rust) repository, located in the `Rust/Code/hello-world-cli` subdirectory.

---

## Table of Contents

- [Hello World CLI](#hello-world-cli)
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

- **Random-color ASCII banner** on startup.
- **Multi-lingual greetings** (randomly chosen from a diverse set).
- **Interactive prompt** for user name input (defaults to “World” if left empty).
- **Console clearing** for a neat, fresh look on each run.

---

## Prerequisites

1. **Rust & Cargo**
   Ensure you have Rust (and Cargo) installed. You can install Rust using [rustup](https://www.rust-lang.org/tools/install).

---

## Installation

1. **Clone the repository**:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   ```

2. **Navigate to the `hello-world-cli` directory**:

   ```bash
   cd Rust/Code/hello-world-cli
   ```

3. **Build and run**:

   ```bash
   cargo build
   cargo run
   ```

---

## Usage

When you run the application, it will:

1. Clear your terminal screen.
2. Display a **random-colored** ASCII banner.
3. Prompt you for your name. Enter your name (or press Enter to skip).
4. Greet you in a randomly selected language and color!

---

## Examples

```bash
# Basic run (interactive prompt)
cargo run

# If you simply press Enter at the prompt,
# it will greet "World" in a random language.

# If you enter "Alice" at the prompt,
# it might say "Spanish: Hola — Alice!" in a random color, for example.
```

---

## Project Structure

```bash
Rust
└── Code
    ├── weather-cli
    ├── file-commander
    ├── <other projects>
    └── hello-world-cli  <-- You are here
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
