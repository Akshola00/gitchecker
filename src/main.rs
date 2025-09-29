use chrono::{DateTime, Duration, Local, NaiveDate, NaiveTime, Utc};
use clap::error::Result;
use clap::{Parser, Subcommand, arg, command};
use git2::{Repository, Sort, Status, StatusOptions, opts};
use serde::{Deserialize, Serialize};
use std::fs::File;

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
        #[arg(short, long, help = "[1 - simple print, 2 - json file]")]
        format: Option<u8>,
    },
    #[command(about = "find all git repositories in a directory")]
    Find {
        #[arg(short, long, help = "/path/to/find-in")]
        path: PathBuf,
        #[arg(short, long, help = "[1 - simple print, 2 - json file]")]
        format: Option<u8>,
        #[arg(short, long, help = "Files you to exclude")]
        exclude: Option<Vec<String>>,
    },
    #[command(about = "clean up a repository")]
    Clean {
        #[arg(short, long, help = "/path/to/clean")]
        path: PathBuf,
        #[arg(short, long, help = "clean untracked files")]
        untracked: Option<bool>,
        #[arg(short, long, help = "clean ignored files")]
        ignored: Option<bool>,
        #[arg(short, long, help = "clean merged branches")]
        branches: Option<bool>,
        #[arg(short, long, help = "clean merged branches")]
        stash: Option<bool>,
        #[arg(
            short = 'S',
            long = "ignore_submodules",
            help = "ignore submodule files"
        )]
        ignore_submodules: Option<bool>,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CommitStats {
    repo_name: String,
    total_commits: u32,
    commits_this_month: u32,
    commits_this_week: u32,
    commits_today: u32,
    last_commit_time: DateTime<Utc>,
}

impl CommitStats {
    pub fn new(repo_name: String) -> Self {
        CommitStats {
            repo_name,
            total_commits: 0,
            commits_this_month: 0,
            commits_this_week: 0,
            commits_today: 0,
            last_commit_time: DateTime::from_timestamp(0, 0).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct FindGitRepos {
    dir: String,
    num_repos: u32,
    repos_found: Vec<String>,
}

impl FindGitRepos {
    fn new(dir: String) -> Self {
        Self {
            dir,
            num_repos: 0,
            repos_found: vec![],
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SinceCommitStat {
    repo_name: String,
    since_time: NaiveDate,
    total_commits_since: u32,
}

impl SinceCommitStat {
    pub fn new(repo_name: String, since_time: NaiveDate) -> Self {
        Self {
            repo_name,
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
            Commands::Find {
                path,
                format,
                exclude,
            } => {
                handle_find(path, format, exclude);
            }
            Commands::Clean {
                path,
                untracked,
                ignored,
                branches,
                stash,
                ignore_submodules,
            } => {
                clean_repo(
                    path,
                    untracked.unwrap_or(false),
                    ignored.unwrap_or(false),
                    branches.unwrap_or(false),
                    stash.unwrap_or(false),
                    ignore_submodules.unwrap_or(false),
                );
            }
        },
        None => todo!(),
    }
}

fn handle_stats(path: &PathBuf, since_date: &Option<NaiveDate>, format: &Option<u8>) {
    let now = Utc::now();

    let since_date = since_date;

    let format = format.unwrap_or(1);

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

    let mut commit_stat = CommitStats::new(get_repo_name(path));

    match since_date {
        Some(date) => {
            let mut since_commit_stat =
                SinceCommitStat::new(get_repo_name(&path), since_date.unwrap_or_default());

            for oid in &all_oids {
                let commit = repo.find_commit(*oid).unwrap();
                let commit_time: git2::Time = commit.time();
                let converted_time_of_commit: DateTime<chrono::Utc> =
                    DateTime::from_timestamp(commit_time.seconds(), 0).expect("Invalid Time");
                let converted_date_time = to_datetime(*date).unwrap();
                if converted_time_of_commit >= converted_date_time {
                    since_commit_stat.total_commits_since += 1;
                }
            }
            since_commit_stats_display_or_json(format, &since_commit_stat);
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
            commit_stats_display_or_json(format, &commit_stat)
        }
    }
}

fn handle_find(path: &PathBuf, format: &Option<u8>, exclude: &Option<Vec<String>>) {
    let read_dir = fs::read_dir(path).unwrap();

    let mut found_repos = FindGitRepos::new(get_repo_name(path));

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
                found_repos.num_repos += 1;
                found_repos
                    .repos_found
                    .push(entry_path.to_string_lossy().to_string());
            }
        }
    }

    find_repos_display_or_json(format.unwrap_or(1), &found_repos)
}

fn clean_repo(
    path: &PathBuf,
    untracked: bool,
    ignored: bool,
    branches: bool,
    stash: bool,
    ignore_submodules: bool,
) {
    // check if theres a repo in that path
    let repo: Repository = Repository::open(&path).unwrap_or_else(|err| {
        eprintln!("an error occured: {} ", err);
        process::exit(1);
    });

    // create a new git 2 status struct
    let mut opts = StatusOptions::new();

    // choose weather to include ignore files
    opts.include_ignored(ignored);

    // choose weather to add tracked or untracked files
    if untracked {
        opts.include_untracked(true);
    } else {
        opts.include_untracked(false);
    }

    // choose weather ignore submodules
    if ignore_submodules {
        opts.exclude_submodules(true);
    } else {
        opts.exclude_submodules(false);
    }

    // println!("\u{1b}[H\u{1b}[2J \n");

    let statuses: git2::Statuses<'_> = repo
        .statuses(Some(&mut opts))
        .map_err(|e| println!("{}", e.to_string()))
        .unwrap();

    details(statuses);

    println!("This is after the screen clear.");
}

fn convert_time_to_days_ago(time: DateTime<Utc>) -> i64 {
    let now = Utc::now();
    let diffrence = now - time;
    diffrence.num_days()
}

const MIDNIGHT: NaiveTime = NaiveTime::from_hms_opt(0, 0, 0).unwrap();

fn to_datetime(date: NaiveDate) -> Option<DateTime<Local>> {
    date.and_time(MIDNIGHT).and_local_timezone(Local).earliest()
}

fn find_repos_display_or_json(format: u8, find_git_details: &FindGitRepos) {
    match format {
        1 => {
            println!(
                "gitchecker Checking for git dirs in {}: {} dierctories with git found, dirs with git found: {:#?}",
                find_git_details.dir, find_git_details.num_repos, find_git_details.repos_found
            )
        }
        2 => {
            let file_name = format!("gitchecker_{}_find_repos.json", find_git_details.dir);

            let file_path = PathBuf::from("./gitchecker_results").join(&file_name);

            let mut file = File::create(&file_path).expect("Error creating file to write data");

            serde_json::to_writer_pretty(&mut file, &find_git_details).unwrap();

            println!("Repo Found here: {}", &file_path.display());
        }
        _ => eprintln!("unsupported format"),
    }
}

fn since_commit_stats_display_or_json(format: u8, since_details: &SinceCommitStat) {
    match format {
        1 => {
            let print_details = format!(
                "gitchecker: {} commits for {} since {}",
                since_details.total_commits_since,
                since_details.repo_name,
                since_details.since_time
            );

            println!("{print_details}");
        }
        2 => {
            let file_name = format!(
                "gitchecker_{}_since_{}_commit_stats.json",
                since_details.repo_name, since_details.since_time
            );

            let file_path = PathBuf::from("./gitchecker_results").join(&file_name);

            let mut file = File::create(&file_path).expect("Error creating file to write data");

            serde_json::to_writer_pretty(&mut file, &since_details).unwrap();

            println!("Since Commit Stats here: {}", &file_path.display());
        }
        _ => eprintln!("unsupported format"),
    }
}

fn commit_stats_display_or_json(format: u8, commit_stat: &CommitStats) {
    match format {
        1 => {
            let mut commit_stats = format!(
                "
         Repository Statistics for: {:?}
         ==============================================

         📊 COMMIT STATISTICS
         Total commits: {}
         Commits this month: {}
         Commits this week: {}
         Commits today: {}
         Last commit time: {}

         ",
                commit_stat.repo_name,
                commit_stat.total_commits,
                commit_stat.commits_this_month,
                commit_stat.commits_this_week,
                commit_stat.commits_today,
                commit_stat.last_commit_time,
            );

            if convert_time_to_days_ago(commit_stat.last_commit_time) > 0 {
                commit_stats.push_str(&format!(
                    ", {} days ago",
                    convert_time_to_days_ago(commit_stat.last_commit_time)
                ));
            }

            println!("{commit_stats}");
        }
        2 => {
            let file_name = format!("gitchecker_{}_commit_stats.json", commit_stat.repo_name);

            let file_path = PathBuf::from("./gitchecker_results").join(&file_name);

            let mut file = File::create(&file_path).expect("Error creating file to write data");

            serde_json::to_writer_pretty(&mut file, &commit_stat).unwrap();

            println!("Commit Stats here: {}", &file_path.display());
        }
        _ => eprintln!("unsupported format"),
    }
}

fn get_repo_name(path: &PathBuf) -> String {
    let string_path = path.to_str().expect("Invalid string path");
    let vec_path_data: Vec<&str> = string_path.split('/').collect();
    let name = vec_path_data.last().unwrap().to_lowercase();
    name
}

fn details(statuses: git2::Statuses) {
    for status in statuses.iter().filter(|e| e.status() != Status::CURRENT) {
        let path = Path::new(
            status
                .path()
                .expect("expected string representation of path"),
        );

        println!("path {} status {:?}", path.display(), status.status());

        match status.status() {
            Status::WT_NEW => {
                if path.is_dir() {
                    println!("Removing untracked Directory: {} \n", path.display());
                    // fs::remove_dir(path).expect("Failed to remove directory");
                } else if path.is_file() {
                    println!("Removing untracked File: {} \n", path.display());
                    // fs::remove_file(path).expect("Failed to remove path");
                }
            }

            Status::IGNORED => {
                if path.is_dir() {
                    println!("Removing Ignored Directory: {} \n", path.display());
                    // fs::remove_dir(path).expect("Failed to remove directory");
                } else if path.is_file() {
                    println!("Removing Ignored File: {} \n", path.display());
                    // fs::remove_file(path).expect("Failed to remove path");
                }
            }
            _ => println!("Ignored Not now"),
        }
    }
}
