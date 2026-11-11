//! Fetching a hub over HTTP, and the date the device compares against.
//!
//! The origin is not trusted: what comes back is verified against the anchor
//! before it is read, and a pack is unpacked and hashed before it reaches a
//! jail. So this module does the plain work of GET and nothing clever: no
//! caching headers to reason about, no partial trust.
use crate::pack::Pack;

/// A hub reachable as `<base>/catalog.json` and `<base>/<artifact>.pack.json`.
pub struct Remote {
    base: String,
}

impl Remote {
    pub fn new(base: &str) -> Self {
        Remote { base: base.trim_end_matches('/').to_string() }
    }

    fn get(&self, path: &str) -> Result<String, String> {
        let url = format!("{}/{}", self.base, path.trim_start_matches('/'));
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(30)))
            .build()
            .new_agent();
        let mut response = agent.get(&url).call().map_err(|e| format!("{url}: {e}"))?;
        response
            .body_mut()
            .with_config()
            .limit(64 * 1024 * 1024)
            .read_to_string()
            .map_err(|e| format!("{url}: {e}"))
    }

    /// The catalog's text, unverified.
    pub fn catalog(&self) -> Result<String, String> {
        self.get("catalog.json")
    }

    /// The GitHub-attested channel. A missing or invalid response must not
    /// cause a caller to fetch the legacy catalog instead.
    pub fn github_catalog(&self) -> Result<String, String> {
        self.get("catalog-v2.json")
    }

    /// An artifact's pack, unverified.
    pub fn pack(&self, artifact: &str) -> Result<Pack, String> {
        let text = self.get(&format!("{artifact}.pack.json"))?;
        serde_json::from_str(&text).map_err(|e| format!("{artifact}: not a pack: {e}"))
    }

    /// A shared component's file (`artifacts/<id>-<version>.wasm`, App Hub
    /// ADR 0003), unverified and at most `max` bytes: the store checks its
    /// size and digest against the verified catalog before keeping it.
    pub fn component(&self, artifact: &str, max: u64) -> Result<Vec<u8>, String> {
        let url = format!("{}/{}", self.base, artifact.trim_start_matches('/'));
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(30)))
            .build()
            .new_agent();
        let mut response = agent.get(&url).call().map_err(|e| format!("{url}: {e}"))?;
        response.body_mut().with_config().limit(max).read_to_vec().map_err(|e| format!("{url}: {e}"))
    }
}

/// Today's date as `YYYY-MM-DD`, from the system clock. What the freshness
/// window and the admission record are measured against.
pub fn today() -> String {
    utc_date(unix_now() / 86_400)
}

/// Now as a UTC time, `YYYY-MM-DDTHH:MM:SSZ` (ISO 8601): when a catalog was
/// published. A device shows it on its own calendar; a date alone can't be,
/// since its day began at a different hour for everyone. Freshness reads the
/// date part, as it reads a catalog that has only a date.
pub fn utc_now() -> String {
    let secs = unix_now();
    let day = secs % 86_400;
    format!("{}T{:02}:{:02}:{:02}Z", utc_date(secs / 86_400), day / 3600, day % 3600 / 60, day % 60)
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// `YYYY-MM-DD` of a day counted from 1970-01-01.
fn utc_date(days: i64) -> String {
    // Civil-from-days (Howard Hinnant's algorithm), proleptic Gregorian.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    #[test]
    fn today_is_a_plausible_iso_date() {
        let t = super::today();
        assert_eq!(t.len(), 10);
        assert!(t.starts_with("20"));
        assert_eq!(crate::client::days_between(&t, &t), Some(0));
    }

    #[test]
    fn a_catalog_is_published_at_a_utc_time_and_its_day_is_still_today() {
        let (now, today) = (super::utc_now(), super::today());
        assert_eq!(now.len(), 20, "{now}");
        assert!(now.ends_with('Z') && now.as_bytes()[10] == b'T', "{now}");
        // A clock passing midnight between the two calls is the only way they differ.
        assert!(now.starts_with(&today) || crate::client::days_between(&now, &today) == Some(1), "{now} {today}");
        assert_eq!(crate::client::days_between("2026-11-06T00:12:00Z", "2026-11-06"), Some(0));
        assert_eq!(crate::client::days_between("2026-11-06T23:59:59Z", "2026-11-20"), Some(14));
        assert_eq!(super::utc_date(0), "1970-01-01");
        assert_eq!(super::utc_date(20_398), "2025-11-06");
    }

    #[test]
    fn days_between_counts_civil_days() {
        assert_eq!(crate::client::days_between("2026-09-01", "2026-09-20"), Some(19));
        assert_eq!(crate::client::days_between("2026-02-27", "2026-03-02"), Some(3));
        assert_eq!(crate::client::days_between("2026-09-20", "2026-09-01"), Some(0), "the future is not stale");
        assert_eq!(crate::client::days_between("garbage", "2026-09-20"), None);
    }
}
