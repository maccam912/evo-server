use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};

pub fn find_latest_checkpoint(directory: &str) -> Option<PathBuf> {
    let dir = Path::new(directory);
    if !dir.exists() {
        return None;
    }

    let mut checkpoints: Vec<(PathBuf, DateTime<Utc>)> = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let datetime: DateTime<Utc> = modified.into();
                        checkpoints.push((path, datetime));
                    }
                }
            }
        }
    }

    checkpoints.sort_by(|a, b| b.1.cmp(&a.1));
    checkpoints.first().map(|(path, _)| path.clone())
}

pub fn create_checkpoint_path(directory: &str) -> PathBuf {
    let now: DateTime<Utc> = Utc::now();
    let filename = format!("checkpoint_{}.json", now.format("%Y-%m-%d_%H-%M-%S"));

    Path::new(directory).join(filename)
}

pub fn cleanup_old_checkpoints(directory: &str, keep_last_n: usize) -> Result<(), Box<dyn std::error::Error>> {
    let dir = Path::new(directory);
    if !dir.exists() {
        return Ok(());
    }

    let mut checkpoints: Vec<(PathBuf, DateTime<Utc>)> = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let datetime: DateTime<Utc> = modified.into();
                        checkpoints.push((path, datetime));
                    }
                }
            }
        }
    }

    checkpoints.sort_by(|a, b| b.1.cmp(&a.1));

    for (path, _) in checkpoints.iter().skip(keep_last_n) {
        log::info!("Deleting old checkpoint: {:?}", path);
        let _ = fs::remove_file(path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_checkpoint_path() {
        let path = create_checkpoint_path("test_checkpoints");
        assert!(path.to_str().unwrap().starts_with("test_checkpoints"));
        assert!(path.to_str().unwrap().ends_with(".json"));
        assert!(path.to_str().unwrap().contains("checkpoint_"));
    }
}
