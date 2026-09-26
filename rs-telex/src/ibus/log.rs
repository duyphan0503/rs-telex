use std::fs::OpenOptions;
use std::io::Write;
use std::time::SystemTime;

pub fn log_info(msg: &str) {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/rs-telex.log")
    {
        let _ = writeln!(file, "[{}] {}", now, msg);
    }
}
