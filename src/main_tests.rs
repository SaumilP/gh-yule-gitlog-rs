use super::*;

/// Helper to build a GitHubEvent with a specific timestamp.
fn mk_event(event_type: &str, repo: &str, created_at: DateTime<Utc>) -> GitHubEvent {
    GitHubEvent {
        created_at: created_at.to_rfc3339(),
        type_: event_type.to_string(),
        repo: Repo {
            name: repo.to_string(),
        },
    }
}

#[test]
/// Ensures pad_right does not truncate when the string is already long enough.
fn pad_right_no_padding_when_long_enough() {
    let out = pad_right("hello", 3);
    assert_eq!(out, "hello");
}

#[test]
/// Ensures pad_right appends the expected number of spaces.
fn pad_right_adds_spaces() {
    let out = pad_right("hi", 5);
    assert_eq!(out, "hi   ");
}

#[test]
/// Ensures empty events yield empty ticker strings and no ticker flag.
fn parse_events_to_ticker_empty() {
    let (msg, meta, have) = parse_events_to_ticker(&[]);
    assert_eq!(msg, "");
    assert_eq!(meta, "");
    assert!(!have);
}

#[test]
/// Ensures recent events are labeled as "today" in the meta ticker.
fn parse_events_to_ticker_today() {
    let now = Utc::now();
    let events = vec![mk_event("PushEvent", "octo/repo", now)];
    let (msg, meta, have) = parse_events_to_ticker(&events);
    assert!(have);
    assert!(msg.contains("PushEvent in octo/repo"));
    assert!(meta.contains("today"));
}

#[test]
/// Ensures a 1-day offset is labeled as "1 day ago".
fn parse_events_to_ticker_one_day_ago() {
    let ts = Utc::now() - Duration::days(1);
    let events = vec![mk_event("PullRequestEvent", "octo/repo", ts)];
    let (_msg, meta, _have) = parse_events_to_ticker(&events);
    assert!(meta.contains("1 day ago"));
}

#[test]
/// Ensures filtering by days keeps only recent events.
fn filter_events_by_days_keeps_recent() {
    let now = Utc::now();
    let events = vec![
        mk_event("PushEvent", "octo/repo", now - Duration::days(1)),
        mk_event("PushEvent", "octo/old", now - Duration::days(10)),
    ];
    let filtered = filter_events_by_days(events, 5);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].repo.name, "octo/repo");
}

#[test]
/// Ensures non-contribution event types are removed by the contrib filter.
fn filter_contrib_events_removes_non_contrib() {
    let now = Utc::now();
    let mut events = vec![
        mk_event("PushEvent", "octo/repo", now),
        mk_event("PublicEvent", "octo/repo", now),
    ];
    filter_contrib_events(&mut events);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].type_, "PushEvent");
}

#[test]
/// Ensures CLI defaults match the expected baseline values.
fn args_defaults() {
    let args = Args::parse_from(["prog"]);
    assert_eq!(args.username, "SaumilP");
    assert_eq!(args.past_days, 365);
    assert!(!args.contribs);
    assert_eq!(args.speed, 1);
    assert_eq!(args.smoke, 5);
}

#[test]
/// Ensures invalid speed values are rejected by clap.
fn args_rejects_invalid_speed() {
    let err = Args::try_parse_from(["prog", "--speed", "0"])
        .err()
        .expect("expected invalid speed error");
    let msg = err.to_string();
    assert!(msg.contains("speed"));
}

#[test]
/// Ensures invalid smoke values are rejected by clap.
fn args_rejects_invalid_smoke() {
    let err = Args::try_parse_from(["prog", "--smoke", "21"])
        .err()
        .expect("expected invalid smoke error");
    let msg = err.to_string();
    assert!(msg.contains("smoke"));
}

#[test]
/// Ensures the contribs flag is parsed as true when provided.
fn args_contribs_flag_is_true() {
    let args = Args::parse_from(["prog", "--contribs"]);
    assert!(args.contribs);
}

#[test]
/// Ensures filtering an empty input yields an empty output.
fn filter_events_by_days_empty_input() {
    let filtered = filter_events_by_days(Vec::new(), 7);
    assert!(filtered.is_empty());
}

/// Ensures the meta ticker uses plural days for older events.
#[test]
fn parse_events_to_ticker_many_days_ago() {
    let ts = Utc::now() - Duration::days(3);
    let events = vec![mk_event("IssuesEvent", "octo/repo", ts)];
    let (_msg, meta, _have) = parse_events_to_ticker(&events);
    assert!(meta.contains("3 days ago"));
}

/// Ensures message and meta strings have equal segment widths.
#[test]
fn parse_events_to_ticker_padding_matches() {
    let now = Utc::now();
    let events = vec![mk_event("CreateEvent", "octo/long-repo-name", now)];
    let (msg, meta, _have) = parse_events_to_ticker(&events);
    assert_eq!(msg.len(), meta.len());
}
