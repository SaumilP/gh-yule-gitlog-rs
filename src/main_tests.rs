use super::*;

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
fn pad_right_no_padding_when_long_enough() {
    let out = pad_right("hello", 3);
    assert_eq!(out, "hello");
}

#[test]
fn pad_right_adds_spaces() {
    let out = pad_right("hi", 5);
    assert_eq!(out, "hi   ");
}

#[test]
fn parse_events_to_ticker_empty() {
    let (msg, meta, have) = parse_events_to_ticker(&[]);
    assert_eq!(msg, "");
    assert_eq!(meta, "");
    assert!(!have);
}

#[test]
fn parse_events_to_ticker_today() {
    let now = Utc::now();
    let events = vec![mk_event("PushEvent", "octo/repo", now)];
    let (msg, meta, have) = parse_events_to_ticker(&events);
    assert!(have);
    assert!(msg.contains("PushEvent in octo/repo"));
    assert!(meta.contains("today"));
}

#[test]
fn parse_events_to_ticker_one_day_ago() {
    let ts = Utc::now() - Duration::days(1);
    let events = vec![mk_event("PullRequestEvent", "octo/repo", ts)];
    let (_msg, meta, _have) = parse_events_to_ticker(&events);
    assert!(meta.contains("1 day ago"));
}

#[test]
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
fn args_defaults() {
    let args = Args::parse_from(["prog"]);
    assert_eq!(args.username, "SaumilP");
    assert_eq!(args.past_days, 365);
    assert!(!args.contribs);
    assert_eq!(args.speed, 1);
    assert_eq!(args.smoke, 5);
}

#[test]
fn args_rejects_invalid_speed() {
    let err = Args::try_parse_from(["prog", "--speed", "0"])
        .err()
        .expect("expected invalid speed error");
    let msg = err.to_string();
    assert!(msg.contains("speed"));
}

#[test]
fn args_rejects_invalid_smoke() {
    let err = Args::try_parse_from(["prog", "--smoke", "21"])
        .err()
        .expect("expected invalid smoke error");
    let msg = err.to_string();
    assert!(msg.contains("smoke"));
}

#[test]
fn args_contribs_flag_is_true() {
    let args = Args::parse_from(["prog", "--contribs"]);
    assert!(args.contribs);
}

#[test]
fn filter_events_by_days_empty_input() {
    let filtered = filter_events_by_days(Vec::new(), 7);
    assert!(filtered.is_empty());
}
