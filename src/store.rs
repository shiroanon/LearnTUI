use std::{fs, path::Path, error::Error};
use crate::models::Progress;

const PROGRESS_FILE: &str = "progress.json";

pub fn load_progress() -> Result<Progress, Box<dyn Error>> {
    let path = Path::new(PROGRESS_FILE);
    if path.exists() {
        let data = fs::read_to_string(path)?;
        let progress: Progress = serde_json::from_str(&data)?;
        Ok(progress)
    } else {
        Ok(Progress::default())
    }
}

pub fn save_progress(progress: &Progress) -> Result<(), Box<dyn Error>> {
    let data = serde_json::to_string_pretty(progress)?;
    fs::write(PROGRESS_FILE, data)?;
    Ok(())
}
