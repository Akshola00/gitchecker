use chrono::{DateTime, Duration, NaiveDate, Utc};
use clap::error::Result;
use clap::{Arg, ArgAction, Command, arg, command, value_parser};
use git2::{Repository, Sort, oid_array};
use std::path::PathBuf;
use std::process;

#[derive(Debug, Clone, Copy, Default)]
struct CommitStats {
    total_commits: u32,
    commits_this_month: u32,
    commits_this_week: u32,
    commits_today: u32,
    last_commit_time: DateTime<Utc>,
}

impl CommitStats {
    pub fn new() -> Self {
        CommitStats {
            total_commits: 0,
            commits_this_month: 0,
            commits_this_week: 0,
            commits_today: 0,
            last_commit_time: DateTime::from_timestamp(0, 0).unwrap(),
        }
    }
}
fn main() {
    let matches = command!()
        .subcommand_required(true)
        .subcommand(
            Command::new("stats")
                .about("Analyze a repository's statistics")
                .arg(
                    arg!([path] "/path/to/repo")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    arg!(-s --since <DATE> "Date you want to fetch statistics from")
                        .value_parser(value_parser!(NaiveDate)),
                )
                .arg(arg!(-f --format <FORMAT_TYPE> "The type you are passing it in")),
        )
        .subcommand(
            Command::new("find")
                .about("Find all git repositories in a directory")
                .arg(
                    arg!([path] "path to project")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                )
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

    let dry_run = matches.get_flag("dry_run");

    match matches.subcommand() {
        Some(("stats", stats)) => {
            let path = stats
                .get_one::<PathBuf>("path")
                .expect("unable to parse path");
            let since_date = stats.get_one::<NaiveDate>("since");
            let format = stats.get_one::<String>("format");
            handle_stats(path, since_date, format, dry_run);
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

fn handle_stats(
    path: &PathBuf,
    since_date: Option<&NaiveDate>,
    format: Option<&String>,
    dry_run: bool,
) {
    // initialze a new commit stat struct
    let mut commit_stat = CommitStats::new();
    // get the current time
    let now = Utc::now();
    // month ago
    let month_ago = now - Duration::weeks(4);
    // going to a week ago
    let week_ago = now - Duration::weeks(1);
    // today get the time of when today started
    let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
    // use the git 2 library to open a github repo path perform error handling
    let repo: Repository = Repository::open(&path).unwrap_or_else(|err| {
        eprintln!("an error occured: {} ", err);
        process::exit(1);
    });
    // get a revwalk of the repo
    let mut revwalk = repo.revwalk().unwrap_or_else(|err| {
        eprintln!("an error occured: {} ", err);
        process::exit(1);
    });
    // sort the commits in the rev walk
    revwalk.set_sorting(Sort::TIME).unwrap_or_else(|err| {
        eprintln!("an error occured: {} ", err);
        process::exit(1);
    });
    revwalk.push_head().unwrap_or_else(|err| {
        eprintln!("an error occured: {} ", err);
        process::exit(1);
    });
    // for borrowing and mutablity issues - collect all the oids and put in a vec
    let all_oids = revwalk.into_iter().collect::<Result<Vec<_>, _>>().unwrap();
    // get total commits and store
    commit_stat.total_commits = all_oids.len() as u32;

    // logic to get last commit date
    if let Some(last_oid) = all_oids.last() {
        let last_commit = repo.find_commit(*last_oid).expect("Last commit not found");
        let last_commit_time = last_commit.time();
        let last_commit_date_time_format =
            DateTime::from_timestamp(last_commit_time.seconds(), 0).expect("invalid time");
        commit_stat.last_commit_time = last_commit_date_time_format;
    }

    // logic to get commits and check weatehr they are in the time frame we want
    for oid in &all_oids {
        let commit = repo.find_commit(*oid).unwrap();
        let commit_time = commit.time();

        // convert the commit time to date time
        let converted_time_of_commit: DateTime<chrono::Utc> =
            DateTime::from_timestamp(commit_time.seconds(), 0).expect("Invalid Time");

        // if thisd commit was this month
        if converted_time_of_commit >= month_ago {
            commit_stat.commits_this_month += 1;
        }

        // if the commit was this week
        if converted_time_of_commit >= week_ago {
            commit_stat.commits_this_week += 1;
        }

        // if the commit was today
        if converted_time_of_commit > today_start {
            commit_stat.commits_today += 1;
        }

        if converted_time_of_commit < month_ago {
            break;
        }
    }
    println!(
        "
    Repository Statistics for: {:?} 
    ==============================================

    📊 COMMIT STATISTICS
    Total commits: {}
    Commits this month: {}
    Commits this week: {}
    Commits today: {}
    Last commit time: {} (3 days ago)

    ",
        path,
        commit_stat.total_commits,
        commit_stat.commits_this_month,
        commit_stat.commits_this_week,
        commit_stat.commits_today,
        commit_stat.last_commit_time
    )
}

fn convert_time_to_days_ago(time: DateTime<Utc>) -> u32 {
    // ymd 2025-01-23 - 2025-01-25
    let now = Utc::now();
    // convert time from date time to epoch
    let time_epoch = time.timestamp();
    // convert today to epoch 
    let today_start: DateTime<Utc> = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
    let today_epoch = today_start.timestamp();
    // minus today - that time 
    let diffrence = today_epoch - time_epoch;
    // convert the time to date time
    let converted_time = DateTime::from_timestamp(diffrence, 0).expect("invalid");
    // get the day 
    0
}

// # Analyze a repository's statistics
// cargo run -- stats /path/to/repo --since "2024-01-01" --format json
