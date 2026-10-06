use std::time::Duration;

use super::{CATCH_UP_WINDOW, RunningGuard, keep_catching_up};

#[test]
fn keeps_going_while_work_remains_inside_the_window() {
    assert!(keep_catching_up(3, Duration::from_secs(30)));
}

#[test]
fn stops_when_the_queue_is_empty() {
    assert!(!keep_catching_up(0, Duration::from_secs(30)));
}

#[test]
fn stops_when_the_window_closes() {
    assert!(!keep_catching_up(3, CATCH_UP_WINDOW));
}

#[test]
fn only_one_run_at_a_time() {
    let first = RunningGuard::acquire().expect("first run starts");
    assert!(RunningGuard::acquire().is_none());
    drop(first);
    assert!(RunningGuard::acquire().is_some());
}
