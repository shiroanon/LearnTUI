use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub completed_files: Vec<String>,
    #[serde(default)]
    pub last_index: usize,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            completed_files: Vec::new(),
            last_index: 0,
        }
    }
}

use ratatui::text::Line;

/// Represents a single Markdown lesson file
#[derive(Debug, Clone)]
pub struct Lesson {
    pub path: String,
    pub title: String,
    pub content: String,
    pub text_lines: Vec<Line<'static>>,
}
