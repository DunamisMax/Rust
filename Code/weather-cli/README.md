# Weather CLI

A simple command-line application written in **Rust** that fetches current weather information for a given city using the [OpenWeatherMap API](https://openweathermap.org/api). It displays temperature (in Fahrenheit), wind speed, gusts, humidity, pressure, sunrise, sunset times, and more.

---

## Features

- **Imperial Units**: Displays temperature in Fahrenheit and wind speed in mph.
- **Extended Weather Data**: Fetches “feels like,” minimum/maximum temperature, wind gust, and sunrise/sunset times.
- **Robust Error Handling**: Uses [Anyhow](https://crates.io/crates/anyhow) for simplified error reporting.
- **Environment Variable**: Stores the API key in an environment variable (`OWM_API_KEY`), which should **not** be committed to version control.
- **Safe and Idiomatic Rust**: Leverages the Rust 2021 edition, `reqwest` with blocking calls, and `serde` for JSON parsing.

---

## Prerequisites

1. **Rust** (1.60 or newer recommended)
2. An **OpenWeatherMap API Key**
3. A **.env** file or equivalent environment variable setup.

---

## Installation & Setup

1. **Clone the repository** (or download the source code):

   ```bash
   git clone https://github.com/dunamismax/Rust.git
   cd /Rust/Code/weather-cli
   ```

2. **Set your OpenWeatherMap API key**:

   - **Via a `.env` file** (recommended for local development):

     ```bash
     echo "OWM_API_KEY=YOUR_API_KEY_HERE" > .env
     ```

     Make sure `.env` is added to your `.gitignore`.
   - **OR via system environment variable** (CI, production):

     ```bash
     export OWM_API_KEY=YOUR_API_KEY_HERE
     ```

3. **Build the project**:

   ```bash
   cargo build --release
   ```

4. **Run the CLI**:

   ```bash
   ./target/release/weather-cli "London"
   ```

   Replace `"London"` with any city of your choice (e.g., `"Paris"`, `"New York"`).

---

## Usage

- **Basic Command**:

  ```bash
  weather-cli <CITY_NAME>
  ```

  Example:

  ```bash
  weather-cli "Berlin"
  ```

  This will display the current weather in Berlin, including temperature (°F), humidity, wind speed, gust, sunrise/sunset times, etc.

- **Error Handling**:
    - If the city name is not found, you’ll see a descriptive error from the OpenWeatherMap API.
    - If the `OWM_API_KEY` environment variable is not set, the app exits with an error message.

---

## Example Output

Here’s a sample output for a call like `weather-cli "London"`:

```plaintext
Current weather in London, GB: Clouds, overcast clouds
Temperature (F): 61.3
Feels like (F): 59.7
Minimum temperature (F): 60.8
Maximum temperature (F): 62.6
Pressure: 1012 hPa
Humidity: 80%
Wind speed: 5.3 mph
Wind gust: 10.2 mph
Wind direction: 220°
Coordinates: lat 51.51, lon -0.13
Sunrise (UTC): 2024-12-28 07:58:00 UTC
Sunset (UTC): 2024-12-28 15:54:00 UTC
```

---

## Project Structure

```
weather-cli
├── Cargo.toml
├── .gitignore
├── .env            # Not committed (example file to store OWM_API_KEY)
└── src
    └── main.rs     # Primary CLI logic
```

- **`Cargo.toml`**: Manifest file specifying dependencies, features, and project metadata.
- **`main.rs`**: Contains the CLI logic, struct definitions, API calls, and printing of weather data.

---

## Contributing

1. **Fork** the repository.
2. **Create** your feature branch (`git checkout -b feature/amazing-feature`).
3. **Commit** your changes (`git commit -m 'Add amazing feature'`).
4. **Push** to the branch (`git push origin feature/amazing-feature`).
5. Create a **Pull Request**.

---

## License

This project is licensed under the [MIT License](LICENSE). Feel free to use, modify, and distribute it as you see fit.

---

**Enjoy fetching weather data in Rust!** If you find any issues or have suggestions for improvement, please open an [issue](https://github.com/your-username/weather-cli/issues) or submit a pull request.
