// use std::path::PathBuf;

use std::process;

use clap::{Arg, ArgAction, Command, arg, command};

fn main() {
    let matches = command!()
        .subcommand_required(true)
        .subcommand(
            Command::new("stats")
                .about("Analyze a repository's statistics")
                .arg(arg!([path] "/path/to/repo").required(true))
                .arg(arg!(-s --since <DATE> "Date you want to fetch statistics from"))
                .arg(arg!(-f --format <FORMAT_TYPE> "The type you are passing it in")),
        )
        .subcommand(
            Command::new("find")
                .about("Find all git repositories in a directory")
                .arg(arg!([path] "path to project").required(true))
                .arg(arg!(-e --exclude <FILES_TO_EXCLUDE> "Files you to exclude")),
        )
        .subcommand(
            Command::new("clean")
                .about("Clean up a repository")
                .arg(arg!([path] "path to repo").required(true)),
        )
        .arg(
            Arg::new("dry_run")
                .short('d')
                .long("dry_run")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("stats", stats)) => {
            println!(
                "
                User wants to check the statistic of a github repo
                Here are the details: \n
                path: {:?}, \n
                since: {:?}, \n
                format: {:?} \n
                ",
                stats.get_one::<String>("path"),
                stats.get_one::<String>("since"),
                stats.get_one::<String>("format"),
            )
        }
        Some(("find", find)) => {
            println!(
                "
                User wants to find all gi repos in a directory
                Here are the details: \n
                path: {:?}, \n
                files to exclude: {:?},
                ",
                find.get_one::<String>("path"),
                find.get_one::<String>("exclude"),
            )
        }
        Some(("clean", clean)) => {
            println!(
                "
                User wants to clean a repo
                Here are the details: \n
                path: {:?}, \n

                ",
                clean.get_one::<String>("path"),
            )
        }
        _ => {
            println!("not recognized");
            process::exit(0);
        }
    }
}

// # Analyze a repository's statistics
// cargo run -- stats /path/to/repo --since "2024-01-01" --format json

// # Find all git repositories in a directory
// cargo run -- find ~/projects --exclude node_modules,target

// # Clean up a repository
// cargo run -- clean /path/to/repo --dry-run --verbose

// # Global flags work with any subcommand
// cargo run -- stats . --verbose --dry-run
