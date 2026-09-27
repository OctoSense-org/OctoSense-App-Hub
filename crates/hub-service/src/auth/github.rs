//! GitHub OAuth device flow. Only GitHub's stable numeric user ID becomes an
//! account subject; provider access tokens are never persisted or returned.
use super::{Clock, DeviceChallenge, IdentityAssertion, IdentityProvider, AUDIENCE};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

trait GitHubHttp: Send + Sync {
    fn device_code(&self, client_id: &str) -> Result<Value, String>;
    fn access_token(&self, client_id: &str, device_code: &str) -> Result<Value, String>;
    fn user(&self, access_token: &str) -> Result<Value, String>;
}

struct NetworkGitHubHttp;
impl NetworkGitHubHttp {
    fn agent() -> ureq::Agent {
        ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(15)))
            .build()
            .new_agent()
    }
    fn json(mut response: ureq::http::Response<ureq::Body>) -> Result<Value, String> {
        let body = response
            .body_mut()
            .with_config()
            .limit(8 * 1024)
            .read_to_string()
            .map_err(|_| "GitHub response unavailable".to_owned())?;
        serde_json::from_str(&body).map_err(|_| "GitHub response invalid".to_owned())
    }
}
impl GitHubHttp for NetworkGitHubHttp {
    fn device_code(&self, client_id: &str) -> Result<Value, String> {
        let response = Self::agent()
            .post("https://github.com/login/device/code")
            .header("accept", "application/json")
            .send_form([("client_id", client_id)])
            .map_err(|_| "GitHub device authorization unavailable".to_owned())?;
        Self::json(response)
    }
    fn access_token(&self, client_id: &str, device_code: &str) -> Result<Value, String> {
        let response = Self::agent()
            .post("https://github.com/login/oauth/access_token")
            .header("accept", "application/json")
            .send_form([
                ("client_id", client_id),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .map_err(|_| "GitHub token exchange unavailable".to_owned())?;
        Self::json(response)
    }
    fn user(&self, access_token: &str) -> Result<Value, String> {
        let response = Self::agent()
            .get("https://api.github.com/user")
            .header("accept", "application/vnd.github+json")
            .header("user-agent", "OctoSense-App-Hub")
            .header("x-github-api-version", "2026-03-10")
            .header("authorization", &format!("Bearer {access_token}"))
            .call()
            .map_err(|_| "GitHub user lookup unavailable".to_owned())?;
        Self::json(response)
    }
}

struct PollState {
    interval: i64,
    next_poll: i64,
    expires_at: i64,
}

pub struct GitHubDeviceProvider {
    client_id: String,
    clock: Arc<dyn Clock>,
    http: Arc<dyn GitHubHttp>,
    pending: Mutex<HashMap<String, PollState>>,
}

impl GitHubDeviceProvider {
    pub fn new(client_id: &str, clock: Arc<dyn Clock>) -> Result<Self, String> {
        if client_id.len() < 3
            || client_id.len() > 128
            || !client_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        {
            return Err("invalid GitHub OAuth client ID".into());
        }
        Ok(Self {
            client_id: client_id.into(),
            clock,
            http: Arc::new(NetworkGitHubHttp),
            pending: Mutex::new(HashMap::new()),
        })
    }
}

impl IdentityProvider for GitHubDeviceProvider {
    fn begin(&self) -> Result<DeviceChallenge, String> {
        let response = self.http.device_code(&self.client_id)?;
        let code = response["device_code"]
            .as_str()
            .ok_or("GitHub device code missing")?;
        let user_code = response["user_code"]
            .as_str()
            .ok_or("GitHub user code missing")?;
        let verification_uri = response["verification_uri"]
            .as_str()
            .ok_or("GitHub verification URL missing")?;
        let expires_in = response["expires_in"]
            .as_u64()
            .ok_or("GitHub expiry missing")?;
        let interval = response["interval"]
            .as_i64()
            .ok_or("GitHub polling interval missing")?;
        if code.is_empty()
            || code.len() > 256
            || user_code.is_empty()
            || user_code.len() > 128
            || verification_uri != "https://github.com/login/device"
            || expires_in == 0
            || expires_in > 900
            || !(1..=120).contains(&interval)
        {
            return Err("GitHub device authorization invalid".into());
        }
        let now = self.clock.now();
        let mut pending = self.pending.lock().unwrap();
        pending.retain(|_, state| state.expires_at > now);
        if pending.len() >= 1_024 || pending.contains_key(code) {
            return Err("GitHub device login capacity reached".into());
        }
        pending.insert(
            code.into(),
            PollState {
                interval,
                next_poll: now + interval,
                expires_at: now + expires_in as i64,
            },
        );
        Ok(DeviceChallenge {
            device_code: code.into(),
            user_code: user_code.into(),
            verification_uri: verification_uri.into(),
            expires_in,
        })
    }

    fn poll(&self, device_code: &str) -> Result<Option<IdentityAssertion>, String> {
        let now = self.clock.now();
        {
            let mut pending = self.pending.lock().unwrap();
            let state = pending
                .get_mut(device_code)
                .ok_or("unknown device authorization")?;
            if now >= state.expires_at {
                pending.remove(device_code);
                return Err("GitHub device authorization expired".into());
            }
            if now < state.next_poll {
                return Ok(None);
            }
            state.next_poll = now + state.interval;
        }
        let response = self.http.access_token(&self.client_id, device_code)?;
        if let Some(error) = response["error"].as_str() {
            match error {
                "authorization_pending" => return Ok(None),
                "slow_down" => {
                    let mut pending = self.pending.lock().unwrap();
                    if let Some(state) = pending.get_mut(device_code) {
                        state.interval = response["interval"]
                            .as_i64()
                            .filter(|value| *value > state.interval && *value <= 120)
                            .unwrap_or((state.interval + 5).min(120));
                        state.next_poll = now + state.interval;
                    }
                    return Ok(None);
                }
                _ => {
                    self.pending.lock().unwrap().remove(device_code);
                    return Err("GitHub device authorization denied or expired".into());
                }
            }
        }
        let token = response["access_token"]
            .as_str()
            .ok_or("GitHub access token missing")?;
        if token.is_empty()
            || token.len() > 512
            || response["token_type"]
                .as_str()
                .is_none_or(|kind| !kind.eq_ignore_ascii_case("bearer"))
        {
            return Err("GitHub token response invalid".into());
        }
        let user = self.http.user(token)?;
        let id = user["id"]
            .as_u64()
            .filter(|id| *id > 0)
            .ok_or("GitHub user ID missing")?;
        self.pending.lock().unwrap().remove(device_code);
        Ok(Some(IdentityAssertion {
            provider: "github".into(),
            subject: id.to_string(),
            audience: AUDIENCE.into(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::VecDeque,
        sync::atomic::{AtomicI64, Ordering},
    };
    struct TestClock(AtomicI64);
    impl Clock for TestClock {
        fn now(&self) -> i64 {
            self.0.load(Ordering::Relaxed)
        }
    }
    struct TestHttp(Mutex<VecDeque<Value>>);
    impl GitHubHttp for TestHttp {
        fn device_code(&self, _: &str) -> Result<Value, String> {
            Ok(self.0.lock().unwrap().pop_front().unwrap())
        }
        fn access_token(&self, _: &str, _: &str) -> Result<Value, String> {
            Ok(self.0.lock().unwrap().pop_front().unwrap())
        }
        fn user(&self, _: &str) -> Result<Value, String> {
            Ok(self.0.lock().unwrap().pop_front().unwrap())
        }
    }
    #[test]
    fn device_flow_obeys_interval_and_uses_stable_numeric_subject() {
        let clock = Arc::new(TestClock(AtomicI64::new(100)));
        let http = Arc::new(TestHttp(Mutex::new(VecDeque::from([
            serde_json::json!({"device_code":"secret-code","user_code":"ABCD-1234",
                "verification_uri":"https://github.com/login/device","expires_in":900,"interval":5}),
            serde_json::json!({"error":"authorization_pending"}),
            serde_json::json!({"access_token":"private-token","token_type":"bearer"}),
            serde_json::json!({"id":42,"login":"mutable-name"}),
        ]))));
        let mut provider = GitHubDeviceProvider::new("Iv1.example", clock.clone()).unwrap();
        provider.http = http.clone();
        let challenge = provider.begin().unwrap();
        assert_eq!(challenge.expires_in, 900);
        assert!(provider.poll(&challenge.device_code).unwrap().is_none());
        clock.0.store(105, Ordering::Relaxed);
        assert!(provider.poll(&challenge.device_code).unwrap().is_none());
        assert!(provider.poll(&challenge.device_code).unwrap().is_none());
        clock.0.store(110, Ordering::Relaxed);
        let identity = provider.poll(&challenge.device_code).unwrap().unwrap();
        assert_eq!(identity.provider, "github");
        assert_eq!(identity.subject, "42");
        assert_eq!(identity.audience, AUDIENCE);
        assert!(provider.poll(&challenge.device_code).is_err());
        assert!(http.0.lock().unwrap().is_empty());
    }
}
