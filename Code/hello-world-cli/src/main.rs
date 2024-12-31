use anyhow::{Context, Result};
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{style, Color, Stylize},
    terminal::{Clear, ClearType},
};
use rand::{seq::SliceRandom, Rng};
use std::io::{self, Write};
use tokio::main;

/// Asynchronous entry point using Tokio.
#[main]
async fn main() -> Result<()> {
    clear_screen()?;
    print_welcome_banner()?;
    prompt_for_name_and_greet()?;
    Ok(())
}

/// Clears the terminal screen for a clean start using crossterm.
fn clear_screen() -> Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    Ok(())
}

/// Prints a welcome banner with ASCII art in a random color.
/// Also provides a short usage hint.
fn print_welcome_banner() -> Result<()> {
    let banner = r#"
 _            _  _                                 _      _
| |          | || |                               | |    | |
| |__    ___ | || |  ___   __      __  ___   _ __ | |  __| |
| '_ \  / _ \| || | / _ \  \ \ /\ / / / _ \ | '__|| | / _` |
| | | ||  __/| || || (_) |  \ V  V / | (_) || |   | || (_| |
|_| |_| \___||_||_| \___/    \_/\_/   \___/ |_|   |_| \__,_|
"#;

    cprintln(banner)?;
    Ok(())
}

/// Prompts the user for their name and greets them in a random language/color.
///
/// If the user provides no input, it defaults to greeting "World".
fn prompt_for_name_and_greet() -> Result<()> {
    cprintln("Welcome to the Interactive, Multilingual Greeter!\r\n")?;

    // Prompt user for their name
    print!("What is your name?\r\n");
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .context("Failed to read from stdin")?;

    let trimmed = name.trim();
    if trimmed.is_empty() {
        greet("World")?;
    } else {
        greet(trimmed)?;
    }
    Ok(())
}

/// Selects a random greeting from a list of world languages and prints it in a random color.
fn greet(name: &str) -> Result<()> {
    let greetings = [
        "Mandarin Chinese: 你好 (Nǐ hǎo)",
        "Spanish: Hola",
        "English: Hello",
        "Hindi: नमस्ते (Namaste)",
        "Arabic (Modern Standard): مرحبا (Marḥaban)",
        "Bengali: নমস্কার (Nomoshkar)",
        "Portuguese: Olá",
        "Russian: Привет (Privet)",
        "Japanese: こんにちは (Konnichiwa)",
        "Punjabi: ਸਤ ਸ੍ਰੀ ਅਕਾਲ (Sat Srī Akāl)",
        "German: Hallo",
        "Javanese: Halo",
        "Wu Chinese (Shanghainese): 侬好 (Nóng hó)",
        "Malay (Bahasa Melayu): Hai",
        "Telugu: నమస్కారం (Namaskāraṁ)",
        "Vietnamese: Xin chào",
        "Korean: 안녕하세요 (Annyeonghaseyo)",
        "French: Bonjour",
        "Tamil: வணக்கம் (Vaṇakkam)",
        "Marathi: नमस्कार (Namaskār)",
        "Urdu: اسلام علیکم (As-salāmu ʿalaykum)",
        "Turkish: Merhaba",
        "Italian: Ciao",
        "Thai: สวัสดี (S̄wạs̄dī)",
        "Gujarati: નમસ્તે (Namaste)",
        "Persian (Farsi): سلام (Salām)",
        "Polish: Cześć",
        "Pashto: السلام علیکم (As-salāmu ʿalaykum)",
        "Kannada: ನಮಸ್ಕಾರ (Namaskāra)",
        "Ukrainian: Привіт (Pryvit)",
        "Swahili: Jambo",
        "Zulu: Sawubona",
        "Greek: Γεια σου (Geia sou)",
        "Dutch: Hallo",
        "Haitian Creole: Bonjou",
        "Tagalog: Kamusta",
        "Hungarian: Szia",
        "Czech: Ahoj",
        "Romanian: Bună",
        "Bulgarian: Здравей (Zdravey)",
        "Catalan: Hola",
        "Finnish: Hei",
        "Norwegian: Hei",
        "Swedish: Hej",
        "Danish: Hej",
        "Slovak: Ahoj",
        "Malayalam: നമസ്കാരം (Namaskāram)",
        "Burmese: မင်္ဂလာပါ (Mingalaba)",
        "Georgian: გამარჯობა (Gamarjoba)",
        "Bosnian: Zdravo",
        "Croatian: Bok",
        "Serbian: Zdravo",
        "Slovene: Živijo",
        "Indonesian: Halo",
        "Afrikaans: Hallo",
    ];

    // Choose a random greeting
    let mut rng = rand::thread_rng();
    let greeting = greetings
        .choose(&mut rng)
        .unwrap_or(&"English: Hello (Fallback)");

    let message = format!("{} — {}!", greeting, name);
    cprintln(&message)?;
    Ok(())
}

/// Prints the given text in a random color using crossterm styling.
///
/// This function appends a carriage-return + line-feed (`\r\n`) at the end of `text`.
fn cprintln(text: &str) -> Result<()> {
    let color = random_color();
    let styled = style(text).with(color).bold();
    print!("{}\r\n", styled);
    Ok(())
}

/// Returns a random color from a standard 8 color list
fn random_color() -> Color {
    let colors = [
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::White,
        Color::Grey,
    ];

    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..colors.len());
    colors[idx]
}
