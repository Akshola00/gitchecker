## gitchecker

Command-line tool to analyze Git repositories, find Git projects in a directory, and export stats to JSON.

Built with Rust (edition 2024) and `git2` (libgit2 bindings). CLI powered by `clap`.

### Features

- **Stats**: Compute commit statistics for a repository
  - Total commits
  - Commits in the last month, week, and today
  - Last commit timestamp (and days ago)
  - Optional: commits since a given date (`--since YYYY-MM-DD`)
- **Find**: Scan a directory and list subdirectories that are Git repos
  - Optional: exclude specific directory names
- **Output formats**
  - `--format 1` print to stdout (default)
  - `--format 2` write JSON to `./gitchecker_results/`
- **Clean**: Placeholder subcommand for future cleanup actions

### Installation

Prerequisites:

- Rust toolchain (install via rustup): `https://rustup.rs`

Build:

```bash
cargo build --release
```

The binary will be at `target/release/gitchecker`.

Run in-place during development:

```bash
cargo run -- <subcommand> [OPTIONS]
```

### Usage

Global options:

- `--dry-run` (currently not used by subcommands, reserved for future behavior)

Subcommands:

1) Stats — analyze a repository's statistics

```bash
gitchecker stats --path /path/to/repo [--since YYYY-MM-DD] [--format 1|2]
```

Examples:

```bash
# Print summary to stdout
cargo run -- stats --path /Users/me/projects/my-repo

# Print commits since a specific date
cargo run -- stats --path /Users/me/projects/my-repo --since 2025-09-16

# Save JSON to ./gitchecker_results
cargo run -- stats --path /Users/me/projects/my-repo --format 2
```

Stdout example (`--format 1`):

```text
Repository Statistics for: "my-repo"
================================================

📊 COMMIT STATISTICS
Total commits: 123
Commits this month: 12
Commits this week: 5
Commits today: 1
Last commit time: 2025-09-18 10:15:30 UTC, 1 days ago
```

JSON file names (`--format 2`):

- Overall stats: `gitchecker_<repo>_commit_stats.json`
- Since date: `gitchecker_<repo>_since_<YYYY-MM-DD>_commit_stats.json`

JSON schema (approximate):

```json
{
  "repo_name": "string",
  "total_commits": 0,
  "commits_this_month": 0,
  "commits_this_week": 0,
  "commits_today": 0,
  "last_commit_time": "RFC3339 timestamp"
}
```

For `--since` output:

```json
{
  "repo_name": "string",
  "since_time": "YYYY-MM-DD",
  "total_commits_since": 0
}
```

2) Find — find all Git repositories in a directory

```bash
gitchecker find --path /path/to/scan [--format 1|2] [--exclude name1 name2]
```

Notes:

- Only immediate children are checked (non-recursive).
- A directory is considered a repo if it contains a `.git` directory.
- `--exclude` matches folder names exactly and skips them.

Examples:

```bash
# Print to stdout
cargo run -- find --path /Users/me/projects

# Exclude specific folders
cargo run -- find --path /Users/me/projects --exclude node_modules tmp

# Save JSON to ./gitchecker_results
cargo run -- find --path /Users/me/projects --format 2
```

Stdout example (`--format 1`):

```text
gitchecker Checking for git dirs in projects: 3 dierctories with git found, dirs with git found: [
  "/Users/me/projects/app1",
  "/Users/me/projects/app2",
  "/Users/me/projects/lib"
]
```

JSON file name (`--format 2`): `gitchecker_<dir>_find_repos.json`

JSON schema (approximate):

```json
{
  "dir": "string",            
  "num_repos": 0,
  "repos_found": ["/absolute/path", "..."]
}
```

3) Clean — clean up a repository (placeholder)

```bash
gitchecker clean --path /path/to/repo
```

Currently prints the provided path; cleanup operations are not yet implemented.

### Output directory

When using `--format 2`, JSON files are written to `./gitchecker_results/` relative to your current working directory. Ensure this folder exists or can be created.

### Troubleshooting

- If you see an error opening a repository, verify the `--path` points to a valid Git repository.
- Time calculations are based on your system clock and UTC conversions; ensure system time is correct.
- For macOS/Linux, no special setup for `libgit2` is usually required; the `git2` crate bundles/links what it needs.

### License

Unspecified. If you plan to distribute, add a license of your choice (e.g., MIT/Apache-2.0).


