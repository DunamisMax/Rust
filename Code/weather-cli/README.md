# weather-cli

A **Rust-based** command-line application that fetches weather data from the [OpenWeatherMap](https://openweathermap.org/) API using a TUI (text-based user interface). **`weather-cli`** is part of the [Rust](https://github.com/dunamismax/Rust) repository maintained by [dunamismax](https://dunamismax.com).

---

## Table of Contents

- [weather-cli](#weather-cli)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Features](#features)
  - [Installation](#installation)
  - [Usage](#usage)
    - [Command-Line Arguments](#command-line-arguments)
    - [Environment Variable](#environment-variable)
  - [Examples](#examples)
  - [Project Structure](#project-structure)
  - [Contributing](#contributing)
  - [License](#license)
  - [Contact](#contact)

---

## Overview

**`weather-cli`** is a small **Rust** application that interacts with [OpenWeatherMap’s API](https://openweathermap.org/) to fetch current weather data for any city or ZIP code. It uses a simple TUI (terminal user interface) powered by `crossterm` and `tui` to display both a welcome screen (prompting for location input) and the resulting weather information.

---

## Features

1. **TUI Welcome Screen**
   - Displays an ASCII banner and prompts for user input (city or ZIP code) if not specified via command line.

2. **Fetch Weather by City or ZIP**
   - Automatically determines whether the input is numeric (ZIP) or alphabetical (city).

3. **Detailed Weather Display**
   - Temperature, pressure, humidity, wind speed, sunrise/sunset times, etc.

4. **Units Selection**
   - Supports **imperial** (°F), **metric** (°C), and **standard** (Kelvin) temperature scales.

5. **Prompted Exit**
   - After displaying weather info, waits for user input (press Enter) before closing.

---

## Installation

1. **Clone** the [parent Rust repository][rust-repo-url]:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   cd Rust/Code/weather-cli
   ```

2. **Create or Update `.env`** (optional but recommended):
   Inside `weather-cli/`, create a file named `.env` with your [OpenWeatherMap API Key][owm-signup-url]:

   ```env
   OWM_API_KEY=YOUR_API_KEY_HERE
   ```

   Alternatively, you can set the environment variable `OWM_API_KEY` in your shell or system environment.

3. **Build** the application using Cargo:

   ```bash
   cargo build --release
   ```

   This will produce a binary in `./target/release/weather-cli`.

---

## Usage

From the **`weather-cli`** directory, run:

```bash
cargo run --release
```

If you already have a `weather-cli` binary, you can also just run:

```bash
./target/release/weather-cli
```

Upon execution, the TUI will display a welcome banner. You can either provide a location directly (e.g., `--location Boston`) or let the app prompt you for a **city name** or **ZIP code**.

### Command-Line Arguments

- **`--location`** (optional)
  A city or ZIP code. If omitted, the TUI will prompt for it.
- **`-c` / `--country`** (optional)
  Default is `"us"` (United States). Can be changed to `"uk"`, `"de"`, etc.
- **`-u` / `--units`** (optional)
  Default is `"imperial"` (°F). Other valid values: `"metric"` (°C) or `"standard"` (Kelvin).

### Environment Variable

- **`OWM_API_KEY`**
  An [OpenWeatherMap API key][owm-signup-url]. Must be set in `.env` or in your environment.

---

## Examples

1. **Fetch Weather for London (Prompted)**

   ```bash
   cargo run --release
   ```

   - The app will display a banner and then ask for a city or ZIP code.
   - Enter `London` and press Enter to view London’s current weather.

2. **Specify a ZIP Code & Country**

   ```bash
   cargo run --release -- --location 10001 --country us
   ```

   - Uses ZIP code `10001` in the United States.

3. **Metric Units**

   ```bash
   cargo run --release -- --location Berlin --country de --units metric
   ```

   - Displays temperatures in Celsius.

---

## Project Structure

Below is a high-level look at the **`weather-cli`** folder within the main [Rust repository][rust-repo-url]:

```bash
Rust/
├─ Code/
│  ├─ hello-world-cli/
│  ├─ net-commander/
│  ├─ rust-top/
│  ├─ ...
│  └─ weather-cli/
│     ├─ src/
│     ├─ .env.example      <-- Sample environment file (if provided)
│     ├─ Cargo.toml
│     └─ README.md         <-- You are here!
├─ Wiki/
│  ├─ ...
├─ LICENSE
└─ README.md
```

Each subfolder in `Code/` is an independent Rust Cargo project with its own `README.md`.

---

## Contributing

Contributions are welcome! If you have a bug to report or a feature to request, please open an [issue][issues-url] or a [pull request][pulls-url] in the main repository. When contributing, follow the existing code style and guidelines.

---

## License

This project is licensed under the [MIT License][license-url]. Please see the [`LICENSE` file][license-url] in the root of the main repository for details.

---

## Contact

- **Author**: [dunamismax](https://dunamismax.com)
- **Repository**: [dunamismax/Rust][rust-repo-url]
- **Email**: [dunamismax@tutamail.com](mailto:dunamismax@tutamail.com)

Feel free to reach out or open an issue if you have questions about **`weather-cli`** or the [Rust repository][rust-repo-url]!

---

[rust-repo-url]: https://github.com/dunamismax/Rust
[owm-signup-url]: https://openweathermap.org/appid
[issues-url]: https://github.com/dunamismax/Rust/issues
[pulls-url]: https://github.com/dunamismax/Rust/pulls
[license-url]: https://github.com/dunamismax/Rust/LICENSE