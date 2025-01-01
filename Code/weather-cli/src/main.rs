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
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
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
    dotenv().ok();
    let cli = Cli::parse();

    let api_key = env::var("OWM_API_KEY")
        .context("Environment variable OWM_API_KEY not set. Please set it or store it in .env.")?;

    //
    // 1) Show the TUI welcome layout (raw mode ON)
    //
    enable_raw_mode()?;
    let mut terminal = setup_terminal()?;
    clear_screen(&mut terminal)?;
    draw_welcome_screen(&mut terminal)?;
    disable_raw_mode()?; // Turn off raw mode so we can read from stdin normally.

    //
    // 2) Prompt for location on the console
    //
    let location = match cli.location {
        Some(loc) => loc,
        None => {
            // Now user sees the TUI with the ">" prompt line. We read console input below it.
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

    //
    // 3) Fetch weather data
    //
    let weather = if is_numeric(&location) {
        fetch_weather_zip(&location, &cli.country, &api_key, &cli.units).await?
    } else {
        fetch_weather_city(&location, &cli.country, &api_key, &cli.units).await?
    };

    //
    // 4) Show the weather info in TUI again
    //
    enable_raw_mode()?;
    let mut terminal = setup_terminal()?;
    clear_screen(&mut terminal)?;
    draw_weather_info(&mut terminal, &weather)?;

    // Pause so user can see output
    disable_raw_mode()?;
    print!("   Press Enter to exit...{}", LINE_ENDING);
    io::stdout().flush()?;
    let mut exit_buf = String::new();
    io::stdin().read_line(&mut exit_buf)?;

    // Cleanup: clear screen, print goodbye
    execute!(terminal.backend_mut(), Clear(ClearType::All), MoveTo(0, 0))?;
    print!("Goodbye!{}", LINE_ENDING);

    Ok(())
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
// Utility: Clears the screen
////////////////////////////////////////////////////////////////////////////////

fn clear_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    terminal.clear()?;
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Show ASCII banner + multiple lines
////////////////////////////////////////////////////////////////////////////////

fn draw_welcome_screen(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    let banner_text = r#"
                         _____ ______                               ___________
___      _______ ______ ___  /____  /_ _____ ________        __________  /___(_)
__ | /| / /_  _ \_  __ `/_  __/__  __ \_  _ \__  ___/_________  ___/__  / __  /
__ |/ |/ / /  __// /_/ / / /_  _  / / //  __/_  /    _/_____// /__  _  /  _  /
____/|__/  \___/ \__,_/  \__/  /_/ /_/ \___/ /_/             \___/  /_/   /_/
"#;

    // We'll define multiple vertical chunks:
    //   - chunk[0]: the 8-line banner
    //   - chunk[1]: blank line
    //   - chunk[2]: "Welcome to the Weather CLI!"
    //   - chunk[3]: blank line
    //   - chunk[4]: "Please enter a ZIP code..."
    //   - chunk[5]: blank line
    //   - chunk[6]: ">"
    //
    // So the user sees each line separated by blank lines.
    terminal.draw(|frame| {
        let size = frame.size();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(8), // banner
                Constraint::Length(1), // blank
                Constraint::Length(1), // welcome line
                Constraint::Length(1), // blank
                Constraint::Length(1), // prompt line
                Constraint::Length(1), // blank
                Constraint::Length(1), // ">"
            ])
            .split(size);

        // chunk[0]: ASCII banner
        let banner_lines = banner_text
            .lines()
            .map(|line| {
                Spans::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            })
            .collect::<Vec<_>>();
        let banner_paragraph = Paragraph::new(banner_lines)
            .alignment(Alignment::Left)
            .block(Block::default());
        frame.render_widget(banner_paragraph, layout[0]);

        // chunk[1]: blank line => just render an empty paragraph
        let blank_paragraph = Paragraph::new("").block(Block::default());
        frame.render_widget(blank_paragraph, layout[1]);

        // chunk[2]: "Welcome to the Weather CLI!"
        let welcome_paragraph = Paragraph::new("Welcome to the Weather CLI!")
            .alignment(Alignment::Left)
            .block(Block::default());
        frame.render_widget(welcome_paragraph, layout[2]);

        // chunk[3]: blank line
        let blank_paragraph = Paragraph::new("").block(Block::default());
        frame.render_widget(blank_paragraph, layout[3]);

        // chunk[4]: "Please enter a ZIP code or city name:"
        let prompt_line = Paragraph::new("Please enter a ZIP code or city name:")
            .alignment(Alignment::Left)
            .block(Block::default());
        frame.render_widget(prompt_line, layout[4]);

        // chunk[5]: blank line
        let blank_paragraph = Paragraph::new("").block(Block::default());
        frame.render_widget(blank_paragraph, layout[5]);

        // chunk[6]: final line with ">"
        let arrow_paragraph = Paragraph::new(">")
            .alignment(Alignment::Left)
            .block(Block::default());
        frame.render_widget(arrow_paragraph, layout[6]);
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

////////////////////////////////////////////////////////////////////////////////
// Utility: Fetch weather by ZIP
////////////////////////////////////////////////////////////////////////////////

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
// Utility: Draw TUI-based weather info
////////////////////////////////////////////////////////////////////////////////

fn draw_weather_info(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    weather: &WeatherResponse,
) -> Result<()> {
    terminal.draw(|frame| {
        let size = frame.size();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(100)])
            .split(size);

        // Build up lines of text
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

        let mut lines = Vec::new();
        lines.push(Spans::from(Span::styled(
            heading,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));

        if let Some(desc) = weather.weather.get(0) {
            let cond_str = format!("Condition: {} ({})", desc.main, desc.description);
            lines.push(Spans::from(Span::styled(
                cond_str,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        let temp_str = format!("Temperature: {:.1}°F", weather.main.temp);
        lines.push(Spans::from(Span::styled(
            temp_str,
            Style::default().fg(Color::Blue),
        )));

        if let Some(fl) = weather.main.feels_like {
            lines.push(Spans::from(Span::styled(
                format!("Feels like: {:.1}°F", fl),
                Style::default().fg(Color::Blue),
            )));
        }

        if let Some(min_temp) = weather.main.temp_min {
            lines.push(Spans::from(Span::styled(
                format!("Min temp: {:.1}°F", min_temp),
                Style::default().fg(Color::Blue),
            )));
        }
        if let Some(max_temp) = weather.main.temp_max {
            lines.push(Spans::from(Span::styled(
                format!("Max temp: {:.1}°F", max_temp),
                Style::default().fg(Color::Blue),
            )));
        }

        if let Some(pressure) = weather.main.pressure {
            lines.push(Spans::from(Span::styled(
                format!("Pressure: {} hPa", pressure),
                Style::default().fg(Color::Blue),
            )));
        }

        lines.push(Spans::from(Span::styled(
            format!("Humidity: {}%", weather.main.humidity),
            Style::default().fg(Color::Blue),
        )));

        if let Some(wind) = &weather.wind {
            lines.push(Spans::from(Span::styled(
                format!("Wind speed: {:.1} mph", wind.speed),
                Style::default().fg(Color::Blue),
            )));
            if let Some(gust) = wind.gust {
                lines.push(Spans::from(Span::styled(
                    format!("Wind gust: {:.1} mph", gust),
                    Style::default().fg(Color::Blue),
                )));
            }
            if let Some(deg) = wind.deg {
                lines.push(Spans::from(Span::styled(
                    format!("Wind direction: {}°", deg),
                    Style::default().fg(Color::Blue),
                )));
            }
        }

        if let Some(coord) = &weather.coord {
            lines.push(Spans::from(Span::styled(
                format!("Coordinates: lat {:.2}, lon {:.2}", coord.lat, coord.lon),
                Style::default().fg(Color::Blue),
            )));
        }

        if let Some(sys) = &weather.sys {
            if let Some(sr) = sys.sunrise {
                lines.push(Spans::from(Span::styled(
                    format!("Sunrise (UTC): {}", format_timestamp(sr)),
                    Style::default().fg(Color::Magenta),
                )));
            }
            if let Some(ss) = sys.sunset {
                lines.push(Spans::from(Span::styled(
                    format!("Sunset (UTC): {}", format_timestamp(ss)),
                    Style::default().fg(Color::Magenta),
                )));
            }
        }

        // blank line
        lines.push(Spans::from(""));

        let block = Block::default().borders(Borders::ALL).title("Weather");
        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Left)
            .block(block);
        frame.render_widget(paragraph, layout[0]);
    })?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
// Utility: Format timestamps
////////////////////////////////////////////////////////////////////////////////

fn format_timestamp(timestamp: u64) -> String {
    let timestamp_i64 = timestamp as i64;
    let datetime = Utc
        .timestamp_opt(timestamp_i64, 0)
        .single()
        .unwrap_or_else(|| Utc.timestamp_opt(0, 0).single().unwrap());
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}
