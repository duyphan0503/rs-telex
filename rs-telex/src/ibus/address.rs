use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Finds the IBus session bus address.
///
/// Looks first at $IBUS_ADDRESS, then checks the configuration directory
/// `~/.config/ibus/bus/` for the address file matching current display (Wayland / X11),
/// with fallback to the newest socket file found.
pub fn get_ibus_address() -> Option<String> {
    // 1. Direct environment variable
    if let Ok(addr) = env::var("IBUS_ADDRESS") {
        let trimmed = addr.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    // 2. Look in ~/.config/ibus/bus/
    let home = env::var("HOME").ok()?;
    let bus_dir = PathBuf::from(home).join(".config/ibus/bus");
    if !bus_dir.is_dir() {
        return None;
    }

    // Identify display target
    let wayland = env::var("WAYLAND_DISPLAY").ok();
    let x11 = env::var("DISPLAY").ok().map(|d| {
        d.trim_start_matches(':')
            .split('.')
            .next()
            .unwrap_or("0")
            .to_string()
    });

    let entries = fs::read_dir(&bus_dir).ok()?;
    let mut files = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let file_name_opt = path
                .file_name()
                .and_then(|f| f.to_str())
                .map(|s| s.to_string());
            if let Some(file_name) = file_name_opt
                && file_name.contains("-unix-")
            {
                let metadata = entry.metadata().ok();
                let mtime = metadata.and_then(|m| m.modified().ok());
                files.push((path, file_name, mtime));
            }
        }
    }

    // Match Wayland display if available
    if let Some(ref wl) = wayland {
        let pattern = format!("-unix-{}", wl);
        for (path, name, _) in &files {
            if name.ends_with(&pattern)
                && let Some(addr) = read_address_from_file(path)
            {
                return Some(addr);
            }
        }
    }

    // Match X11 display if available
    if let Some(ref disp) = x11 {
        let pattern = format!("-unix-{}", disp);
        for (path, name, _) in &files {
            if name.ends_with(&pattern)
                && let Some(addr) = read_address_from_file(path)
            {
                return Some(addr);
            }
        }
    }

    // Fallback: sort files by newest mtime
    files.sort_by_key(|a| std::cmp::Reverse(a.2));
    for (path, _, _) in files {
        if let Some(addr) = read_address_from_file(&path) {
            return Some(addr);
        }
    }

    None
}

fn read_address_from_file(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let line = line.trim();
        if let Some(stripped) = line.strip_prefix("IBUS_ADDRESS=") {
            let addr = stripped.trim();
            if !addr.is_empty() {
                return Some(addr.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_resolution() {
        let addr = get_ibus_address();
        assert!(addr.is_some(), "Should find active IBus address on desktop");
        let a = addr.unwrap();
        assert!(
            a.starts_with("unix:path="),
            "Address format unexpected: {}",
            a
        );
    }
}
