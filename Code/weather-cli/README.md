# Weather CLI

A simple command-line application written in Rust that fetches weather information from the [OpenWeatherMap](https://openweathermap.org/) API.
This project is part of the [dunamismax/Rust](https://github.com/dunamismax/Rust) repository, located in the `Rust/Code/weather-cli` subdirectory.

---

## Table of Contents

- [Weather CLI](#weather-cli)
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

- Fetch current weather by **city name** or **ZIP code**.
- Support for **interactive** input if no location is supplied.
- Multiple **units of measurement** (metric, imperial, standard).
- **ASCII banner** in random colors for a fun CLI experience.
- Displays:
  - Temperature, feels like, min/max temps
  - Pressure, humidity
  - Wind speed/direction
  - Sunrise/sunset times (in UTC)

---

## Prerequisites

1. **Rust & Cargo**
   Ensure you have Rust (and Cargo) installed. You can install Rust using [rustup](https://www.rust-lang.org/tools/install).
2. **OpenWeatherMap API Key**
   Sign up for a free API key at [OpenWeatherMap](https://home.openweathermap.org/users/sign_up).

---

## Installation

1. **Clone the repository**:

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   ```

2. **Navigate to the `weather-cli` directory**:

   ```bash
   cd Rust/Code/weather-cli
   ```

3. **Create a `.env` file (optional)**:
   Create a file named `.env` in this directory and add the following line:

   ```dotenv
   OWM_API_KEY=your_openweathermap_api_key_here
   ```

   Alternatively, you may set this environment variable in your shell/session:

   ```bash
   export OWM_API_KEY=your_openweathermap_api_key_here
   ```

4. **Build and run**:

   ```bash
   cargo build
   cargo run
   ```

---

## Usage

1. **Run without arguments**:
   If no arguments are provided, the CLI will prompt you for a location (ZIP code or city):

   ```bash
   cargo run
   ```

2. **Run with a city name**:

   ```bash
   cargo run -- "London"
   ```

3. **Run with a ZIP code**:

   ```bash
   cargo run -- 10001
   ```

4. **Specify country code**:

   ```bash
   cargo run -- "London" -c uk
   ```

   or

   ```bash
   cargo run -- 10001 -c us
   ```

5. **Specify units**:
   - `imperial` (Fahrenheit)
   - `metric` (Celsius)
   - `standard` (Kelvin)

   Example:

   ```bash
   cargo run -- 10001 -c us -u metric
   ```

---

## Examples

```bash
# Fetch weather for London (defaults to country = us, units = imperial)
cargo run -- "London"

# Fetch weather for ZIP code 90001 in the US, using Celsius
cargo run -- 90001 -c us -u metric

# If OWM_API_KEY is not set in your environment, you will see an error message
```

---

## Project Structure

```bash
Rust
└── Code
    ├── file-commander
    ├── <other projects>
    └── weather-cli  <-- You are here
        ├── Cargo.toml
        ├── src
        │   └── main.rs
        ├── .env (optional)
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
