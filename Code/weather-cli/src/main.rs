//! A modernized, async Weather CLI application in Rust using Tokio + crossterm
//! for a clean terminal experience with minimal color theming.

use anyhow::{Context, Result};
use chrono::{TimeZone, Utc};
use clap::Parser;
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Stylize},
    terminal::{Clear, ClearType},
};
use dotenv::dotenv;
use reqwest::Client;
use serde::Deserialize;
use std::{
    env,
    io::{self, Write},
};

/// Command-line arguments handled by Clap.
#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "A simple async weather CLI using the OpenWeatherMap API"
)]
struct Cli {
    /// The location to query; can be a city name or ZIP code
    #[arg(required = false)]
    location: Option<String>,

    /// The country code (optional), e.g., "us", "uk", "de", etc.
    #[arg(short, long, default_value = "us")]
    country: String,

    /// Units of measurement: "metric" (Celsius), "imperial" (Fahrenheit), or "standard" (Kelvin)
    #[arg(short, long, default_value = "imperial")]
    units: String,
}

/// Full response from OpenWeatherMap (partial subset of fields).
#[derive(Debug, Deserialize)]
struct WeatherResponse {
    coord: Option<Coord>,
    weather: Vec<WeatherDescription>,
    main: MainData,
    wind: Option<WindData>,
    sys: Option<SysData>,
    name: String,
}

/// Coordinates for location.
#[derive(Debug, Deserialize)]
struct Coord {
    lon: f64,
    lat: f64,
}

/// Represents weather conditions.
#[derive(Debug, Deserialize)]
struct WeatherDescription {
    main: String,
    description: String,
}

/// Main temperature and atmospheric data.
#[derive(Debug, Deserialize)]
struct MainData {
    temp: f64,
    feels_like: Option<f64>,
    temp_min: Option<f64>,
    temp_max: Option<f64>,
    pressure: Option<f64>,
    humidity: f64,
}

/// Wind data (speed in mph for imperial, m/s for metric).
#[derive(Debug, Deserialize)]
struct WindData {
    speed: f64,
    gust: Option<f64>,
    deg: Option<f64>, // direction in degrees
}

/// Additional system data like sunrise/sunset.
#[derive(Debug, Deserialize)]
struct SysData {
    country: Option<String>,
    sunrise: Option<u64>,
    sunset: Option<u64>,
}

/// Asynchronous entry point using Tokio.
#[tokio::main]
async fn main() -> Result<()> {
    clear_screen()?;
    print_welcome_banner()?;

    // Load environment variables from .env if present
    dotenv().ok();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Retrieve API key
    let api_key = env::var("OWM_API_KEY")
        .context("Environment variable OWM_API_KEY not set. Please set it or store it in .env.")?;

    // If no location was provided as an argument, prompt user interactively
    let location = match cli.location {
        Some(l) => l,
        None => prompt_for_location()?,
    };

    // Decide which fetch strategy to use (city vs. zip)
    let weather = if is_numeric(&location) {
        fetch_weather_zip(&location, &cli.country, &api_key, &cli.units).await?
    } else {
        fetch_weather_city(&location, &cli.country, &api_key, &cli.units).await?
    };

    // Print the resulting weather
    print_weather(&weather);

    Ok(())
}

/// Clears the terminal screen for a clean start using crossterm.
fn clear_screen() -> Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    Ok(())
}

/// Prints a banner with ASCII art in a consistent color theme.
fn print_welcome_banner() -> Result<()> {
    let banner = r#"
                        _    _                         _  _
                       | |  | |                       | |(_)
__      __  ___   __ _ | |_ | |__    ___  _ __    ___ | | _
\ \ /\ / / / _ \ / _` || __|| '_ \  / _ \| '__|  / __|| || |
 \ V  V / |  __/| (_| || |_ | | | ||  __/| |    | (__ | || |
  \_/\_/   \___| \__,_| \__||_| |_| \___||_|     \___||_||_|
    "#;

    // Use crossterm's Stylize to color the banner
    let styled_banner = banner.with(Color::Cyan).bold();
    print!("{}\r\n", styled_banner);

    // A simple introduction line
    let intro = "Welcome to the Weather CLI!".with(Color::Magenta).bold();
    print!("{}\r\n\r\n", intro);

    Ok(())
}

/// Prompts user for ZIP code or city name (only if CLI arg was not provided).
fn prompt_for_location() -> Result<String> {
    print!("Please enter a ZIP code or city name: \r\n");
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read input")?;

    let trimmed = input.trim().to_string();
    if trimmed.is_empty() {
        // Provide a default if empty
        Ok("London".to_string())
    } else {
        Ok(trimmed)
    }
}

/// Fetches weather data by city name asynchronously.
async fn fetch_weather_city(
    city: &str,
    country: &str,
    api_key: &str,
    units: &str,
) -> Result<WeatherResponse> {
    let query_city = format!("{},{}", city, country);
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units={}",
        query_city, api_key, units
    );

    let client = Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to send request to URL: {url}"))?
        .error_for_status()
        .context("Received an error status code from OpenWeatherMap")?
        .json::<WeatherResponse>()
        .await
        .context("Failed to parse JSON response from OpenWeatherMap")?;

    Ok(resp)
}

/// Fetches weather data by ZIP code asynchronously.
async fn fetch_weather_zip(
    zip: &str,
    country: &str,
    api_key: &str,
    units: &str,
) -> Result<WeatherResponse> {
    let query_zip = format!("{},{}", zip, country);
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?zip={}&appid={}&units={}",
        query_zip, api_key, units
    );

    let client = Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to send request to URL: {url}"))?
        .error_for_status()
        .context("Received an error status code from OpenWeatherMap")?
        .json::<WeatherResponse>()
        .await
        .context("Failed to parse JSON response from OpenWeatherMap")?;

    Ok(resp)
}

/// Prints the weather data in a user-friendly format using crossterm color styling.
fn print_weather(weather: &WeatherResponse) {
    // Heading
    let heading = "Current weather".bold().with(Color::Cyan);
    let condition = weather.weather[0].main.clone().bold().with(Color::Yellow);

    // Intro line
    print!(
        "{} in {}{}: {}, {}\r\n",
        heading,
        weather.name,
        match &weather.sys {
            Some(sys) => match &sys.country {
                Some(country) => format!(", {}", country),
                None => "".to_string(),
            },
            None => "".to_string(),
        },
        condition,
        weather.weather[0].description
    );

    // Temperature
    let temp_label = "Temperature:".bold().with(Color::White);
    let temp_value = format!("{:.1}°F", weather.main.temp).with(Color::Blue);
    print!("{} {}\r\n", temp_label, temp_value);

    // Feels like
    if let Some(feels_like) = weather.main.feels_like {
        let feels_label = "Feels like:".bold().with(Color::White);
        let feels_value = format!("{:.1}°F", feels_like).with(Color::Blue);
        print!("{} {}\r\n", feels_label, feels_value);
    }

    // Min / max temperature
    if let Some(min_temp) = weather.main.temp_min {
        let min_label = "Minimum temperature:".bold().with(Color::White);
        let min_value = format!("{:.1}°F", min_temp).with(Color::Blue);
        print!("{} {}\r\n", min_label, min_value);
    }
    if let Some(max_temp) = weather.main.temp_max {
        let max_label = "Maximum temperature:".bold().with(Color::White);
        let max_value = format!("{:.1}°F", max_temp).with(Color::Blue);
        print!("{} {}\r\n", max_label, max_value);
    }

    // Pressure
    if let Some(pressure) = weather.main.pressure {
        let pressure_label = "Pressure:".bold().with(Color::White);
        let pressure_value = format!("{} hPa", pressure).with(Color::Blue);
        print!("{} {}\r\n", pressure_label, pressure_value);
    }

    // Humidity
    let hum_label = "Humidity:".bold().with(Color::White);
    let hum_value = format!("{}%", weather.main.humidity).with(Color::Blue);
    print!("{} {}\r\n", hum_label, hum_value);

    // Wind
    if let Some(wind) = &weather.wind {
        let wind_speed_label = "Wind speed:".bold().with(Color::White);
        let wind_speed_val = format!("{:.1} mph", wind.speed).with(Color::Blue);
        print!("{} {}\r\n", wind_speed_label, wind_speed_val);

        if let Some(gust) = wind.gust {
            let wind_gust_label = "Wind gust:".bold().with(Color::White);
            let wind_gust_val = format!("{:.1} mph", gust).with(Color::Blue);
            print!("{} {}\r\n", wind_gust_label, wind_gust_val);
        }

        if let Some(deg) = wind.deg {
            let wind_dir_label = "Wind direction:".bold().with(Color::White);
            let wind_dir_val = format!("{}°", deg).with(Color::Blue);
            print!("{} {}\r\n", wind_dir_label, wind_dir_val);
        }
    }

    // Coordinates
    if let Some(coord) = &weather.coord {
        let coord_label = "Coordinates:".bold().with(Color::White);
        let lat_val = format!("{:.2}", coord.lat).with(Color::Blue);
        let lon_val = format!("{:.2}", coord.lon).with(Color::Blue);
        print!("{} lat {}, lon {}\r\n", coord_label, lat_val, lon_val);
    }

    // Sunrise / Sunset
    if let Some(sys) = &weather.sys {
        if let Some(sunrise) = sys.sunrise {
            let sunrise_label = "Sunrise (UTC):".bold().with(Color::White);
            let sunrise_val = format_timestamp(sunrise).with(Color::Magenta);
            print!("{} {}\r\n", sunrise_label, sunrise_val);
        }
        if let Some(sunset) = sys.sunset {
            let sunset_label = "Sunset (UTC):".bold().with(Color::White);
            let sunset_val = format_timestamp(sunset).with(Color::Magenta);
            print!("{} {}\r\n", sunset_label, sunset_val);
        }
    }
    print!("\r\n");
}

/// Helper function to format a Unix timestamp into a readable UTC time.
fn format_timestamp(timestamp: u64) -> String {
    let timestamp_i64 = timestamp as i64;
    let datetime = Utc
        .timestamp_opt(timestamp_i64, 0)
        .single()
        .unwrap_or_else(|| Utc.timestamp_opt(0, 0).single().unwrap());
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Returns true if the string consists only of digits.
fn is_numeric(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}
