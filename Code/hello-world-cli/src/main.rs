use clap::{Arg, Command};

fn main() {
    // Define the CLI interface
    let matches = Command::new("hello-world-cli")
        .version("1.0")
        .author("o1 pro")
        .about("A clean and beautiful Hello World CLI in Rust")
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .value_name("NAME")
                .help("Specify the name to greet")
                .default_value("World")
        )
        .get_matches();

    // Retrieve the "name" argument
    let name = matches.get_one::<String>("name").unwrap();

    println!("Hello, {}!", name);
}
