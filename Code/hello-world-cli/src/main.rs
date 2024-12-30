use anyhow::{Context, Result};
use colored::{Color, Colorize};
use rand::seq::SliceRandom;
use rand::Rng;
use std::io::{self, Write};

fn main() -> Result<()> {
    clear_screen();
    print_welcome_banner();
    prompt_and_greet()?;
    Ok(())
}

/// Clears the terminal screen for a clean start.
fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
}

/// Prints a banner with ASCII art in a random color (similar to your weather CLI).
fn print_welcome_banner() {
    let banner = r#"
 _            _  _                                 _      _
| |          | || |                               | |    | |
| |__    ___ | || |  ___   __      __  ___   _ __ | |  __| |
| '_ \  / _ \| || | / _ \  \ \ /\ / / / _ \ | '__|| | / _` |
| | | ||  __/| || || (_) |  \ V  V / | (_) || |   | || (_| |
|_| |_| \___||_||_| \___/    \_/\_/   \___/ |_|   |_| \__,_|
    "#;

    // Print the banner in a random color
    cprintln(banner);
}

/// Prompts the user for their name and greets them in a random language/color.
fn prompt_and_greet() -> Result<()> {
    cprintln("Welcome to the Interactive, Multi-Lingual Greeter!\n");

    print!("What is your name? ");
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut name = String::new();
    io::stdin()
        .read_line(&mut name)
        .context("Failed to read input from stdin")?;

    let trimmed = name.trim();
    if trimmed.is_empty() {
        greet("World");
    } else {
        greet(trimmed);
    }

    Ok(())
}

/// Selects a random greeting from a list of world languages and prints it in a random color.
fn greet(name: &str) {
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

    let mut rng = rand::thread_rng();
    if let Some(greeting) = greetings.choose(&mut rng) {
        let message = format!("{} — {}!", greeting, name);
        cprintln(&message);
    } else {
        // fallback (shouldn't happen if greetings is non-empty)
        cprintln(&format!("Hello, {}!", name));
    }
}

/// Prints the given text in a random color.
fn cprintln(text: &str) {
    println!("{}", text.color(random_color()).bold());
}

/// Returns a random color (standard 8 + bright 8).
fn random_color() -> Color {
    let colors = [
        // Standard
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::White,
        // Bright
        Color::BrightRed,
        Color::BrightGreen,
        Color::BrightYellow,
        Color::BrightBlue,
        Color::BrightMagenta,
        Color::BrightCyan,
        Color::BrightWhite,
    ];

    let idx = rand::thread_rng().gen_range(0..colors.len());
    colors[idx]
}
