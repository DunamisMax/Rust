use colored::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
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

/// Returns a randomly selected color from a predefined list.
fn random_color() -> Color {
    let colors = [
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::White,
    ];
    let mut rng = thread_rng();
    *colors.choose(&mut rng).unwrap_or(&Color::White)
}
