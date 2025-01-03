use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::env;
use chrono::{TimeZone, Utc};

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

/// Wind data (speed in mph when using imperial units).
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
    // Load .env if available (comment out if using system environment variables)
    dotenv::dotenv().ok();

    // Retrieve the API key from environment variables
    let api_key = env::var("OWM_API_KEY")
        .map_err(|_| anyhow!("Environment variable OWM_API_KEY not set"))?;

    // Parse the city name from CLI arguments. Expect 1 argument, e.g. "Paris".
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <CITY_NAME>", args[0]);
        std::process::exit(1);
    }
    let city_name = &args[1];

    // Fetch weather data
    let weather = fetch_weather(city_name, &api_key)?;

    // Print general weather info
    println!(
        "\nCurrent weather in {}{}: {}, {}",
        weather.name,
        match &weather.sys {
            Some(sys) => match &sys.country {
                Some(country) => format!(", {}", country),
                None => "".to_string(),
            },
            None => "".to_string(),
        },
        weather.weather[0].main,
        weather.weather[0].description
    );

    // Print temperature details
    println!("Temperature (F): {:.1}", weather.main.temp);
    if let Some(feels_like) = weather.main.feels_like {
        println!("Feels like (F): {:.1}", feels_like);
    }
    if let Some(min_temp) = weather.main.temp_min {
        println!("Minimum temperature (F): {:.1}", min_temp);
    }
    if let Some(max_temp) = weather.main.temp_max {
        println!("Maximum temperature (F): {:.1}", max_temp);
    }

    // Print other atmospheric data
    if let Some(pressure) = weather.main.pressure {
        println!("Pressure: {} hPa", pressure);
    }
    println!("Humidity: {}%", weather.main.humidity);

    // Print wind data
    if let Some(wind) = weather.wind {
        println!("Wind speed: {:.1} mph", wind.speed);
        if let Some(gust) = wind.gust {
            println!("Wind gust: {:.1} mph", gust);
        }
        if let Some(deg) = wind.deg {
            println!("Wind direction: {}°", deg);
        }
    }

    // Print coordinates, sunrise, and sunset if available
    if let Some(coord) = &weather.coord {
        println!("Coordinates: lat {:.2}, lon {:.2}", coord.lat, coord.lon);
    }
    if let Some(sys) = weather.sys {
        if let Some(sunrise) = sys.sunrise {
            println!("Sunrise (UTC): {}", format_timestamp(sunrise));
        }
        if let Some(sunset) = sys.sunset {
            println!("Sunset (UTC): {}", format_timestamp(sunset));
        }
    }

    Ok(())
}

/// Fetches weather data from OpenWeatherMap using imperial units (Fahrenheit, mph).
///
/// # Arguments
///
/// * `city` - The city name, e.g. "London".
/// * `api_key` - Your OpenWeatherMap API key.
///
/// # Returns
///
/// A `WeatherResponse` struct containing current weather data.
///
/// # Errors
///
/// Returns an `anyhow::Error` if the request fails or if the JSON is invalid.
fn fetch_weather(city: &str, api_key: &str) -> Result<WeatherResponse> {
    let client = Client::new();

    // Construct the request URL with "imperial" unit system
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=imperial",
        city, api_key
    );

    // Perform the GET request and parse JSON
    let resp = client
        .get(&url)
        .send()
        .map_err(|e| anyhow!("Failed to send request: {}", e))?
        .error_for_status() // convert HTTP errors into a Rust error
        .map_err(|e| anyhow!("Received an error HTTP status code: {}", e))?
        .json::<WeatherResponse>()
        .map_err(|e| anyhow!("Failed to parse JSON: {}", e))?;

    Ok(resp)
}

/// Helper function to format a Unix timestamp into a readable UTC time without deprecation warnings.
fn format_timestamp(timestamp: u64) -> String {
    // Convert `timestamp` (u64) to `i64` safely (assuming it's in range).
    let timestamp_i64 = timestamp as i64;

    // Use the newer, recommended `timestamp_opt` method on Utc.
    // This returns a `LocalResult<DateTime<Utc>>`.
    let datetime = Utc.timestamp_opt(timestamp_i64, 0)
        .single()
        .unwrap_or_else(|| {
            // If invalid or out of range, fallback to a default of 0 UNIX epoch.
            Utc.timestamp_opt(0, 0).single().unwrap()
        });

    // Format to something readable, e.g., "2024-12-28 13:45:00 UTC"
    datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}