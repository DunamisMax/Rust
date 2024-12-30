# Rust Projects by [dunamismax.com](https://github.com/dunamismax)

A collection of Rust projects showcasing a variety of use cases and coding practices. Each project resides in its own subdirectory under `Rust/Code/`, with its own `Cargo.toml` and source files. Clone the repository, explore the projects that interest you, and feel free to contribute!

---

## Table of Contents

- [Rust Projects by dunamismax.com](#rust-projects-by-dunamismaxcom)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Projects](#projects)
    - [1. Weather CLI](#1-weather-cli)
    - [2. User Greeter](#2-user-greeter)
    - [3. File Commander](#3-file-commander)
  - [Getting Started](#getting-started)
  - [Usage](#usage)
  - [Project Structure](#project-structure)
  - [Contributing](#contributing)
  - [Contact](#contact)

---

## Overview

This repository holds multiple Rust-based applications, each demonstrating different features or concepts such as CLI design, concurrency, file manipulation, API interaction, and more. Whether you’re a beginner looking to learn Rust or an experienced developer seeking real-world examples, you’ll find something useful here.

---

## Projects

Below are some highlighted projects in the `Rust/Code/` directory. Each has its own `README.md` with detailed setup instructions, features, and usage examples.

### 1. Weather CLI

**Description**: A command-line application that fetches real-time weather data from the [OpenWeatherMap](https://openweathermap.org/) API.
**Key Features**:

- Fetch weather by city or ZIP code
- Optional interactive mode if no arguments are given
- ASCII banner with random colors
- Displays temperature, humidity, wind info, and more

Go to [Rust/Code/weather-cli](./Code/weather-cli) to learn more.

### 2. User Greeter

**Description**: A simple CLI that demonstrates user input, random color greetings, and basic I/O in Rust.

**Key Features**:

- Interactive prompts to greet users in different styles
- Random color selection for output text
- Beginner-friendly example of Rust’s `colored` and `rand` crates

Visit [Rust/Code/user-greeter](./Code/user-greeter) for details.

### 3. File Commander

**Description**: A command-line utility for organizing, copying, moving, or deleting files—an all-in-one file management tool.
**Key Features**:

- Organize files by extension, date, or size
- Parallel processing using [Rayon](https://crates.io/crates/rayon)
- Interactive prompts to guide file actions (copy, move, delete)
- Example of error handling, concurrency, and filesystem operations

Refer to [Rust/Code/file-commander](./Code/file-commander) for further info.

> **Note**: As new projects are added, you’ll see additional folders under `Rust/Code/`. Each will have a dedicated README with installation and usage instructions.

---

## Getting Started

1. **Install Rust and Cargo**
   - Make sure [Rustup](https://rustup.rs/) is installed on your system. You can confirm with:

     ```bash
     rustc --version
     cargo --version
     ```

2. **Clone the Repository**

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   ```

3. **Navigate to the `Code` folder**

   ```bash
   cd Rust/Code
   ```

4. **Pick a Project**
   - Example for the Weather CLI:

     ```bash
     cd weather-cli
     ```

---

## Usage

Because each project is standalone, usage typically follows a pattern:

```bash
cd Rust/Code/<project-name>
cargo build
cargo run [-- <arguments>]
```

For complete details on command-line arguments, environment variables (like API keys), or optional config files, see the individual project’s README.md.

---

## Project Structure

Below is a general outline of how this repository is organized. Each subdirectory under `Rust/Code/` represents a distinct project:

```bash
Rust
├── Code
│   ├── weather-cli
│   │   ├── src
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── user-greeter
│   │   ├── src
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── file-commander
│   │   ├── src
│   │   ├── Cargo.toml
│   │   └── README.md
│   └── ...
└── README.md  <-- You are here
```

---

## Contributing

Contributions are welcome! Whether it’s fixing bugs, improving documentation, or adding new features/projects:

1. [Fork](https://github.com/dunamismax/Rust/fork) the repository
2. Create a new branch:

   ```bash
   git checkout -b feature/my-feature
   ```

3. Commit your changes:

   ```bash
   git commit -m "Describe your feature"
   ```

4. Push to your fork:

   ```bash
   git push origin feature/my-feature
   ```

5. Open a [Pull Request](https://github.com/dunamismax/Rust/pulls)

---

## Contact

Maintained by [dunamismax.com](https://github.com/dunamismax).
For any inquiries, you can reach out via [email](mailto:dunamismax@tutamail.com).

---

Thank you for checking out the **dunamismax/Rust** repository! We hope these projects help you learn something new about Rust—or that they serve as useful utilities for your workflow. Feel free to explore, experiment, and contribute. Happy coding!
