use reqwest::Client;
use serde::Deserialize;
use chrono::{Utc, Duration, DateTime};
use tokio;
use clap::Parser;
use animation;

#[derive(Parser)]
#[command(author, version, about = "GitHub CLI extension for animated git log visualization", long_about = None)]
struct Args {
    /// GitHub username
    #[arg(long, default_value = "SaumilP")]
    username: String,

    /// Number of past days to look back
    #[arg(long, default_value_t = 365)]
    past_days: u32,

    /// Filter for contribution events only
    #[arg(long)]
    contribs: bool,

    /// Animation speed (1-10, default 1)
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=10))]
    speed: u8,

    /// Smoke factor (0-20, default 5)
    #[arg(long, default_value_t = 5, value_parser = clap::value_parser!(u8).range(0..=20))]
    smoke: u8,
}

#[derive(Debug, Deserialize)]
struct GitHubEvent {
    pub created_at: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub repo: Repo,
}

#[derive(Debug, Deserialize)]
struct Repo {
    pub name: String,
}

fn parse_events_to_ticker(events: &[GitHubEvent]) -> (String, String, bool) {
    if events.is_empty() {
        return ("".to_string(), "".to_string(), false);
    }

    let mut msg_segs = Vec::new();
    let mut meta_segs = Vec::new();

    for event in events {
        let message = format!("{} in {}", event.type_, event.repo.name);
        let created_at = DateTime::parse_from_rfc3339(&event.created_at).unwrap().with_timezone(&Utc);
        let now = Utc::now();
        let duration = now.signed_duration_since(created_at);
        let days = duration.num_days();
        let meta = if days == 0 {
            "today".to_string()
        } else if days == 1 {
            "1 day ago".to_string()
        } else {
            format!("{} days ago", days)
        };

        let msg_runes = message.chars().count();
        let meta_runes = meta.chars().count();
        let segment_width = msg_runes.max(meta_runes) + 4;

        msg_segs.push(pad_right(&message, segment_width));
        meta_segs.push(pad_right(&meta, segment_width));
    }

    (msg_segs.join(""), meta_segs.join(""), true)
}

fn pad_right(s: &str, n: usize) -> String {
    let len = s.chars().count();
    if len >= n {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(n - len))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Try to get GitHub token from gh CLI
    let token = std::process::Command::new("gh")
        .args(&["auth", "token"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        });

    // Set up the client and the GitHub API URL
    let client = Client::new();
    let url = format!("https://api.github.com/users/{}/events", args.username);

    // Build the request
    let mut request = client
        .get(&url)
        .header("User-Agent", "gh-yule-gitlog-rs");

    if let Some(token) = token {
        request = request.header("Authorization", format!("token {}", token));
    }

    // Fetch events from GitHub API
    let response = request
        .send()
        .await?
        .json::<Vec<GitHubEvent>>()
        .await?;

    // Get the date from past_days ago
    let past_date = Utc::now() - Duration::days(args.past_days as i64);

    // Filter events that happened in the past days
    let mut filtered_events: Vec<_> = response
        .into_iter()
        .filter(|event| {
            let event_date = chrono::DateTime::parse_from_rfc3339(&event.created_at)
                .unwrap()
                .with_timezone(&Utc);
            event_date > past_date
        })
        .collect();

    // If --contribs is specified, filter for contribution events
    if args.contribs {
        let contrib_types = vec![
            "PushEvent",
            "PullRequestEvent",
            "IssuesEvent",
            "IssueCommentEvent",
            "PullRequestReviewEvent",
            "PullRequestReviewCommentEvent",
            "CreateEvent",
            "ForkEvent",
            "WatchEvent",
        ];
        filtered_events.retain(|event| contrib_types.contains(&event.type_.as_str()));
    }

    // Parse events to ticker
    let (msg_text, meta_text, have_ticker) = parse_events_to_ticker(&filtered_events);

    // Start animation
    animation::run_animation(args.contribs, msg_text, meta_text, have_ticker, args.speed, filtered_events.len(), args.smoke)?;

    Ok(())
}