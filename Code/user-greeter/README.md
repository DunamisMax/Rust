# user-greeter

A fun little command-line application written in Rust that prints a colorful ASCII banner and greets the user by name.
This project is part of the [dunamismax/Rust](https://github.com/dunamismax/Rust) repository, located in the `Rust/Code/user-greeter` subdirectory.

---

## Table of Contents

- [user-greeter](#user-greeter)
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

- **Colorful ASCII banner** for a fun CLI experience.
- **Random color selection** each time you run the app.
- **Interactive prompt** to enter your name.
- **Graceful fallback** if no name is provided.

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

2. **Navigate to the `user-greeter` directory**:

   ```bash
   cd Rust/Code/user-greeter
   ```

3. **Build and run**:

   ```bash
   cargo build
   cargo run
   ```

---

## Usage

1. **Run without arguments**:
   You’ll be greeted with a colorful ASCII banner. Then, the application will prompt you for your name. Just type your name and press **Enter**.

   ```bash
   cargo run
   ```

2. **No name provided**:
   If you leave the prompt blank, the app will greet you with `"Hello, World! (No name provided.)"`.

---

## Examples

```bash
# Standard run
cargo run
```

**Expected output** (colors will vary):
```bash
        _
       | |
 _   _  ___   ___  _ __    __ _  _ __   ___   ___ | |_   ___  _ __
| | | |/ __| / _ \| '__|  / _` || '__| / _ \ / _ \| __| / _ \| '__|
| |_| |\__ \|  __/| |    | (_| || |   |  __/|  __/| |_ |  __/| |
 \__,_||___/ \___||_|     \__, ||_|    \___| \___| \__| \___||_|
                        __/ |
                       |___/

Welcome to the colorful user-greeter!!

What's your name? John

Hello, John!
```

---

## Project Structure

```bash
Rust
└── Code
    ├── weather-cli
    ├── file-commander
    ├── <other projects>
    └── user-greeter  <-- You are here
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
