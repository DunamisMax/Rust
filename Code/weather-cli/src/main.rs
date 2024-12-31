////////////////////////////////////////////////////////////////////////////////
// Imports
////////////////////////////////////////////////////////////////////////////////

use anyhow::{Context, Result};
use chrono::{TimeZone, Utc};
use clap::Parser;
use dotenv::dotenv;
use reqwest::Client;
use serde::Deserialize;
use std::{
    env,
    io::{self, Write},
};

// Crossterm + ratatui
use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

////////////////////////////////////////////////////////////////////////////////
// Cross-Platform Line Endings
////////////////////////////////////////////////////////////////////////////////

#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";

#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";

////////////////////////////////////////////////////////////////////////////////
// CLI Arguments
////////////////////////////////////////////////////////////////////////////////

/// A simple async weather CLI using the OpenWeatherMap API
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
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

////////////////////////////////////////////////////////////////////////////////
// JSON Models for Deserialization
////////////////////////////////////////////////////////////////////////////////

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

#[derive(Debug, Deserialize)]
struct WeatherDescription {
    main: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct MainData {
    temp: f64,
    feels_like: Option<f64>,
    temp_min: Option<f64>,
    temp_max: Option<f64>,
    pressure: Option<f64>,
    humidity: f64,
}

#[derive(Debug, Deserialize)]
struct WindData {
    speed: f64,
    gust: Option<f64>,
    deg: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct SysData {
    country: Option<String>,
    sunrise: Option<u64>,
    sunset: Option<u64>,
}

////////////////////////////////////////////////////////////////////////////////
// Main (Tokio) Entry Point
////////////////////////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<()> {
    // Enable raw mode for our TUI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Clear the screen & display a banner
    clear_screen(&mut terminal)?;
    print_welcome_banner(&mut terminal)?;

    // Load .env if present
    dotenv().ok();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Retrieve API key from environment or fail gracefully
    let api_key = env::var("OWM_API_KEY")
        .context("Environment variable OWM_API_KEY not set. Please set it or store it in .env.")?;

    // Prompt user if location is not provided
    let location = match cli.location {
        Some(loc) => loc,
        None => prompt_for_location()?,
    };

    // Decide fetch strategy (ZIP or city name)
    let weather = if is_numeric(&location) {
        fetch_weather_zip(&location, &cli.country, &api_key, &cli.units).await?
    } else {
        fetch_weather_city(&location, &cli.country, &api_key, &cli.units).await?
    };

    // Display the results
    print_weather(&mut terminal, &weather)?;

    // Pause so user can see the output
    print!("Press Enter to exit...{}", LINE_ENDING);
    io::stdout().flush()?;
    let mut exit_buf = String::new();
    io::stdin().read_line(&mut exit_buf)?;

    // Restore the terminal to normal mode
    disable_raw_mode()?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Terminal UI Helpers
////////////////////////////////////////////////////////////////////////////////

/// Clears the terminal screen using ratatui's Terminal API.
fn clear_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    terminal.clear()?;
    Ok(())
}

/// Prints a multi-line ASCII banner at the top using ratatui widgets.
fn print_welcome_banner(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    terminal.draw(|f| {
        let size = f.area();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(size);

        let banner_text = r#"
                        _    _                         _  _
                       | |  | |                       | |(_)
__      __  ___   __ _ | |_ | |__    ___  _ __    ___ | | _
\ \ /\ / / / _ \ / _` || __|| '_ \  / _ \| '__|  / __|| || |
 \ V  V / |  __/| (_| || |_ | | | ||  __/| |    | (__ | || |
  \_/\_/   \___| \__,_| \__||_| |_| \___||_|     \___||_||_|
"#;

        let banner = Paragraph::new(banner_text).style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
        f.render_widget(banner, layout[0]);
    })?;

    print!("Welcome to the Weather CLI!{}", LINE_ENDING);
    print!("{}", LINE_ENDING);
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Prompt for Location
////////////////////////////////////////////////////////////////////////////////

/// If no city/ZIP was provided, interactively prompt user.
fn prompt_for_location() -> Result<String> {
    print!("Please enter a ZIP code or city name: {}", LINE_ENDING);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim().to_string();

    // Provide a default if the user pressed Enter without typing anything
    if trimmed.is_empty() {
        Ok("London".to_string())
    } else {
        Ok(trimmed)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Fetching Weather Data
////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////
// Display Logic
////////////////////////////////////////////////////////////////////////////////

/// Renders the fetched weather info using ratatui’s widget system.
fn print_weather(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    weather: &WeatherResponse,
) -> Result<()> {
    // Construct a heading for the city/country
    let heading = format!(
        "Current weather in {}{}",
        weather.name,
        weather
            .sys
            .as_ref()
            .and_then(|sys| sys.country.as_ref())
            .map(|cc| format!(", {cc}"))
            .unwrap_or_default(),
    );

    // Start building lines for our Paragraph
    let mut lines = Vec::new();

    // Heading line: cyan + bold
    lines.push(Line::from(Span::styled(
        heading,
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    // Condition
    if let Some(description) = weather.weather.get(0) {
        let cond_str = format!("Condition: {} ({})", description.main, description.description);
        lines.push(Line::from(Span::styled(
            cond_str,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
    }

    // Temperature
    let temp_str = format!("Temperature: {:.1}°F", weather.main.temp);
    lines.push(Line::from(Span::styled(temp_str, Style::default().fg(Color::Blue))));

    // Feels like
    if let Some(fl) = weather.main.feels_like {
        let feels_str = format!("Feels like: {:.1}°F", fl);
        lines.push(Line::from(Span::styled(
            feels_str,
            Style::default().fg(Color::Blue),
        )));
    }

    // Min / Max
    if let Some(min_temp) = weather.main.temp_min {
        lines.push(Line::from(Span::styled(
            format!("Min temp: {:.1}°F", min_temp),
            Style::default().fg(Color::Blue),
        )));
    }
    if let Some(max_temp) = weather.main.temp_max {
        lines.push(Line::from(Span::styled(
            format!("Max temp: {:.1}°F", max_temp),
            Style::default().fg(Color::Blue),
        )));
    }

    // Pressure
    if let Some(pressure) = weather.main.pressure {
        lines.push(Line::from(Span::styled(
            format!("Pressure: {} hPa", pressure),
            Style::default().fg(Color::Blue),
        )));
    }

    // Humidity
    lines.push(Line::from(Span::styled(
        format!("Humidity: {}%", weather.main.humidity),
        Style::default().fg(Color::Blue),
    )));

    // Wind info
    if let Some(wind) = &weather.wind {
        lines.push(Line::from(Span::styled(
            format!("Wind speed: {:.1} mph", wind.speed),
            Style::default().fg(Color::Blue),
        )));
        if let Some(gust) = wind.gust {
            lines.push(Line::from(Span::styled(
                format!("Wind gust: {:.1} mph", gust),
                Style::default().fg(Color::Blue),
            )));
        }
        if let Some(deg) = wind.deg {
            lines.push(Line::from(Span::styled(
                format!("Wind direction: {}°", deg),
                Style::default().fg(Color::Blue),
            )));
        }
    }

    // Coordinates
    if let Some(coord) = &weather.coord {
        lines.push(Line::from(Span::styled(
            format!("Coordinates: lat {:.2}, lon {:.2}", coord.lat, coord.lon),
            Style::default().fg(Color::Blue),
        )));
    }

    // Sunrise / Sunset
    if let Some(sys) = &weather.sys {
        if let Some(sr) = sys.sunrise {
            lines.push(Line::from(Span::styled(
                format!("Sunrise (UTC): {}", format_timestamp(sr)),
                Style::default().fg(Color::Magenta),
            )));
        }
        if let Some(ss) = sys.sunset {
            lines.push(Line::from(Span::styled(
                format!("Sunset (UTC): {}", format_timestamp(ss)),
                Style::default().fg(Color::Magenta),
            )));
        }
    }

    // Blank line at end
    lines.push(Line::from(""));

    // Render with a standard block border
    terminal.draw(|f| {
        let size = f.area();
        let block = Block::default().borders(Borders::ALL).title("Weather");
        let paragraph = Paragraph::new(lines).block(block);
        f.render_widget(paragraph, size);
    })?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Utilities
////////////////////////////////////////////////////////////////////////////////

/// Returns true if `s` consists only of ASCII digits.
fn is_numeric(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

/// Formats a Unix timestamp into a human-readable UTC time (YYYY-MM-DD HH:MM:SS).
fn format_timestamp(timestamp: u64) -> String {
    let timestamp_i64 = timestamp as i64;
    let datetime = Utc
        .timestamp_opt(timestamp_i64, 0)
        .single()
        .unwrap_or_else(|| Utc.timestamp_opt(0, 0).single().unwrap());
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}
