use colored::Colorize;
use rand::seq::SliceRandom;
use std::io::{self, Write};

fn main() {
    // Print a welcome message
    println!("{}", "Welcome to the Interactive Greeter!".cyan().bold());

    // Prompt user for their name
    print!("What is your name? ");
    // Make sure everything written so far is displayed before reading input
    io::stdout().flush().expect("Failed to flush stdout");

    let mut name = String::new();
    match io::stdin().read_line(&mut name) {
        Ok(_) => {
            let trimmed = name.trim();
            // If no name was provided, default to "World"
            if trimmed.is_empty() {
                greet("World");
            } else {
                greet(trimmed);
            }
        }
        Err(e) => eprintln!("Failed to read input from stdin: {e}"),
    }
}

/// Randomly selects a greeting from a list of world languages and prints it in color.
fn greet(name: &str) {
    // A list of “Hello” in a variety of world languages.
    // You can add, remove, or modify entries as desired.
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
        println!("{}", format!("{} — {}!", greeting, name).green().bold());
    } else {
        // Fallback (should never happen if `greetings` is non-empty)
        println!("Hello, {}!", name);
    }
}
