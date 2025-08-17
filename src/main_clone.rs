use std::path::PathBuf;

use clap::{arg, command, value_parser, Arg, ArgAction, Command};

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(arg!([name] "Optional name to operate on"))
        .arg(
            arg!(
                -c --config <FILE> "Sets a custom config file"
            )
            // We don't have syntax yet for optional options, so manually calling `required`
            .required(false)
            .value_parser(value_parser!(PathBuf)),
        )
        .arg(arg!(
            -d --debug ... "Turn debugging information on"
        ))
        .subcommand(
            Command::new("test")
                .about("does testing things")
                .arg(arg!(-l --list "lists test values").action(ArgAction::SetTrue)),
        )
        .get_matches();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = matches.get_one::<String>("name") {
        println!("Value for name: {name}");
    }

    if let Some(config_path) = matches.get_one::<PathBuf>("config") {
        println!("Value for config: {}", config_path.display());
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match matches
        .get_one::<u8>("debug")
        .expect("Counts are defaulted")
    {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        3 => println!("China will be great again"),
        _ => println!("Don't be crazy"),
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    if let Some(matches) = matches.subcommand_matches("test") {
        // "$ myapp test" was run
        if matches.get_flag("list") {
            // "$ myapp test -l" was run
            println!("Printing testing lists...");
        } else {
            println!("Not printing testing lists...");
        }
    }

    // Continued program logic goes here...
}

// git-analyzer stats <path> [OPTIONS]
// git-analyzer find <search-path> [OPTIONS] 
// git-analyzer clean <path> [OPTIONS]

fn check() {
let app_m = Command::new("myprog")
    .arg(Arg::new("debug")
        .short('d')
        .action(ArgAction::SetTrue)
    )
    .subcommand(Command::new("test")
        .arg(Arg::new("opt")
            .long("option")
            .action(ArgAction::Set)))
    .get_matches_from(vec![
        "myprog", "-d", "test", "--option", "val"
    ]);

// Both parent commands, and child subcommands can have arguments present at the same times
assert!(app_m.get_flag("debug"));

// Get the subcommand's ArgMatches instance
if let Some(sub_m) = app_m.subcommand_matches("test") {
    // Use the struct like normal
    assert_eq!(sub_m.get_one::<String>("opt").map(|s| s.as_str()), Some("val"));
}}