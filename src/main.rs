use chrono::{DateTime, Duration, Local, NaiveDate, NaiveTime, Utc};
use clap::error::Result;
use clap::{Parser, Subcommand, arg, command};
use git2::{Repository, Sort};
// use std::fmt::format;
// use std::fs::ReadDir;
use std::path::{Path, PathBuf};
use std::{fs, process};

#[derive(Parser)]
#[command(version, about, long_about = None, about="Github Analyser, Helper & Cleaner")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(short, long)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "analyze a repository's statistics")]
    Stats {
        #[arg(short, long, help = "/path/to/repo")]
        path: PathBuf,
        #[arg(short, long, value_parser = clap::value_parser!(NaiveDate), help = "Date 'Since' you want to fetch statistics from", )]
        since: Option<NaiveDate>,
        #[arg(short, long, help = "The type you are want it in: 1 - Terminal")]
        format: Option<u8>,
    },
    #[command(about = "find all git repositories in a directory")]
    Find {
        #[arg(short, long, help = "/path/to/find-in")]
        path: PathBuf,
        #[arg(short, long, help = "Files you to exclude")]
        exclude: Option<Vec<String>>,
    },
    #[command(about = "clean up a repository")]
    Clean {
        #[arg(short, long, help = "/path/to/clean")]
        path: PathBuf,
    },
}

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
#[derive(Debug, Clone, Copy, Default)]
struct SinceCommitStat {
    since_time: NaiveDate,
    total_commits_since: u32,
}

impl SinceCommitStat {
    pub fn new(since_time: NaiveDate) -> Self {
        Self {
            since_time,
            total_commits_since: 0,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(some_comands) => match some_comands {
            Commands::Stats {
                path,
                since,
                format,
            } => {
                handle_stats(path, since, format);
            }
            Commands::Find { path, exclude } => {
                handle_find(path, exclude);
            }
            Commands::Clean { path } => {
                println!("'myapp add' was used, path is: {path:?}");
            }
        },
        None => todo!(),
    }
}

fn handle_stats(path: &PathBuf, since_date: &Option<NaiveDate>, format: &Option<u8>) {
    let now = Utc::now();
    let _since_date = since_date;
    let _format = format;

    let mut since_commit_stat = SinceCommitStat::new(_since_date.unwrap());
    let mut commit_stat = CommitStats::new();

    let month_ago = now - Duration::weeks(4);
    let week_ago = now - Duration::weeks(1);
    let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
    let repo: Repository = Repository::open(&path).unwrap_or_else(|err| {
        eprintln!("an error occured: {} ", err);
        process::exit(1);
    });
    let mut revwalk = repo.revwalk().unwrap_or_else(|err| {
        eprintln!("an error occured: {} ", err);
        process::exit(1);
    });
    revwalk.set_sorting(Sort::TIME).unwrap_or_else(|err| {
        eprintln!("an error occured: {} ", err);
        process::exit(1);
    });
    revwalk.push_head().unwrap_or_else(|err| {
        eprintln!("an error occured: {} ", err);
        process::exit(1);
    });
    let all_oids = revwalk.into_iter().collect::<Result<Vec<_>, _>>().unwrap();

    match _since_date {
        Some(date) => {
            for oid in &all_oids {
                let commit = repo.find_commit(*oid).unwrap();
                let commit_time: git2::Time = commit.time();

                let converted_time_of_commit: DateTime<chrono::Utc> =
                    DateTime::from_timestamp(commit_time.seconds(), 0).expect("Invalid Time");
                // NaiveDate
                let converted_date_time = to_datetime(_since_date.unwrap()).unwrap();
                if converted_time_of_commit >= converted_date_time {
                    since_commit_stat.total_commits_since += 1;
                }
            }
        }
        None => {

            commit_stat.total_commits = all_oids.len() as u32;

            if let Some(last_oid) = all_oids.first() {
                let last_commit = repo.find_commit(*last_oid).expect("Last commit not found");
                let last_commit_time = last_commit.time();
                let last_commit_date_time_format =
                    DateTime::from_timestamp(last_commit_time.seconds(), 0).expect("invalid time");
                commit_stat.last_commit_time = last_commit_date_time_format;
            }

            for oid in &all_oids {
                let commit = repo.find_commit(*oid).unwrap();
                let commit_time = commit.time();

                let converted_time_of_commit: DateTime<chrono::Utc> =
                    DateTime::from_timestamp(commit_time.seconds(), 0).expect("Invalid Time");

                if converted_time_of_commit >= month_ago {
                    commit_stat.commits_this_month += 1;
                }

                if converted_time_of_commit >= week_ago {
                    commit_stat.commits_this_week += 1;
                }

                if converted_time_of_commit > today_start {
                    commit_stat.commits_today += 1;
                }

                if converted_time_of_commit < month_ago {
                    break;
                }
            }
        }
    }

    println!("all_commits_for_the_time {:?}", since_commit_stat);
    let mut commit_stats = format!( "
    Repository Statistics for: {:?}
    ==============================================

    📊 COMMIT STATISTICS
    Total commits: {}
    Commits this month: {}
    Commits this week: {}
    Commits today: {}
    Last commit time: {}

    ", path,
        commit_stat.total_commits,
        commit_stat.commits_this_month,
        commit_stat.commits_this_week,
        commit_stat.commits_today,
        commit_stat.last_commit_time,
        );

    if convert_time_to_days_ago(commit_stat.last_commit_time) > 0 {
        commit_stats.push_str(&format!(", {} days ago", convert_time_to_days_ago(commit_stat.last_commit_time)));
    }

    println!("{commit_stats}");
}

fn handle_find(path: &PathBuf, exclude: &Option<Vec<String>>) {
    let read_dir = fs::read_dir(path).unwrap();

    for dir in read_dir {
        let entry = dir.unwrap();
        let entry_path = entry.path();
        let path_str = entry_path.to_str().unwrap().to_owned();
        let folder_name = entry.file_name().into_string().unwrap();
        if entry_path.is_dir() && Path::new(&format!("{}/.git", path_str)).exists() {
            let is_excluded = exclude
                .as_ref()
                .map_or(false, |ex| ex.contains(&folder_name));
            if !is_excluded {
                println!("Directory with git found {:?}", entry_path);
            }
        }
    }
}

fn convert_time_to_days_ago(time: DateTime<Utc>) -> i64 {
    let now = Utc::now();
    let diffrence = now - time;
    diffrence.num_days()
}

// fn main() {
//     let paths = fs::read_dir("/Users/macbookpro/Documents/projects").unwrap();

//     for path in paths {
//         println!("Name: {}", path.unwrap().path().display())
//     }
// }

const MIDNIGHT: NaiveTime = NaiveTime::from_hms(0, 0, 0);

fn to_datetime(date: NaiveDate) -> Option<DateTime<Local>> {
    date.and_time(MIDNIGHT).and_local_timezone(Local).earliest()
}
