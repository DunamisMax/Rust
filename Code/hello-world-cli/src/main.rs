use clap::{Arg, ArgAction, Command};
use colored::*; // for colorful text
use std::io::{self, Write};

/// The main entry point for our CLI.
fn main() {
    // Build the CLI using clap
    let matches = Command::new("cool-greeter")
        .version("1.1")
        .author("o1 pro")
        .about("An enhanced and colorful Hello World CLI in Rust!")
        .subcommand_required(false)
        .arg_required_else_help(false)
        // Top-level arguments:
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .value_name("NAME")
                .help("Specify the name to greet")
                .default_value("World"),
        )
        .arg(
            Arg::new("language")
                .short('l')
                .long("language")
                .help("Choose the language of the greeting")
                .default_value("English")
                .value_parser(["English", "Spanish", "Pirate"]),
        )
        .arg(
            Arg::new("excited")
                .short('e')
                .long("excited")
                .help("Add more exclamation points to the greeting")
                .action(ArgAction::Count),
        )
        // Subcommands:
        .subcommand(
            Command::new("interactive")
                .about("Enter interactive mode to input the name at runtime"),
        )
        .get_matches();

    // Handle subcommands first
    if let Some(("interactive", _sub_m)) = matches.subcommand() {
        run_interactive_mode();
        return;
    }

    // Retrieve top-level arguments
    let name = matches.get_one::<String>("name").unwrap();
    let language = matches.get_one::<String>("language").unwrap();
    let excitement_level = matches.get_count("excited");

    // Build the greeting
    let greeting = build_greeting(name, language, excitement_level);

    // Print with color
    println!("{}", greeting.green().bold());
}

/// Constructs the greeting string based on the user inputs.
fn build_greeting(name: &str, language: &str, excitement_level: u8) -> String {
    // Determine the base greeting
    let base = match language {
        "English" => format!("Hello, {}", name),
        "Spanish" => format!("¡Hola, {}!", name),
        "Pirate" => format!("Ahoy, {}!", name),
        _ => unreachable!(), // we've covered all possible language choices above
    };

    // Add exclamation points
    // (already includes an exclamation in Spanish/Pirate, but let's compound it)
    let mut greeting = base;
    for _ in 0..excitement_level {
        greeting.push('!');
    }

    greeting
}

/// Enters an interactive mode where the user can input a name at runtime.
fn run_interactive_mode() {
    print!("Please enter a name to greet: ");
    // Make sure to flush stdout so the prompt shows
    io::stdout().flush().unwrap();

    let mut buffer = String::new();
    if io::stdin().read_line(&mut buffer).is_ok() {
        let trimmed = buffer.trim();
        if trimmed.is_empty() {
            println!(
                "{}",
                "No name provided, so I'll greet the whole world!".yellow()
            );
            println!("{}", "Hello, World!".green().bold());
        } else {
            println!("{}", format!("Hello, {}!", trimmed).green().bold());
        }
    } else {
        eprintln!("Failed to read input from stdin.");
    }
}
