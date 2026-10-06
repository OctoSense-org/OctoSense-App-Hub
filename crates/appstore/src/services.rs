//! Host services: what a contained app asks the shell to do for it.
//!
//! An app's script calls `host.request("mail.list", {…}, fn(r){…})`. The
//! isolate refuses the call unless the app's policy grants the family (`mail`);
//! what is granted is queued, and the Card runner hands it here. A service
//! registered for the family does the work, in Rust, with whatever it holds
//! that the app must not: a socket, a credential, a device. The app gets data
//! back, never the means.
//!
//! Some work needs the person, not the app: typing a password, approving an
//! account. A service raises a **sheet** for that: a host-owned surface the
//! runner draws over the app, in an isolate of its own under no app's policy.
//! Its script calls the same `host.request`, and those calls arrive marked
//! `from_sheet`, so a service accepts a password only from its own sheet and
//! never from the app.
//!
//! **Secrets are the host's.** A contained app never collects a password,
//! a PIN or a code: its password fields take no input (the runtime refuses
//! them in a policed isolate), and a service method that takes one lives
//! under `<family>.sheet.`, which [`dispatch`] accepts only from the sheet,
//! before any service sees the call. An app cannot open a sheet either:
//! only a service can, through [`ServiceHost`].
//!
//! Answers can come later, from a worker thread: [`Replier::send`] queues the
//! result and wakes the UI, and the runner delivers it to the isolate that
//! asked on its next event.
//!
//! **Every request settles.** Each one answers exactly once: with the
//! service's reply, or with an error when it waited longer than its service
//! allows ([`DEFAULT_TIMEOUT`], or [`HostService::timeout`]). A request whose
//! service raised a sheet waits for the person instead, and its clock starts
//! again when the sheet closes. A host drops an isolate's requests and
//! answers when the isolate closes ([`cancel_heap`]), so nothing reaches a
//! replacement. The requests waiting, their arguments and their answers are
//! bounded, and malformed arguments are refused rather than read as `null`.
//!
//! **The surface decides about sheets.** Every call carries `may_prompt`,
//! the isolate's surface permission: an app in the foreground may have a
//! service raise a sheet over it; a home-screen tile or an agent's tool call
//! may not, and a service that tries is answered with a refusal.
use makepad_widgets::makepad_platform::SignalToUI;
use makepad_widgets::{Cx, SplashRef};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// How long a request may wait for its service, unless the service says
/// otherwise ([`HostService::timeout`]).
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
/// Requests one isolate may have waiting at once, and all isolates together.
pub const MAX_PENDING_PER_ISOLATE: usize = 32;
pub const MAX_PENDING: usize = 256;
/// The largest arguments a request carries, and the largest answer.
pub const MAX_ARGS_BYTES: usize = 1 << 20;
pub const MAX_REPLY_BYTES: usize = 4 << 20;
/// An answer nobody takes (its isolate went away without closing) is dropped
/// after this long.
pub const REPLY_TTL: Duration = Duration::from_secs(120);

/// One request from an app (or from a service's sheet over it).
#[derive(Clone, Debug)]
pub struct ServiceCall {
    /// The app the request is for: its manifest id.
    pub app_id: String,
    /// The full service name, `family.method`.
    pub service: String,
    pub args: Value,
    /// It came from the service's own sheet, which the person typed into,
    /// not from the app.
    pub from_sheet: bool,
    /// May the service raise a sheet for this call: true for an app in the
    /// foreground, false for a background surface (a home-screen tile, an
    /// agent's tool call). A service that needs the person should refuse
    /// early without it; a sheet raised anyway is refused for it.
    pub may_prompt: bool,
    /// A directory only the host can reach, for the service's own state
    /// (accounts, secrets, caches): outside every app's jail.
    pub host_dir: PathBuf,
}

impl ServiceCall {
    /// The part of the service name after the family: `list` in `mail.list`.
    pub fn method(&self) -> &str {
        self.service.split_once('.').map(|(_, m)| m).unwrap_or("")
    }
}

/// Where one request's answer goes. Cheap to move to a worker thread.
#[derive(Clone, Debug)]
pub struct Replier {
    heap_key: usize,
    req_id: u64,
}

/// An answer waiting for its isolate: heap, request, result, and when it
/// was queued.
type QueuedReply = (usize, u64, Result<String, String>, Instant);

static REPLIES: Mutex<Vec<QueuedReply>> = Mutex::new(Vec::new());

/// A request its service has not answered yet.
struct Pending {
    app_id: String,
    timeout: Duration,
    /// None while a sheet the service raised for it is up.
    deadline: Option<Instant>,
}

static PENDING: Mutex<Option<HashMap<(usize, u64), Pending>>> = Mutex::new(None);

fn with_pending<R>(f: impl FnOnce(&mut HashMap<(usize, u64), Pending>) -> R) -> R {
    f(PENDING.lock().unwrap().get_or_insert_with(HashMap::new))
}

fn queue_reply(heap_key: usize, req_id: u64, result: Result<String, String>) {
    REPLIES.lock().unwrap().push((heap_key, req_id, result, Instant::now()));
    SignalToUI::set_ui_signal();
}

impl Replier {
    /// Answer the request. An answer after the request timed out, or after
    /// its isolate closed, or a second answer, goes nowhere.
    pub fn send(self, result: Result<Value, String>) {
        if with_pending(|pending| pending.remove(&(self.heap_key, self.req_id))).is_none() {
            return;
        }
        let result = result.map(|value| value.to_string()).and_then(|json| {
            if json.len() > MAX_REPLY_BYTES {
                Err(format!("the service's answer exceeds {} MiB", MAX_REPLY_BYTES >> 20))
            } else {
                Ok(json)
            }
        });
        queue_reply(self.heap_key, self.req_id, result);
    }
}

/// Start waiting for one request's answer, or refuse it when too many wait.
fn track(heap_key: usize, req_id: u64, app_id: &str, timeout: Duration) -> Result<(), String> {
    start_sweeper();
    with_pending(|pending| {
        let mine = pending.keys().filter(|(heap, _)| *heap == heap_key).count();
        if mine >= MAX_PENDING_PER_ISOLATE || pending.len() >= MAX_PENDING {
            return Err("too many host requests are waiting; try again when some have answered".into());
        }
        let deadline = Some(Instant::now() + timeout);
        pending.insert((heap_key, req_id), Pending { app_id: app_id.to_string(), timeout, deadline });
        Ok(())
    })
}

/// Answer, with a timeout, every request of the matching isolates whose
/// deadline passed by `now`.
fn expire_pending(now: Instant, heaps: impl Fn(usize) -> bool) {
    let expired: Vec<(usize, u64)> = with_pending(|pending| {
        let keys: Vec<_> = pending
            .iter()
            .filter(|((heap, _), entry)| heaps(*heap) && entry.deadline.is_some_and(|deadline| deadline <= now))
            .map(|(key, _)| *key)
            .collect();
        for key in &keys {
            pending.remove(key);
        }
        keys
    });
    for (heap_key, req_id) in expired {
        queue_reply(heap_key, req_id, Err("the host service timed out".into()));
    }
}

/// Drop answers of the matching isolates that nobody took within [`REPLY_TTL`].
fn purge_replies(now: Instant, heaps: impl Fn(usize) -> bool) {
    REPLIES.lock().unwrap().retain(|(heap, _, _, queued)| !(heaps(*heap) && now.duration_since(*queued) > REPLY_TTL));
}

/// A sheet is up for this request: it waits for the person, not the clock.
/// A sheet it replaced no longer holds its own request.
fn hold_for_sheet(heap_key: usize, req_id: u64, app_id: &str) {
    let now = Instant::now();
    with_pending(|pending| {
        for (key, entry) in pending.iter_mut() {
            if entry.app_id == app_id && entry.deadline.is_none() && *key != (heap_key, req_id) {
                entry.deadline = Some(now + entry.timeout);
            }
        }
        if let Some(entry) = pending.get_mut(&(heap_key, req_id)) {
            entry.deadline = None;
        }
    });
}

/// The sheet over `app_id` closed: its requests wait on the clock again.
fn resume_after_sheet(app_id: &str, now: Instant) {
    with_pending(|pending| {
        for entry in pending.values_mut() {
            if entry.app_id == app_id && entry.deadline.is_none() {
                entry.deadline = Some(now + entry.timeout);
            }
        }
    });
}

/// Time requests out while no event arrives. Without threads (the web), the
/// hosts' pumps do it on their next event.
fn start_sweeper() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static STARTED: std::sync::Once = std::sync::Once::new();
        STARTED.call_once(|| {
            let spawned = std::thread::Builder::new().name("host-requests".into()).spawn(|| loop {
                std::thread::sleep(Duration::from_millis(250));
                let now = Instant::now();
                expire_pending(now, |_| true);
                purge_replies(now, |_| true);
            });
            if let Err(e) = spawned {
                makepad_widgets::error!("host services: cannot start the request timer: {e}");
            }
        });
    }
}

/// Forget an isolate that is closing: its waiting requests and its queued
/// answers. A late answer for it goes nowhere, and never to a replacement.
pub fn cancel_heap(heap_key: usize) {
    with_pending(|pending| pending.retain(|(heap, _), _| *heap != heap_key));
    REPLIES.lock().unwrap().retain(|(heap, _, _, _)| *heap != heap_key);
}

/// A request's arguments, or why it is refused before any service sees it.
fn request_args(service: &str, args_json: &str) -> Result<Value, String> {
    let named = service.len() <= 256 && service.split_once('.').is_some_and(|(family, method)| !family.is_empty() && !method.is_empty());
    if !named {
        return Err(format!("{service:?} is not a family.method service name"));
    }
    if args_json.len() > MAX_ARGS_BYTES {
        return Err(format!("the request's arguments exceed {} MiB", MAX_ARGS_BYTES >> 20));
    }
    serde_json::from_str(args_json).map_err(|e| format!("the request's arguments are not valid JSON: {e}"))
}

/// What a service may ask of the runner that called it.
pub trait ServiceHost {
    /// Show a sheet over the app: `body` is a Splash program, run in a
    /// host-owned isolate. Replaces any sheet already up.
    fn open_sheet(&mut self, body: String);
    fn close_sheet(&mut self);
}

pub trait HostService: Send {
    /// The capability family this service answers: `mail`.
    fn family(&self) -> &'static str;
    /// How long a call may wait for its answer. A sheet the service raises
    /// holds its call open, whatever this says.
    fn timeout(&self, _call: &ServiceCall) -> Duration {
        DEFAULT_TIMEOUT
    }
    fn call(&mut self, call: ServiceCall, reply: Replier, host: &mut dyn ServiceHost);
}

static SERVICES: Mutex<Vec<Box<dyn HostService>>> = Mutex::new(Vec::new());

/// Offer a service to every app granted its family. A second registration
/// for a family replaces the first.
pub fn register_host_service(service: Box<dyn HostService>) {
    let mut services = SERVICES.lock().unwrap();
    let family = service.family();
    services.retain(|s| s.family() != family);
    services.push(service);
}

pub fn has_service(family: &str) -> bool {
    SERVICES.lock().unwrap().iter().any(|s| s.family() == family)
}

/// Hand one request to the service for its family. No service: the app hears
/// so, rather than waiting forever; a service that never answers times out.
pub fn dispatch(call: ServiceCall, heap_key: usize, req_id: u64, host: &mut dyn ServiceHost) {
    let family = call.service.split('.').next().unwrap_or("").to_string();
    let mut services = SERVICES.lock().unwrap();
    let service = services.iter().position(|s| s.family() == family);
    let timeout = service.map(|i| services[i].timeout(&call)).unwrap_or(DEFAULT_TIMEOUT);
    if let Err(refusal) = track(heap_key, req_id, &call.app_id, timeout) {
        drop(services);
        queue_reply(heap_key, req_id, Err(refusal));
        return;
    }
    let reply = Replier { heap_key, req_id };
    // What the person types on a sheet reaches only the sheet's methods,
    // whatever the service does: an app calling one is refused here.
    if call.method().starts_with("sheet.") && !call.from_sheet {
        drop(services);
        reply.send(Err(format!("{} is for the host's sheet, not an app", call.service)));
        return;
    }
    match service {
        Some(i) => services[i].call(call, reply, host),
        None => {
            drop(services);
            reply.send(Err(format!("no service answers {family:?} on this device")));
        }
    }
}

/// The answers ready for these isolates, taken off the queue.
pub fn take_replies_for(heap_keys: &[usize]) -> Vec<(usize, u64, Result<String, String>)> {
    let mut replies = REPLIES.lock().unwrap();
    let (mine, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut *replies).into_iter().partition(|(heap, _, _, _)| heap_keys.contains(heap));
    *replies = rest;
    mine.into_iter().map(|(heap, req_id, result, _)| (heap, req_id, result)).collect()
}

/// The sheet changes a service asked for during one dispatch, applied after,
/// for the request being dispatched.
struct SheetOps {
    change: Option<Option<String>>,
    app_id: String,
    heap_key: usize,
    req_id: u64,
    may_prompt: bool,
}

impl ServiceHost for SheetOps {
    fn open_sheet(&mut self, body: String) {
        if !self.may_prompt {
            Replier { heap_key: self.heap_key, req_id: self.req_id }
                .send(Err("this surface cannot raise a prompt; open the app to continue".into()));
            return;
        }
        hold_for_sheet(self.heap_key, self.req_id, &self.app_id);
        self.change = Some(Some(body));
    }
    fn close_sheet(&mut self) {
        resume_after_sheet(&self.app_id, Instant::now());
        self.change = Some(None);
    }
}

/// Sheet closes asked for from a worker thread, which has no ServiceHost at
/// hand, by app: applied by the next pump for that app. Every running app
/// pumps (a home screen shows several), so one app must not take another's.
static PENDING_CLOSE: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Close the sheet over `app_id`, from a service's worker (a sign-in that
/// finished).
pub fn close_sheet_later(app_id: &str) {
    resume_after_sheet(app_id, Instant::now());
    PENDING_CLOSE.lock().unwrap().push(app_id.to_string());
    SignalToUI::set_ui_signal();
}

fn heap_of(cx: &mut Cx, splash: &SplashRef, only_visible: bool) -> Option<usize> {
    if only_visible && !splash.borrow().map(|s| s.view.visible).unwrap_or(false) {
        return None;
    }
    let mut s = splash.borrow_mut()?;
    s.isolate_heap_key(cx)
}

fn apply_sheet(cx: &mut Cx, sheet: &SplashRef, change: Option<String>) {
    // A non-empty Splash::set_text reuses its isolate. A new program string
    // alone does not revoke the old sheet's callbacks, timers or queued work.
    // Stop it first, including replies that a worker has not delivered yet.
    if let Some(heap) = heap_of(cx, sheet, false) {
        cancel_heap(heap);
    }
    sheet.set_text(cx, "");
    let up = change.is_some();
    if let Some(body) = change {
        sheet.set_text(cx, &body);
    }
    // After the text: a new program, or tearing the old one down, replaces
    // the sheet's view, and the replacement is visible. A sheet left
    // visible and empty would stay modal and take every touch meant for
    // the app.
    if let Some(mut s) = sheet.borrow_mut() {
        s.view.visible = up;
    }
    cx.redraw_all();
}

/// One turn of a host running an app: hand the app's (and its sheet's) host
/// requests to the services, apply the sheets they raise, and deliver the
/// answers that are ready. Call it on every event the host sees.
pub fn pump(cx: &mut Cx, app_id: &str, host_dir: &std::path::Path, card: &SplashRef, sheet: &SplashRef) {
    // An answer runs the app's script, which may re-render a list, and a
    // redraw asked for during a draw is dropped: the new rows would take
    // taps without ever being drawn. Everything queued here signalled the
    // UI when it was queued, so it waits for that event instead.
    if cx.in_draw_event() {
        return;
    }
    let app = heap_of(cx, card, false);
    let sheet_heap = heap_of(cx, sheet, true);
    let heaps: Vec<usize> = app.into_iter().chain(sheet_heap).collect();
    if heaps.is_empty() {
        return;
    }
    #[cfg(target_arch = "wasm32")]
    {
        let now = Instant::now();
        expire_pending(now, |_| true);
        purge_replies(now, |_| true);
    }
    // Requests drained together may span a replacement: the first request can
    // open another service's sheet before a queued close/submit from the old
    // sheet is visited. Heap addresses may also be reused after teardown, so
    // invalidate this batch's sheet authority explicitly on every change.
    // A replacement's startup requests remain queued for the next pump.
    let mut original_sheet_valid = true;
    for request in makepad_widgets::splash_host::take_splash_host_requests_for(&heaps) {
        let current_app = heap_of(cx, card, false);
        let current_sheet = if original_sheet_valid { heap_of(cx, sheet, true) } else { None };
        let from_sheet = Some(request.heap_key) == current_sheet;
        if Some(request.heap_key) != current_app && !from_sheet {
            // Its surface went away earlier in this batch. Never downgrade it
            // to an app request: ordinary methods can also mutate host state.
            continue;
        }
        let args = match request_args(&request.service, &request.args_json) {
            Ok(args) => args,
            Err(refusal) => {
                queue_reply(request.heap_key, request.req_id, Err(refusal));
                continue;
            }
        };
        let call = ServiceCall {
            app_id: app_id.to_string(),
            service: request.service.clone(),
            args,
            from_sheet,
            may_prompt: request.may_prompt,
            host_dir: host_dir.to_path_buf(),
        };
        let mut ops = SheetOps {
            change: None,
            app_id: app_id.to_string(),
            heap_key: request.heap_key,
            req_id: request.req_id,
            may_prompt: request.may_prompt,
        };
        dispatch(call, request.heap_key, request.req_id, &mut ops);
        if let Some(change) = ops.change {
            original_sheet_valid = false;
            apply_sheet(cx, sheet, change);
        }
    }
    for (heap, req_id, result) in take_replies_for(&heaps) {
        let answer = match &result {
            Ok(json) => Ok(json.as_str()),
            Err(error) => Err(error.as_str()),
        };
        makepad_widgets::splash_host::splash_host_respond(cx, heap, req_id, answer);
    }
    let closing = {
        let mut pending = PENDING_CLOSE.lock().unwrap();
        let before = pending.len();
        pending.retain(|app| app != app_id);
        pending.len() != before
    };
    if closing {
        apply_sheet(cx, sheet, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct Echo;
    impl HostService for Echo {
        fn family(&self) -> &'static str {
            "echo"
        }
        fn call(&mut self, call: ServiceCall, reply: Replier, host: &mut dyn ServiceHost) {
            if call.method() == "sheet" {
                host.open_sheet("Label{text: \"hi\"}".into());
            }
            reply.send(Ok(serde_json::json!({"method": call.method(), "sheet": call.from_sheet})));
        }
    }

    #[derive(Default)]
    struct Host {
        sheet: Option<String>,
    }
    impl ServiceHost for Host {
        fn open_sheet(&mut self, body: String) {
            self.sheet = Some(body);
        }
        fn close_sheet(&mut self) {
            self.sheet = None;
        }
    }

    fn call(service: &str) -> ServiceCall {
        ServiceCall {
            app_id: "os.demo".into(),
            service: service.into(),
            args: Value::Null,
            from_sheet: false,
            may_prompt: true,
            host_dir: std::env::temp_dir(),
        }
    }

    /// Answers later, or never: the service keeps each request's replier.
    struct Holds {
        family: &'static str,
        timeout: Option<Duration>,
        sheet: bool,
        held: Arc<Mutex<Vec<Replier>>>,
    }
    impl HostService for Holds {
        fn family(&self) -> &'static str {
            self.family
        }
        fn timeout(&self, _call: &ServiceCall) -> Duration {
            self.timeout.unwrap_or(DEFAULT_TIMEOUT)
        }
        fn call(&mut self, _call: ServiceCall, reply: Replier, host: &mut dyn ServiceHost) {
            if self.sheet {
                host.open_sheet("Label{text: \"sign in\"}".into());
            }
            self.held.lock().unwrap().push(reply);
        }
    }

    fn holds(family: &'static str, timeout: Option<Duration>, sheet: bool) -> Arc<Mutex<Vec<Replier>>> {
        let held = Arc::new(Mutex::new(Vec::new()));
        register_host_service(Box::new(Holds { family, timeout, sheet, held: held.clone() }));
        held
    }

    fn sheet_ops(app_id: &str, heap_key: usize, req_id: u64, may_prompt: bool) -> SheetOps {
        SheetOps { change: None, app_id: app_id.into(), heap_key, req_id, may_prompt }
    }

    fn later(by: Duration) -> Instant {
        Instant::now() + by
    }

    #[test]
    fn a_request_reaches_its_familys_service_and_the_answer_waits_for_its_isolate() {
        register_host_service(Box::new(Echo));
        let mut host = Host::default();
        dispatch(call("echo.sheet"), 7001, 1, &mut host);
        dispatch(call("nobody.here"), 7002, 2, &mut host);
        assert!(host.sheet.is_some(), "a service can raise a sheet");
        let mine = take_replies_for(&[7001]);
        assert_eq!(mine.len(), 1);
        assert!(mine[0].2.as_ref().unwrap().contains("\"method\":\"sheet\""));
        let other = take_replies_for(&[7002]);
        assert!(other[0].2.as_ref().unwrap_err().contains("no service"), "an unanswered family fails, not hangs");
    }

    #[test]
    fn only_the_sheet_reaches_a_sheet_method() {
        register_host_service(Box::new(Echo));
        let mut host = Host::default();
        dispatch(call("echo.sheet.submit"), 7011, 1, &mut host);
        let refused = take_replies_for(&[7011]);
        assert!(refused[0].2.as_ref().unwrap_err().contains("for the host's sheet"), "an app is refused before the service sees it");
        let mut from_sheet = call("echo.sheet.submit");
        from_sheet.from_sheet = true;
        dispatch(from_sheet, 7012, 1, &mut host);
        assert!(take_replies_for(&[7012])[0].2.as_ref().unwrap().contains("\"sheet\":true"));
    }


    #[test]
    fn a_request_nobody_answers_times_out() {
        let _held = holds("quiet", None, false);
        dispatch(call("quiet.read"), 7101, 1, &mut Host::default());
        assert!(take_replies_for(&[7101]).is_empty(), "still waiting");
        expire_pending(later(DEFAULT_TIMEOUT + Duration::from_secs(1)), |h| h == 7101);
        let answer = take_replies_for(&[7101]);
        assert_eq!(answer.len(), 1);
        assert!(answer[0].2.as_ref().unwrap_err().contains("timed out"), "{answer:?}");
    }

    #[test]
    fn an_answer_after_the_timeout_is_dropped() {
        let held = holds("late", None, false);
        dispatch(call("late.read"), 7102, 1, &mut Host::default());
        expire_pending(later(DEFAULT_TIMEOUT + Duration::from_secs(1)), |h| h == 7102);
        assert_eq!(take_replies_for(&[7102]).len(), 1, "the timeout");
        held.lock().unwrap().pop().unwrap().send(Ok(Value::Null));
        assert!(take_replies_for(&[7102]).is_empty(), "one answer per request");
    }

    #[test]
    fn a_service_may_ask_for_more_time() {
        let _held = holds("slow", Some(Duration::from_secs(300)), false);
        dispatch(call("slow.think"), 7103, 1, &mut Host::default());
        expire_pending(later(DEFAULT_TIMEOUT + Duration::from_secs(1)), |h| h == 7103);
        assert!(take_replies_for(&[7103]).is_empty(), "within the service's own time");
        expire_pending(later(Duration::from_secs(301)), |h| h == 7103);
        assert!(take_replies_for(&[7103])[0].2.as_ref().unwrap_err().contains("timed out"));
    }

    #[test]
    fn closing_an_isolate_drops_its_requests_and_its_answers() {
        let held = holds("closing", None, false);
        dispatch(call("closing.read"), 7104, 1, &mut Host::default());
        dispatch(call("closing.read"), 7104, 2, &mut Host::default());
        held.lock().unwrap().pop().unwrap().send(Ok(Value::Null)); // answered, not yet taken
        cancel_heap(7104);
        assert!(take_replies_for(&[7104]).is_empty(), "a queued answer goes with the isolate");
        held.lock().unwrap().pop().unwrap().send(Ok(Value::Null));
        expire_pending(later(DEFAULT_TIMEOUT + Duration::from_secs(1)), |h| h == 7104);
        assert!(take_replies_for(&[7104]).is_empty(), "nothing reaches a replacement isolate");
    }

    #[test]
    fn a_background_surface_cannot_raise_a_sheet() {
        let _held = holds("signin_bg", None, true);
        let mut ops = sheet_ops("os.demo", 7105, 1, false);
        let mut background = call("signin_bg.add");
        background.may_prompt = false;
        dispatch(background, 7105, 1, &mut ops);
        assert!(ops.change.is_none(), "no sheet over a tile");
        let answer = take_replies_for(&[7105]);
        assert_eq!(answer.len(), 1);
        assert!(answer[0].2.as_ref().unwrap_err().contains("cannot raise a prompt"), "{answer:?}");
    }

    #[test]
    fn a_sheet_holds_its_request_open_until_it_closes() {
        let _held = holds("signin_fg", None, true);
        let mut ops = sheet_ops("os.sheet-holder", 7106, 1, true);
        let mut foreground = call("signin_fg.add");
        foreground.app_id = "os.sheet-holder".into();
        dispatch(foreground, 7106, 1, &mut ops);
        assert!(matches!(ops.change, Some(Some(_))), "the sheet is up");
        expire_pending(later(Duration::from_secs(3600)), |h| h == 7106);
        assert!(take_replies_for(&[7106]).is_empty(), "the person is still typing");
        ops.close_sheet();
        expire_pending(later(DEFAULT_TIMEOUT + Duration::from_secs(1)), |h| h == 7106);
        assert!(take_replies_for(&[7106])[0].2.as_ref().unwrap_err().contains("timed out"), "a closed sheet restarts the clock");
    }

    #[test]
    fn one_isolate_cannot_queue_unbounded_requests() {
        let _held = holds("flood", None, false);
        for req in 0..MAX_PENDING_PER_ISOLATE as u64 {
            dispatch(call("flood.read"), 7107, req, &mut Host::default());
        }
        assert!(take_replies_for(&[7107]).is_empty());
        dispatch(call("flood.read"), 7107, 999, &mut Host::default());
        let refused = take_replies_for(&[7107]);
        assert_eq!(refused.len(), 1);
        assert!(refused[0].2.as_ref().unwrap_err().contains("too many"), "{refused:?}");
        cancel_heap(7107);
    }

    #[test]
    fn malformed_or_oversized_arguments_are_refused_not_passed_on_as_null() {
        assert!(request_args("mail.list", "{").unwrap_err().contains("not valid JSON"));
        assert!(request_args("mail.list", &format!("\"{}\"", "x".repeat(MAX_ARGS_BYTES))).unwrap_err().contains("1 MiB"));
        assert!(request_args("mail", "{}").unwrap_err().contains("family.method"));
        assert_eq!(request_args("mail.list", "null").unwrap(), Value::Null);
    }

    #[test]
    fn an_oversized_answer_becomes_an_error() {
        let held = holds("big", None, false);
        dispatch(call("big.read"), 7108, 1, &mut Host::default());
        held.lock().unwrap().pop().unwrap().send(Ok(Value::String("x".repeat(MAX_REPLY_BYTES))));
        assert!(take_replies_for(&[7108])[0].2.as_ref().unwrap_err().contains("4 MiB"));
    }

    #[test]
    fn answers_nobody_collects_expire() {
        register_host_service(Box::new(Echo));
        dispatch(call("echo.read"), 7109, 1, &mut Host::default());
        purge_replies(later(REPLY_TTL + Duration::from_secs(1)), |h| h == 7109);
        assert!(take_replies_for(&[7109]).is_empty(), "an isolate that went away without closing leaves nothing behind");
    }

    #[test]
    fn replacing_a_sheet_revokes_its_queued_requests_and_late_replies() {
        use makepad_widgets::*;

        struct LifecycleProbe {
            family: &'static str,
            seen: Arc<Mutex<Vec<String>>>,
            held: Arc<Mutex<Option<Replier>>>,
        }
        impl HostService for LifecycleProbe {
            fn family(&self) -> &'static str { self.family }
            fn call(&mut self, call: ServiceCall, reply: Replier, host: &mut dyn ServiceHost) {
                self.seen.lock().unwrap().push(call.service.clone());
                match call.method() {
                    "hold" => { *self.held.lock().unwrap() = Some(reply); return; }
                    "open" => host.open_sheet(r#"
                        let previous = try {mod.previous_sheet_secret} catch {nil}
                        host.request("sheet_lifecycle_new.sheet.ready", {previous: previous}, fn(r){})
                        Label {text: "replacement"}
                    "#.into()),
                    "sheet.close" => host.close_sheet(),
                    "sheet.ready" => {
                        assert!(call.from_sheet, "the replacement has its own sheet authority");
                        assert!(call.args["previous"].is_null(), "old sheet state crossed into the replacement");
                    }
                    _ => {}
                }
                reply.send(Ok(Value::Null));
            }
        }

        let seen = Arc::new(Mutex::new(Vec::new()));
        let held = Arc::new(Mutex::new(None));
        for family in ["sheet_lifecycle_old", "sheet_lifecycle_new"] {
            register_host_service(Box::new(LifecycleProbe {family, seen: seen.clone(), held: held.clone()}));
        }
        let mut cx = Cx::new(Box::new(|_, _| {}));
        let root = cx.with_vm(|vm| {
            makepad_widgets::script_mod(vm);
            let value = script_eval!(vm, {
                use mod.widgets.*
                View {card := Splash{} sheet := Splash{visible: false}}
            });
            WidgetRef::script_from_value(vm, value)
        });
        let card = root.splash(&mut cx, ids!(card));
        let sheet = root.splash(&mut cx, ids!(sheet));
        card.set_text(&mut cx, "Label{text: \"app\"}");
        apply_sheet(&mut cx, &sheet, Some(r#"
            mod.previous_sheet_secret = "must not survive"
            host.request("sheet_lifecycle_old.hold", {}, fn(r){})
            host.request("sheet_lifecycle_new.open", {}, fn(r){})
            host.request("sheet_lifecycle_old.sheet.close", {}, fn(r){})
            host.request("sheet_lifecycle_old.mutate", {}, fn(r){})
            Label {text: "old sheet"}
        "#.into()));
        let old_heap = heap_of(&mut cx, &sheet, true).expect("old sheet has an isolate");
        pump(&mut cx, "org.example.sheet-lifecycle", &std::env::temp_dir(), &card, &sheet);
        assert_eq!(*seen.lock().unwrap(), ["sheet_lifecycle_old.hold", "sheet_lifecycle_new.open"]);
        assert!(sheet.borrow().unwrap().view.visible, "the stale close must not hide the replacement");
        held.lock().unwrap().take().unwrap().send(Ok(Value::Null));
        assert!(take_replies_for(&[old_heap]).is_empty(), "the retired sheet's worker cannot reply late");

        pump(&mut cx, "org.example.sheet-lifecycle", &std::env::temp_dir(), &card, &sheet);
        assert_eq!(*seen.lock().unwrap(), [
            "sheet_lifecycle_old.hold", "sheet_lifecycle_new.open", "sheet_lifecycle_new.sheet.ready"
        ], "only the replacement's new request may reach a service");
        apply_sheet(&mut cx, &sheet, None);
        card.set_text(&mut cx, "");
    }
}
