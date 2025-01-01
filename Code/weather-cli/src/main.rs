////////////////////////////////////////////////////////////////////////////////
// weather-cli - A Ratatui-based Weather CLI using Tokio, Clap, crossterm, etc.
////////////////////////////////////////////////////////////////////////////////

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

use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
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

#[derive(Debug, Parser)]
#[command(author, version, about = "A Weather CLI using Ratatui", long_about = None)]
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
    dotenv().ok();
    let args = Cli::parse();

    let api_key = env::var("OWM_API_KEY")
        .context("Environment variable OWM_API_KEY not set. Please set it or store it in .env.")?;

    // 1) Enable raw mode automatically via RAII guard.
    //    Once the guard is dropped (goes out of scope), raw mode is disabled.
    let _raw_guard = RawModeGuard::new().context("Failed to enable raw mode")?;

    // 2) Create Ratatui Terminal and clear screen
    let mut terminal = setup_terminal().context("Failed to create terminal")?;
    clear_screen(&mut terminal).context("Failed to clear terminal")?;

    // 3) Draw the Ratatui “Welcome” screen
    draw_welcome_screen(&mut terminal)?;

    // 4) Temporarily drop raw mode to allow normal keyboard input
    drop(_raw_guard);

    // 5) If user didn’t pass an input argument, prompt them for a location
    let location = match args.location {
        Some(loc) => loc,
        None => {
            // The Ratatui screen is still visible, but we’re in normal mode.
            // Type below the TUI lines:
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let trimmed = input.trim().to_string();
            if trimmed.is_empty() {
                "London".to_string()
            } else {
                trimmed
            }
        }
    };

    // 6) Fetch weather data
    let weather = if is_numeric(&location) {
        fetch_weather_zip(&location, &args.country, &api_key, &args.units).await?
    } else {
        fetch_weather_city(&location, &args.country, &api_key, &args.units).await?
    };

    // 7) Re-enable raw mode for the final TUI
    let _raw_guard = RawModeGuard::new().context("Failed to re-enable raw mode")?;

    // 8) Re-create the terminal, clear screen, and draw weather info
    let mut terminal = setup_terminal().context("Failed to create terminal")?;
    clear_screen(&mut terminal).context("Failed to clear terminal")?;
    draw_weather_info(&mut terminal, &weather)?;

    // 9) Disable raw mode so user can press Enter, then exit
    drop(_raw_guard);

    print!("   Press Enter to exit...{}", LINE_ENDING);
    io::stdout().flush()?;
    let mut exit_buf = String::new();
    io::stdin().read_line(&mut exit_buf)?;

    // 10) Final cleanup: clear screen, print goodbye
    execute!(terminal.backend_mut(), Clear(ClearType::All), MoveTo(0, 0))?;
    print!("Goodbye!{}", LINE_ENDING);

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// RAII guard for raw mode
////////////////////////////////////////////////////////////////////////////////

struct RawModeGuard {
    active: bool,
}

impl RawModeGuard {
    fn new() -> Result<Self> {
        enable_raw_mode().context("Unable to enable raw mode")?;
        Ok(Self { active: true })
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = disable_raw_mode();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Setup Terminal
////////////////////////////////////////////////////////////////////////////////

fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Clears the terminal screen
////////////////////////////////////////////////////////////////////////////////

fn clear_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    terminal.clear()?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Show the “Welcome” Ratatui with ASCII Banner
////////////////////////////////////////////////////////////////////////////////

fn draw_welcome_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    // You can replace this static ASCII art with figlet-rs output if preferred
    let banner_text = r#"
                         _____ ______                               ___________
___      _______ ______ ___  /____  /_ _____ ________        __________  /___(_)
__ | /| / /_  _ \_  __ `/_  __/__  __ \_  _ \__  ___/_________  ___/__  / __  /
__ |/ |/ / /  __// /_/ / / /_  _  / / //  __/_  /    _/_____// /__  _  /  _  /
____/|__/  \___/ \__,_/  \__/  /_/ /_/ \___/ /_/             \___/  /_/   /_/
"#;

    terminal.draw(|frame| {
        // NOTE: frame.area() instead of frame.size()
        let screen = frame.area();

        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(8), // ASCII banner
                Constraint::Length(1), // blank line
                Constraint::Length(1), // "Welcome to the Weather CLI!"
                Constraint::Length(1), // blank line
                Constraint::Length(2), // instructions
                Constraint::Length(1), // blank line
                Constraint::Length(1), // prompt arrow
            ])
            .split(screen);

        // chunk[0]: ASCII Banner
        let banner_lines: Vec<Line> = banner_text
            .lines()
            .map(|line| {
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            })
            .collect();
        let banner_paragraph = Paragraph::new(banner_lines)
            .block(Block::default())
            .alignment(Alignment::Left);
        frame.render_widget(banner_paragraph, vertical_chunks[0]);

        // chunk[1]: blank line
        let blank_paragraph = Paragraph::new(Line::from("")).block(Block::default());
        frame.render_widget(blank_paragraph, vertical_chunks[1]);

        // chunk[2]: "Welcome to the Weather CLI!"
        let welcome_line = Paragraph::new(Line::from("Welcome to the Weather CLI!"))
            .alignment(Alignment::Left)
            .block(Block::default());
        frame.render_widget(welcome_line, vertical_chunks[2]);

        // chunk[3]: blank line
        let blank_paragraph = Paragraph::new(Line::from("")).block(Block::default());
        frame.render_widget(blank_paragraph, vertical_chunks[3]);

        // chunk[4]: instructions
        let instruction_lines = vec![
            Line::from("Please enter a ZIP code or city name below:"),
            Line::from("Example: \"90001\" or \"London\""),
        ];
        let instruction_p = Paragraph::new(instruction_lines)
            .alignment(Alignment::Left)
            .block(Block::default());
        frame.render_widget(instruction_p, vertical_chunks[4]);

        // chunk[5]: blank line
        let blank_paragraph = Paragraph::new(Line::from("")).block(Block::default());
        frame.render_widget(blank_paragraph, vertical_chunks[5]);

        // chunk[6]: ">"
        let arrow_line = Paragraph::new(Line::from(">"))
            .alignment(Alignment::Left)
            .block(Block::default());
        frame.render_widget(arrow_line, vertical_chunks[6]);
    })?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Check if input is numeric (ZIP) or not (city)
////////////////////////////////////////////////////////////////////////////////

fn is_numeric(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_digit())
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Fetch weather by city
////////////////////////////////////////////////////////////////////////////////

async fn fetch_weather_city(
    city: &str,
    country: &str,
    api_key: &str,
    units: &str,
) -> Result<WeatherResponse> {
    let query_city = format!("{city},{country}");
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

////////////////////////////////////////////////////////////////////////////////
// Utility: Fetch weather by ZIP
////////////////////////////////////////////////////////////////////////////////

async fn fetch_weather_zip(
    zip: &str,
    country: &str,
    api_key: &str,
    units: &str,
) -> Result<WeatherResponse> {
    let query_zip = format!("{zip},{country}");
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
// Utility: Draw the weather info TUI
////////////////////////////////////////////////////////////////////////////////

fn draw_weather_info(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    weather: &WeatherResponse,
) -> Result<()> {
    // Build lines for the TUI
    let heading = format!(
        "Current weather in {}{}",
        weather.name,
        weather
            .sys
            .as_ref()
            .and_then(|s| s.country.as_ref())
            .map(|cc| format!(", {cc}"))
            .unwrap_or_default()
    );

    let mut lines: Vec<Line> = vec![];
    lines.push(Line::from(Span::styled(
        heading,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));

    if let Some(desc) = weather.weather.first() {
        let cond_str = format!("Condition: {} ({})", desc.main, desc.description);
        lines.push(Line::from(Span::styled(
            cond_str,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
    }

    // Temperature data
    lines.push(Line::from(Span::styled(
        format!("Temperature: {:.1}°", weather.main.temp),
        Style::default().fg(Color::Blue),
    )));

    if let Some(fl) = weather.main.feels_like {
        lines.push(Line::from(Span::styled(
            format!("Feels like: {:.1}°", fl),
            Style::default().fg(Color::Blue),
        )));
    }
    if let Some(min) = weather.main.temp_min {
        lines.push(Line::from(Span::styled(
            format!("Min temp: {:.1}°", min),
            Style::default().fg(Color::Blue),
        )));
    }
    if let Some(max) = weather.main.temp_max {
        lines.push(Line::from(Span::styled(
            format!("Max temp: {:.1}°", max),
            Style::default().fg(Color::Blue),
        )));
    }

    if let Some(p) = weather.main.pressure {
        lines.push(Line::from(Span::styled(
            format!("Pressure: {} hPa", p),
            Style::default().fg(Color::Blue),
        )));
    }

    lines.push(Line::from(Span::styled(
        format!("Humidity: {}%", weather.main.humidity),
        Style::default().fg(Color::Blue),
    )));

    // Wind data
    if let Some(wind) = &weather.wind {
        lines.push(Line::from(Span::styled(
            format!("Wind speed: {:.1} mph", wind.speed),
            Style::default().fg(Color::Blue),
        )));
        if let Some(g) = wind.gust {
            lines.push(Line::from(Span::styled(
                format!("Wind gust: {:.1} mph", g),
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

    // A blank line
    lines.push(Line::from(""));

    terminal.draw(|frame| {
        // Use .area() instead of deprecated .size()
        let screen = frame.area();

        let block = Block::default().borders(Borders::ALL).title("Weather");
        let paragraph = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, screen);
    })?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Format timestamps
////////////////////////////////////////////////////////////////////////////////

fn format_timestamp(ts: u64) -> String {
    match Utc.timestamp_opt(ts as i64, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        _ => "Invalid timestamp".to_string(),
    }
}
