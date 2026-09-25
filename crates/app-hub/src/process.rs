use std::{path::Path, time::Duration};

pub(crate) struct Limits { pub wall: Duration, pub output: usize, pub isolated_env: bool }
impl Default for Limits {
    fn default() -> Self { Self { wall: Duration::from_secs(10), output: 64 * 1024, isolated_env: true } }
}

#[cfg(not(unix))]
pub(crate) fn run(_executable: &Path, _args: &[&std::ffi::OsStr], _cwd: &Path, _input: Vec<u8>, _limits: Limits) -> Result<Vec<u8>, String> {
    Err("bounded validation workers require a supported Unix runner".into())
}

#[cfg(unix)]
pub(crate) fn run(executable: &Path, args: &[&std::ffi::OsStr], cwd: &Path, input: Vec<u8>, limits: Limits) -> Result<Vec<u8>, String> {
    use std::{io::{Read, Write}, os::unix::process::CommandExt, process::{Command, Stdio}, sync::mpsc, time::Instant};
    if input.len() > 8 * 1024 * 1024 { return Err("worker input exceeds size limit".into()); }
    let mut command = Command::new(executable);
    command.args(args).current_dir(cwd).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    if limits.isolated_env { command.env_clear().env("PATH", "/usr/bin:/bin").env("LANG", "C"); }
    let cpu = limits.wall.as_secs().saturating_add(1).max(1) as libc::rlim_t;
    // Only async-signal-safe libc operations between fork and exec. The child
    // has its own process group so timeouts also close inherited pipe writers.
    unsafe {
        command.pre_exec(move || {
            if libc::setsid() < 0 { return Err(std::io::Error::last_os_error()); }
            for (resource, value) in [(libc::RLIMIT_CPU, cpu), (libc::RLIMIT_CORE, 0), (libc::RLIMIT_FSIZE, 1024 * 1024)] {
                let limit = libc::rlimit { rlim_cur: value, rlim_max: value };
                if libc::setrlimit(resource, &limit) != 0 { return Err(std::io::Error::last_os_error()); }
            }
            #[cfg(target_os = "linux")]
            {
                let limit = libc::rlimit { rlim_cur: 1024 * 1024 * 1024, rlim_max: 1024 * 1024 * 1024 };
                if libc::setrlimit(libc::RLIMIT_AS, &limit) != 0 { return Err(std::io::Error::last_os_error()); }
            }
            Ok(())
        });
    }
    struct Worker(std::process::Child);
    impl Drop for Worker {
        fn drop(&mut self) {
            unsafe { libc::kill(-(self.0.id() as i32), libc::SIGKILL); }
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut worker = Worker(command.spawn().map_err(|e| format!("cannot start validator/reviewer: {e}"))?);
    let mut stdin = worker.0.stdin.take().unwrap();
    std::thread::spawn(move || { let _ = stdin.write_all(&input); });
    let (sender, receiver) = mpsc::channel();
    fn read_output(reader: impl Read + Send + 'static, sender: mpsc::Sender<(bool, Result<Vec<u8>, String>)>, stdout: bool, limit: usize) {
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = reader.take(limit as u64 + 1).read_to_end(&mut bytes).map_err(|e| e.to_string()).and_then(|_| {
                if bytes.len() > limit { Err("worker output exceeds size limit".into()) } else { Ok(bytes) }
            });
            let _ = sender.send((stdout, result));
        });
    }
    read_output(worker.0.stdout.take().unwrap(), sender.clone(), true, limits.output);
    read_output(worker.0.stderr.take().unwrap(), sender, false, limits.output);
    let mut stdout = None;
    let mut stderr = None;
    let started = Instant::now();
    loop {
        while let Ok((is_stdout, result)) = receiver.try_recv() {
            if is_stdout { stdout = Some(result?); } else { stderr = Some(result?); }
        }
        if let Some(status) = worker.0.try_wait().map_err(|e| e.to_string())? {
            // Even a successful parent cannot leave helpers holding its pipes.
            unsafe { libc::kill(-(worker.0.id() as i32), libc::SIGKILL); }
            if let (Some(out), Some(err)) = (stdout.as_ref(), stderr.as_ref()) {
                if !status.success() { return Err(format!("worker exited with {status}: {}", String::from_utf8_lossy(err))); }
                return Ok(out.clone());
            }
        }
        if started.elapsed() >= limits.wall { return Err("validator/reviewer timed out".into()); }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    fn shell(script: &str, limits: Limits) -> Result<Vec<u8>, String> {
        run(Path::new("/bin/sh"), &["-c".as_ref(), script.as_ref()], &std::env::temp_dir(), vec![], limits)
    }
    #[test]
    fn successful_worker_returns_bounded_output() {
        assert_eq!(shell("printf '{\"ok\":true}'", Limits::default()).unwrap(), b"{\"ok\":true}");
    }
    #[test]
    fn crashed_or_hung_workers_never_pass() {
        assert!(shell("exit 7", Limits::default()).is_err());
        let started = std::time::Instant::now();
        assert!(shell("sleep 30", Limits { wall: Duration::from_millis(100), ..Default::default() }).unwrap_err().contains("timed out"));
        assert!(started.elapsed() < Duration::from_secs(2));
    }
    #[test]
    fn output_flood_and_inherited_credentials_are_refused() {
        assert!(shell("while :; do printf 1234567890; done", Limits { output: 100, ..Default::default() }).unwrap_err().contains("output"));
        let output = shell("env", Limits::default()).unwrap();
        let env = String::from_utf8(output).unwrap();
        assert!(!env.contains("OCTOSENSE_HUB") && !env.contains("HOME="));
    }
}
