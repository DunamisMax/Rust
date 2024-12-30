use anyhow::{Context, Result};
use chrono::{TimeZone, Utc};
use clap::Parser;
use colored::*;
use dotenv::dotenv;
use rand::Rng;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::env;
use std::io::{self, Write};

/// Command-line arguments handled by Clap.
#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "A simple weather CLI using OpenWeatherMap API"
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

fn main() -> Result<()> {
    // Initialize .env if available
    dotenv().ok();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Print our fancy ASCII banner
    print_welcome_banner();

    // Retrieve API key
    let api_key = env::var("OWM_API_KEY")
        .context("Environment variable OWM_API_KEY not set. Please set it or store it in .env")?;

    // If no location was provided as an argument, prompt user interactively
    let location = match cli.location {
        Some(l) => l,
        None => prompt_for_location()?,
    };

    // Determine if the user entered a numeric ZIP code or a city
    let weather = if is_numeric(&location) {
        fetch_weather_zip(&location, &cli.country, &api_key, &cli.units)?
    } else {
        fetch_weather_city(&location, &cli.country, &api_key, &cli.units)?
    };

    // Print the resulting weather
    print_weather(&weather);

    Ok(())
}

/// Prints a banner with ASCII art in a random color.
fn print_welcome_banner() {
    let banner = r#"
                        _    _                         _  _
                       | |  | |                       | |(_)
__      __  ___   __ _ | |_ | |__    ___  _ __    ___ | | _
\ \ /\ / / / _ \ / _` || __|| '_ \  / _ \| '__|  / __|| || |
 \ V  V / |  __/| (_| || |_ | | | ||  __/| |    | (__ | || |
  \_/\_/   \___| \__,_| \__||_| |_| \___||_|     \___||_||_|

    "#;

    cprintln(banner);
    cprintln("Welcome to the Weather CLI!\n");
}

/// Prompts user for ZIP code or city name (only if CLI arg was not provided).
fn prompt_for_location() -> Result<String> {
    print!("Please enter a ZIP code or city name: ");
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

/// Fetches weather data by city name.
fn fetch_weather_city(
    city: &str,
    country: &str,
    api_key: &str,
    units: &str,
) -> Result<WeatherResponse> {
    // e.g., q=London,uk  => or if user doesn't want to specify country, omit it.
    let query_city = format!("{},{}", city, country);

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units={}",
        query_city, api_key, units
    );

    let client = Client::new();
    let resp = client
        .get(&url)
        .send()
        .with_context(|| format!("Failed to send request to URL: {url}"))?
        .error_for_status()
        .context("Received an error status code from OpenWeatherMap")?
        .json::<WeatherResponse>()
        .context("Failed to parse JSON response from OpenWeatherMap")?;

    Ok(resp)
}

/// Fetches weather data by ZIP code.
fn fetch_weather_zip(
    zip: &str,
    country: &str,
    api_key: &str,
    units: &str,
) -> Result<WeatherResponse> {
    // For US ZIP codes, you might do: zip=90001,us
    // If you want to handle multiple countries, pass them in from CLI arguments.
    let query_zip = format!("{},{}", zip, country);

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?zip={}&appid={}&units={}",
        query_zip, api_key, units
    );

    let client = Client::new();
    let resp = client
        .get(&url)
        .send()
        .with_context(|| format!("Failed to send request to URL: {url}"))?
        .error_for_status()
        .context("Received an error status code from OpenWeatherMap")?
        .json::<WeatherResponse>()
        .context("Failed to parse JSON response from OpenWeatherMap")?;

    Ok(resp)
}

/// Prints the weather data in a user-friendly format.
fn print_weather(weather: &WeatherResponse) {
    // Heading line
    println!(
        "\n{} in {}{}: {}, {}",
        "Current weather".bold().cyan(), // <-- heading in bold cyan
        weather.name,
        match &weather.sys {
            Some(sys) => match &sys.country {
                Some(country) => format!(", {}", country),
                None => "".to_string(),
            },
            None => "".to_string(),
        },
        weather.weather[0].main.bold().yellow(), // <-- main weather condition in bold yellow
        weather.weather[0].description
    );

    // Temperature
    println!(
        "{} {}",
        "Temperature:".bold().white(), // <-- label in bold white
        format!("{:.1}°F", weather.main.temp).bright_blue()  // <-- value in bright blue
    );

    // Feels like
    if let Some(feels_like) = weather.main.feels_like {
        println!(
            "{} {}",
            "Feels like:".bold().white(),
            format!("{:.1}°F", feels_like).bright_blue()
        );
    }

    // Min / max temperature
    if let Some(min_temp) = weather.main.temp_min {
        println!(
            "{} {}",
            "Minimum temperature:".bold().white(),
            format!("{:.1}°F", min_temp).bright_blue()
        );
    }
    if let Some(max_temp) = weather.main.temp_max {
        println!(
            "{} {}",
            "Maximum temperature:".bold().white(),
            format!("{:.1}°F", max_temp).bright_blue()
        );
    }

    // Pressure
    if let Some(pressure) = weather.main.pressure {
        println!(
            "{} {}",
            "Pressure:".bold().white(),
            format!("{} hPa", pressure).bright_blue()
        );
    }

    // Humidity
    println!(
        "{} {}",
        "Humidity:".bold().white(),
        format!("{}%", weather.main.humidity).bright_blue()
    );

    // Wind
    if let Some(wind) = &weather.wind {
        println!(
            "{} {}",
            "Wind speed:".bold().white(),
            format!("{:.1} mph", wind.speed).bright_blue()
        );
        if let Some(gust) = wind.gust {
            println!(
                "{} {}",
                "Wind gust:".bold().white(),
                format!("{:.1} mph", gust).bright_blue()
            );
        }
        if let Some(deg) = wind.deg {
            println!(
                "{} {}",
                "Wind direction:".bold().white(),
                format!("{}°", deg).bright_blue()
            );
        }
    }

    // Coordinates
    if let Some(coord) = &weather.coord {
        println!(
            "{} lat {}, lon {}",
            "Coordinates:".bold().white(),
            format!("{:.2}", coord.lat).bright_blue(),
            format!("{:.2}", coord.lon).bright_blue()
        );
    }

    // Sunrise / Sunset
    if let Some(sys) = &weather.sys {
        if let Some(sunrise) = sys.sunrise {
            println!(
                "{} {}",
                "Sunrise (UTC):".bold().white(),
                format_timestamp(sunrise).bright_magenta()
            );
        }
        if let Some(sunset) = sys.sunset {
            println!(
                "{} {}",
                "Sunset (UTC):".bold().white(),
                format_timestamp(sunset).bright_magenta()
            );
        }
    }
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

/// Prints colored text in a random color.
fn cprintln(text: &str) {
    let color = random_color();
    println!("{}", text.color(color));
}

/// Returns a random color from the `colored` crate.
fn random_color() -> colored::Color {
    let colors = [
        colored::Color::Red,
        colored::Color::Green,
        colored::Color::Yellow,
        colored::Color::Blue,
        colored::Color::Magenta,
        colored::Color::Cyan,
        colored::Color::White,
    ];
    let random_index = rand::thread_rng().gen_range(0..colors.len());
    colors[random_index]
}
