use colored::*;
use rand::Rng;
use std::io::{self, Write};

fn main() {
    print_welcome_banner();
    prompt_and_greet();
}

/// Prints a banner with ASCII art in a random color.
fn print_welcome_banner() {
    let banner = r#"
                                                   _
                                                  | |
 _   _  ___   ___  _ __    __ _  _ __   ___   ___ | |_   ___  _ __
| | | |/ __| / _ \| '__|  / _` || '__| / _ \ / _ \| __| / _ \| '__|
| |_| |\__ \|  __/| |    | (_| || |   |  __/|  __/| |_ |  __/| |
 \__,_||___/ \___||_|     \__, ||_|    \___| \___| \__| \___||_|
                           __/ |
                          |___/
    "#;

    cprintln(banner);
    cprintln("Welcome to the colorful user-greeter!!\n");
}

/// Prompts the user for their name and greets them with a random-color greeting.
fn prompt_and_greet() {
    cprint("What's your name? ");

    let mut name = String::new();
    match io::stdin().read_line(&mut name) {
        Ok(_) => {
            let trimmed = name.trim();
            cprintln(""); // blank line after input
            if trimmed.is_empty() {
                cprintln("Hello, World! (No name provided.)");
            } else {
                greet(trimmed);
            }
        }
        Err(e) => cprintln(format!("Failed to read input from stdin: {e}")),
    }

    cprintln(""); // final blank line
}

/// Greets the user in a random color.
fn greet(name: &str) {
    cprintln(format!("Hello, {name}!"));
}

/// Prints the given text in a random color (with a trailing newline).
fn cprintln<T: AsRef<str>>(msg: T) {
    let color = random_color();
    println!("{}", msg.as_ref().color(color).bold());
}

/// Prints the given text in a random color (no newline), then flushes stdout.
fn cprint<T: AsRef<str>>(msg: T) {
    let color = random_color();
    print!("{}", msg.as_ref().color(color).bold());
    io::stdout().flush().expect("Failed to flush stdout");
}

/// Returns a random color from the `colored` crate (standard 8 + bright 8).
fn random_color() -> Color {
    let colors = [
        // Standard colors
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::White,
        // Bright variants
        Color::BrightRed,
        Color::BrightGreen,
        Color::BrightYellow,
        Color::BrightBlue,
        Color::BrightMagenta,
        Color::BrightCyan,
        Color::BrightWhite,
    ];

    // Note: Black or BrightBlack may be invisible if your terminal background is black!
    let idx = rand::thread_rng().gen_range(0..colors.len());
    colors[idx]
}
