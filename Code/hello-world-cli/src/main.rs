use clap::{value_parser, Arg, ArgAction, Command};
use colored::*;
use std::io::{self, Write};
use std::str::FromStr;

/// Represents the supported greeting languages.
/// We derive `Clone` and `Debug` for convenience in usage and testing.
#[derive(Clone, Debug)]
enum Language {
    English,
    Spanish,
    Pirate,
}

/// Custom parsing so Clap can validate the language.
impl FromStr for Language {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "english" => Ok(Language::English),
            "spanish" => Ok(Language::Spanish),
            "pirate" => Ok(Language::Pirate),
            other => Err(format!("Unsupported language: {other}")),
        }
    }
}

/// The main entry point for the CLI.
fn main() {
    let matches = build_cli().get_matches();

    // Handle subcommands first
    match matches.subcommand() {
        Some(("interactive", _sub_matches)) => {
            run_interactive_mode();
        }
        // No subcommand used, just parse top-level args
        _ => {
            let name = matches
                .get_one::<String>("name")
                .expect("Name argument not found, even though it has a default.");

            let language = matches
                .get_one::<Language>("language")
                .expect("Language argument not found, even though it has a default.");

            let excitement = *matches
                .get_one::<u8>("excitement")
                .expect("Excitement argument not found, even though it has a default.");

            // Construct the greeting
            let greeting = build_greeting(name, language, excitement);
            println!("{}", greeting.green().bold());
        }
    }
}

/// Builds the CLI definition using Clap.
fn build_cli() -> Command {
    Command::new("cool-greeter")
        .version("1.2")
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
                .help("Specify the name to greet.")
                .default_value("World"),
        )
        .arg(
            Arg::new("language")
                .short('l')
                .long("language")
                .help("Choose the language of the greeting.")
                // Value parser uses our custom FromStr for `Language`
                .value_parser(value_parser!(Language))
                .default_value("English"),
        )
        .arg(
            Arg::new("excitement")
                .short('e')
                .long("excitement")
                .help("Number of extra exclamation marks to add.")
                .default_value("0") // default extra excitement is 0
                .value_parser(value_parser!(u8))
                .action(ArgAction::Set),
        )
        // Subcommands:
        .subcommand(
            Command::new("interactive")
                .about("Enter interactive mode to input the name at runtime."),
        )
}

/// Constructs the greeting string based on the user inputs.
fn build_greeting(name: &str, language: &Language, excitement_level: u8) -> String {
    // Determine the base greeting
    let base = match language {
        Language::English => format!("Hello, {}", name),
        Language::Spanish => format!("¡Hola, {}!", name),
        Language::Pirate => format!("Ahoy, {}!!", name),
    };

    // Add exclamation points
    // (Note that Spanish/Pirate already end with an exclamation mark—this just adds more)
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
    match io::stdin().read_line(&mut buffer) {
        Ok(_) => {
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
        }
        Err(e) => eprintln!("Failed to read input from stdin: {e}"),
    }
}

// Below are optional tests showing how you might verify that `build_greeting` works as expected.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_greeting_english_no_excitement() {
        let result = build_greeting("Alice", &Language::English, 0);
        assert_eq!(result, "Hello, Alice");
    }

    #[test]
    fn test_build_greeting_spanish_extra_excitement() {
        let result = build_greeting("Bob", &Language::Spanish, 3);
        assert_eq!(result, "¡Hola, Bob!!!!");
    }

    #[test]
    fn test_build_greeting_pirate_excitement() {
        let result = build_greeting("Matey", &Language::Pirate, 2);
        assert_eq!(result, "Ahoy, Matey!!!!");
    }
}
