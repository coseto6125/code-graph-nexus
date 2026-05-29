//! `ecp admin check-update` — hidden, background-only update probe.
//!
//! Spawned detached by the session_start hook (never invoked by a user or the
//! LLM). It decides *whether* to hit the network based on a throttle file, so
//! the network call happens at most once a day, and at most once per 8h after a
//! failure. Reuses the doctor's version logic (`latest_published_version`) so
//! there is one network/parse implementation, not two.
//!
//! Throttle (`<home_ecp>/.update-check.json`):
//!   - succeeded today                  → skip (one check per day)
//!   - last attempt failed < 8h ago     → skip (back off)
//!   - otherwise                         → query
//!
//! On a successful query with a newer remote version, writes
//! `<home_ecp>/.update-available` (consumed once by UserPromptSubmit). Network
//! failure is silent: it only stamps `last_attempt_epoch` so the 8h backoff
//! applies, and never writes a notification or surfaces an error.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use ecp_core::registry::{atomic_write_json, resolve_home_ecp};
use ecp_core::EcpError;
use serde::{Deserialize, Serialize};

use crate::commands::admin::doctor::checks::version::{latest_published_version, parse_semver};
use crate::git::safe_exec;

const DAY_SECS: u64 = 86_400;
const FAIL_BACKOFF_SECS: u64 = 8 * 3_600;

/// Persisted throttle state. Missing / unreadable file is treated as "never
/// checked" — the probe runs. Success and failure are tracked separately so the
/// two throttle rules (daily cap, 8h failure backoff) don't have to be inferred
/// from one ambiguous timestamp.
#[derive(Default, Serialize, Deserialize)]
struct CheckState {
    /// Day bucket (`epoch / 86400`) of the last *successful* network query.
    last_success_day: u64,
    /// Unix epoch (secs) of the last *failed* attempt. `0` = no pending
    /// failure (cleared on success). Drives the 8h backoff.
    last_failure_epoch: u64,
    /// Latest version string seen from the remote, for the notification text.
    latest_version: String,
}

pub fn run() -> Result<(), EcpError> {
    let home_ecp = resolve_home_ecp();
    let state_path = home_ecp.join(".update-check.json");
    let now = now_epoch();
    let state = read_state(&state_path);

    if !should_query(&state, now) {
        return Ok(());
    }

    // A restricted-network sandbox would block until the timeout backstop; treat
    // it as a failure so the 8h backoff applies, then return silently.
    if safe_exec::sandbox_network_restricted() {
        write_state(&state_path, &on_failure(&state, now));
        return Ok(());
    }

    match latest_published_version() {
        Some(latest) => {
            let local = parse_semver(env!("CARGO_PKG_VERSION"));
            let latest_str = format!("{}.{}.{}", latest.0, latest.1, latest.2);
            // Success clears any pending failure backoff.
            write_state(
                &state_path,
                &CheckState {
                    last_success_day: now / DAY_SECS,
                    last_failure_epoch: 0,
                    latest_version: latest_str.clone(),
                },
            );
            if local.map(|l| latest > l).unwrap_or(false) {
                write_notification(&home_ecp, &latest_str);
            }
        }
        // Silent failure: stamp the backoff clock, write no notification.
        None => write_state(&state_path, &on_failure(&state, now)),
    }
    Ok(())
}

/// Query when we haven't succeeded today AND we're past the 8h failure backoff.
/// The two rules are independent: a same-day success blocks re-query regardless
/// of failures; a recent failure blocks re-query even on a fresh day.
fn should_query(state: &CheckState, now: u64) -> bool {
    let succeeded_today = state.last_success_day == now / DAY_SECS;
    let in_backoff = state.last_failure_epoch != 0
        && now.saturating_sub(state.last_failure_epoch) < FAIL_BACKOFF_SECS;
    !succeeded_today && !in_backoff
}

/// Next state after a failed attempt: keep the prior success day / version
/// (a stale "available" notice is better than none), only move the backoff clock.
fn on_failure(prev: &CheckState, now: u64) -> CheckState {
    CheckState {
        last_success_day: prev.last_success_day,
        last_failure_epoch: now,
        latest_version: prev.latest_version.clone(),
    }
}

fn write_notification(home_ecp: &Path, latest: &str) {
    let local = env!("CARGO_PKG_VERSION");
    let body = format!(
        "ecp v{latest} is available (you have v{local}). Upgrade: `ecp admin doctor version` shows the command for your install channel."
    );
    let _ = std::fs::write(home_ecp.join(".update-available"), body);
}

fn read_state(path: &Path) -> CheckState {
    std::fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn write_state(path: &Path, state: &CheckState) {
    let _ = atomic_write_json(path, state);
}

fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(success_day: u64, failure_epoch: u64) -> CheckState {
        CheckState {
            last_success_day: success_day,
            last_failure_epoch: failure_epoch,
            latest_version: String::new(),
        }
    }

    #[test]
    fn first_ever_check_queries() {
        // Never checked (all zero) → query.
        assert!(should_query(&state(0, 0), 10 * DAY_SECS));
    }

    #[test]
    fn same_day_success_skips() {
        let now = 10 * DAY_SECS + 500;
        // Succeeded today, no failure pending.
        assert!(!should_query(&state(10, 0), now));
    }

    #[test]
    fn same_day_success_skips_even_with_old_failure() {
        let now = 10 * DAY_SECS + 500;
        // Daily cap wins regardless of an ancient failure timestamp.
        assert!(!should_query(&state(10, now - 100), now));
    }

    #[test]
    fn new_day_after_success_queries() {
        let now = 11 * DAY_SECS + 500;
        // Last success was yesterday (day 10), no pending failure.
        assert!(should_query(&state(10, 0), now));
    }

    #[test]
    fn recent_failure_backs_off_even_on_new_day() {
        // Failed 1h ago on a fresh day → still inside 8h backoff.
        let now = 12 * DAY_SECS + 3_600;
        assert!(!should_query(&state(10, now - 3_600), now));
    }

    #[test]
    fn failure_past_8h_retries() {
        // Failed 9h ago → past the 8h backoff → retry.
        let now = 12 * DAY_SECS + 10 * 3_600;
        assert!(should_query(&state(10, now - 9 * 3_600), now));
    }
}
