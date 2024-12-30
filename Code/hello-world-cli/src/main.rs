use colored::{Color, Colorize};
use figlet_rs::FIGfont; // <-- Make sure to add this import
use rand::seq::SliceRandom;
use std::io::{self, Write};

fn main() {
    clear_screen();
    print_colored_banner();
    prompt_and_greet();
}

/// Clears the terminal screen for a clean start.
fn clear_screen() {
    // This escape sequence should work on most Unix-like systems.
    // On Windows, "cls" might be used, but this often works as well.
    print!("\x1B[2J\x1B[1;1H");
}

/// Displays a random-color ASCII banner for "DunamisMax" using the figlet-rs library.
fn print_colored_banner() {
    // Load the standard FIGfont (bundled with figlet-rs).
    // You can also try other built-in fonts like FIGfont::shadow(), FIGfont::big(), etc.
    let standard_font = FIGfont::standard().expect("Failed to load standard FIGfont");

    // Convert "DunamisMax" into ASCII art
    let figure = standard_font
        .convert("DunamisMax")
        .expect("Failed to convert text with FIGfont");

    // Created with o1-pro tagline
    let tagline = "~ Created with o1-pro ~";

    // Pick a random color to make things vibrant
    let color = random_color();

    // Print the ASCII art in a random color, then the tagline
    println!("{}", figure.to_string().color(color).bold());
    println!("{}", tagline.color(color).bold());
    println!(); // This adds the blank line
}

/// Prompts the user for their name and greets them with a random-color greeting.
fn prompt_and_greet() {
    // Prompt user for their name
    println!(
        "{}",
        "Welcome to the Interactive, Multi-Lingual Greeter!"
            .cyan()
            .bold()
    );
    println!(); // This adds the blank line
    print!("What is your name? ");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut name = String::new();
    match io::stdin().read_line(&mut name) {
        Ok(_) => {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                greet("World");
            } else {
                greet(trimmed);
            }
        }
        Err(e) => eprintln!("Failed to read input from stdin: {e}"),
    }
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
        // Choose a random color for the greeting text
        let color = random_color();
        let message = format!("{} — {}!", greeting, name)
            .color(color)
            .bold()
            .to_string();

        // Print the final greeting
        println!("{}", message);
    } else {
        // Fallback (should never happen if `greetings` is non-empty)
        println!("Hello, {}!", name);
    }
}

/// Picks a random color from a palette of bright, eye-catching choices.
fn random_color() -> Color {
    let palette = [
        Color::Red,
        Color::Green,
        Color::Blue,
        Color::Yellow,
        Color::Magenta,
        Color::Cyan,
        Color::White,
    ];
    let mut rng = rand::thread_rng();
    *palette.choose(&mut rng).unwrap_or(&Color::White)
}
